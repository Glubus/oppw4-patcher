use std::ffi::c_void;

type Dword = u32;
type Hmodule = *mut c_void;
type Lpvoid = *mut c_void;

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> Hmodule;
    fn VirtualProtect(
        address: Lpvoid,
        size: usize,
        new_protect: Dword,
        old_protect: *mut Dword,
    ) -> i32;
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
