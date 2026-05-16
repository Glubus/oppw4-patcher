use std::{ffi::c_void, ptr, slice};

use crate::win;

pub fn module_base() -> usize {
    win::main_module() as usize
}

pub unsafe fn read_memory(address: usize, out: *mut u8, len: usize) -> i32 {
    if address == 0 || out.is_null() {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    ptr::copy_nonoverlapping(address as *const u8, out, len);
    0
}

pub unsafe fn write_memory(address: usize, bytes: *const u8, len: usize) -> i32 {
    if address == 0 || bytes.is_null() {
        return -1;
    }
    if len == 0 {
        return 0;
    }

    let mut old_protect = 0;
    if !win::make_memory_writable(address as *mut c_void, len, &mut old_protect) {
        return -2;
    }
    ptr::copy_nonoverlapping(bytes, address as *mut u8, len);
    let _ = win::flush_instruction_cache(address as *const c_void, len);
    let _ = win::restore_memory_protection(address as *mut c_void, len, old_protect);
    0
}

pub unsafe fn scan_memory(pattern: *const u8, mask: *const u8, len: usize) -> usize {
    if pattern.is_null() || mask.is_null() || len == 0 {
        return 0;
    }
    let base = module_base();
    let Some(size) = module_image_size(base) else {
        return 0;
    };
    let image = slice::from_raw_parts(base as *const u8, size);
    let pattern = slice::from_raw_parts(pattern, len);
    let mask = slice::from_raw_parts(mask, len);
    scan_slice(image, pattern, mask)
        .map(|offset| base + offset)
        .unwrap_or(0)
}

unsafe fn module_image_size(module: usize) -> Option<usize> {
    if module == 0 {
        return None;
    }
    let dos = module as *const u8;
    if slice::from_raw_parts(dos, 2) != b"MZ" {
        return None;
    }
    let nt_offset = *(dos.add(0x3c) as *const u32) as usize;
    let nt = dos.add(nt_offset);
    if slice::from_raw_parts(nt, 4) != b"PE\0\0" {
        return None;
    }
    let optional_header = nt.add(24);
    let magic = *(optional_header as *const u16);
    if magic != 0x20b {
        return None;
    }
    Some(*(optional_header.add(0x38) as *const u32) as usize)
}

fn scan_slice(haystack: &[u8], pattern: &[u8], mask: &[u8]) -> Option<usize> {
    if pattern.is_empty() || pattern.len() != mask.len() || pattern.len() > haystack.len() {
        return None;
    }
    haystack.windows(pattern.len()).position(|window| {
        window
            .iter()
            .zip(pattern.iter())
            .zip(mask.iter())
            .all(|((byte, expected), mask)| *mask == 0 || byte == expected)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_slice_supports_wildcards() {
        let haystack = [0x10, 0x44, 0x8b, 0xaa, 0x41, 0xff];
        let pattern = [0x44, 0x8b, 0x00, 0x41];
        let mask = [1, 1, 0, 1];

        assert_eq!(scan_slice(&haystack, &pattern, &mask), Some(1));
    }

    #[test]
    fn scan_slice_rejects_mismatched_mask() {
        assert_eq!(scan_slice(&[1, 2, 3], &[2, 3], &[1]), None);
    }
}
