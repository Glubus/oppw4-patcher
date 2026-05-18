mod r#unsafe;

pub(super) use r#unsafe::{
    host_module_base, host_read_memory, host_scan_memory, host_write_memory,
};

fn module_base() -> usize {
    hooks::module_base()
}

fn read_memory(address: usize, out: *mut u8, len: usize) -> i32 {
    hooks::read_memory(address, out, len)
}

fn write_memory(address: usize, bytes: *const u8, len: usize) -> i32 {
    hooks::write_memory(address, bytes, len)
}

fn scan_memory(pattern: *const u8, mask: *const u8, len: usize) -> usize {
    hooks::scan_memory(pattern, mask, len)
}
