use std::{
    ffi::{c_char, c_void},
    mem::transmute,
    path::PathBuf,
    ptr::null_mut,
    sync::OnceLock,
};

use crate::log;

type Dword = u32;
type Hinstance = *mut c_void;
type Hmodule = *mut c_void;
type Hresult = i32;
type Lpvoid = *mut c_void;

const MAX_PATH: usize = 260;
const LOAD_LIBRARY_FAILED: Hmodule = null_mut();

#[repr(C)]
pub struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

pub type DirectInput8CreateFn =
    unsafe extern "system" fn(Hinstance, Dword, *const Guid, *mut Lpvoid, *mut c_void) -> Hresult;
pub type DllCanUnloadNowFn = unsafe extern "system" fn() -> Hresult;
pub type DllGetClassObjectFn =
    unsafe extern "system" fn(*const Guid, *const Guid, *mut Lpvoid) -> Hresult;
pub type DllRegisterServerFn = unsafe extern "system" fn() -> Hresult;
pub type DllUnregisterServerFn = unsafe extern "system" fn() -> Hresult;

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    fn GetModuleFileNameW(module: Hmodule, buffer: *mut u16, size: u32) -> u32;
    fn LoadLibraryW(path: *const u16) -> Hmodule;
    fn GetProcAddress(module: Hmodule, name: *const c_char) -> *mut c_void;
    fn GetModuleHandleW(module_name: *const u16) -> Hmodule;
    fn VirtualProtect(
        address: Lpvoid,
        size: usize,
        new_protect: Dword,
        old_protect: *mut Dword,
    ) -> i32;
    fn VirtualAlloc(address: Lpvoid, size: usize, allocation_type: Dword, protect: Dword)
        -> Lpvoid;
    fn FlushInstructionCache(process: *mut c_void, address: Lpvoid, size: usize) -> i32;
    fn RtlCaptureStackBackTrace(
        frames_to_skip: Dword,
        frames_to_capture: Dword,
        back_trace: *mut *mut c_void,
        back_trace_hash: *mut Dword,
    ) -> u16;
}

pub unsafe fn direct_input8_create() -> DirectInput8CreateFn {
    transmute(load_system_dinput8_proc(b"DirectInput8Create\0"))
}

pub unsafe fn dll_can_unload_now() -> DllCanUnloadNowFn {
    transmute(load_system_dinput8_proc(b"DllCanUnloadNow\0"))
}

pub unsafe fn dll_get_class_object() -> DllGetClassObjectFn {
    transmute(load_system_dinput8_proc(b"DllGetClassObject\0"))
}

pub unsafe fn dll_register_server() -> DllRegisterServerFn {
    transmute(load_system_dinput8_proc(b"DllRegisterServer\0"))
}

pub unsafe fn dll_unregister_server() -> DllUnregisterServerFn {
    transmute(load_system_dinput8_proc(b"DllUnregisterServer\0"))
}

pub fn module_directory(module: Hmodule) -> Option<PathBuf> {
    let mut buffer = [0u16; MAX_PATH];
    let len =
        unsafe { GetModuleFileNameW(module, buffer.as_mut_ptr(), buffer.len() as u32) } as usize;
    if len == 0 || len >= buffer.len() {
        return None;
    }

    let path = PathBuf::from(String::from_utf16_lossy(&buffer[..len]));
    path.parent().map(PathBuf::from)
}

pub fn main_module() -> Hmodule {
    unsafe { GetModuleHandleW(std::ptr::null()) }
}

pub unsafe fn make_memory_writable(address: Lpvoid, size: usize, old_protect: *mut Dword) -> bool {
    const PAGE_READWRITE: Dword = 0x04;
    VirtualProtect(address, size, PAGE_READWRITE, old_protect) != 0
}

pub unsafe fn restore_memory_protection(address: Lpvoid, size: usize, old_protect: Dword) -> bool {
    let mut ignored = 0;
    VirtualProtect(address, size, old_protect, &mut ignored) != 0
}

pub fn allocate_executable_memory(size: usize) -> Option<usize> {
    if size == 0 {
        return None;
    }
    const MEM_COMMIT: Dword = 0x1000;
    const MEM_RESERVE: Dword = 0x2000;
    const PAGE_EXECUTE_READWRITE: Dword = 0x40;
    let ptr = unsafe {
        VirtualAlloc(
            std::ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };
    (!ptr.is_null()).then_some(ptr as usize)
}

pub fn flush_instruction_cache(address: usize, size: usize) -> bool {
    unsafe { FlushInstructionCache(GetCurrentProcess(), address as Lpvoid, size) != 0 }
}

pub fn capture_stack_backtrace(frames_to_skip: u32, max_frames: usize) -> Vec<usize> {
    const MAX_FRAMES: usize = 16;
    let count = max_frames.min(MAX_FRAMES);
    if count == 0 {
        return Vec::new();
    }

    let mut frames = [std::ptr::null_mut::<c_void>(); MAX_FRAMES];
    let captured = unsafe {
        RtlCaptureStackBackTrace(
            frames_to_skip,
            count as Dword,
            frames.as_mut_ptr(),
            std::ptr::null_mut(),
        )
    } as usize;
    frames[..captured.min(count)]
        .iter()
        .filter_map(|frame| {
            let address = *frame as usize;
            (address != 0).then_some(address)
        })
        .collect()
}

pub fn write_process_memory(address: usize, bytes: &[u8]) -> Option<usize> {
    if address == 0 || bytes.is_empty() {
        return None;
    }
    let mut old_protect = 0;
    let target = address as Lpvoid;
    unsafe {
        if !make_memory_writable(target, bytes.len(), &mut old_protect) {
            return None;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), address as *mut u8, bytes.len());
        let _ = restore_memory_protection(target, bytes.len(), old_protect);
    }
    Some(bytes.len())
}

unsafe fn load_system_dinput8_proc(name: &'static [u8]) -> *mut c_void {
    let module = system_dinput8_module();
    let proc = GetProcAddress(module, name.as_ptr().cast());
    if proc.is_null() {
        abort_proxy();
    }
    proc
}

fn system_dinput8_module() -> Hmodule {
    static MODULE: OnceLock<usize> = OnceLock::new();
    let module = *MODULE.get_or_init(load_system_dinput8_module);
    module as Hmodule
}

fn load_system_dinput8_module() -> usize {
    let path = system_dinput8_path();
    log::write_line(format!(
        "loading original dinput8 from {}",
        wide_path_to_string(&path)
    ));
    let module = unsafe { LoadLibraryW(path.as_ptr()) };
    if module == LOAD_LIBRARY_FAILED {
        log::write_line("failed to load original dinput8");
        abort_proxy();
    }
    log::write_line(format!(
        "original dinput8 loaded at 0x{:x}",
        module as usize
    ));
    module as usize
}

fn system_dinput8_path() -> Vec<u16> {
    let mut buffer = [0u16; MAX_PATH];
    let len = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) } as usize;
    if len == 0 || len >= buffer.len() {
        abort_proxy();
    }

    let mut path = buffer[..len].to_vec();
    path.extend("\\dinput8.dll".encode_utf16());
    path.push(0);
    path
}

fn wide_path_to_string(path: &[u16]) -> String {
    let end = path
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(path.len());
    String::from_utf16_lossy(&path[..end])
}

fn abort_proxy() -> ! {
    std::process::abort()
}
