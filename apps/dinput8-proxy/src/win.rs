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

pub fn load_library(path: &[u16]) -> Hmodule {
    unsafe { LoadLibraryW(path.as_ptr()) }
}

pub unsafe fn get_proc_address(module: Hmodule, name: *const c_char) -> *mut c_void {
    GetProcAddress(module, name)
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
