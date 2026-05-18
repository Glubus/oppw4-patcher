use std::ffi::c_void;

pub(crate) unsafe extern "system" fn host_module_base(_host_context: *mut c_void) -> usize {
    super::module_base()
}

pub(crate) unsafe extern "system" fn host_read_memory(
    _host_context: *mut c_void,
    address: usize,
    out: *mut u8,
    len: usize,
) -> i32 {
    super::read_memory(address, out, len)
}

pub(crate) unsafe extern "system" fn host_write_memory(
    _host_context: *mut c_void,
    address: usize,
    bytes: *const u8,
    len: usize,
) -> i32 {
    super::write_memory(address, bytes, len)
}

pub(crate) unsafe extern "system" fn host_scan_memory(
    _host_context: *mut c_void,
    pattern: *const u8,
    mask: *const u8,
    len: usize,
) -> usize {
    super::scan_memory(pattern, mask, len)
}
