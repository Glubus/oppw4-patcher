// apps/dinput8-clean/src/changes/internal_hook.rs
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{log, win};

const ABSOLUTE_JUMP_LEN: usize = 14;

pub(crate) unsafe fn install_absolute_jump_hook(
    module: usize,
    rva: usize,
    stolen_len: usize,
    replacement: usize,
    original_storage: &'static AtomicUsize,
    label: &str,
) -> bool {
    if stolen_len < ABSOLUTE_JUMP_LEN {
        log::write_line(format!(
            "{label} hook skipped target=game+0x{rva:x} reason=stolen_len_too_short"
        ));
        return false;
    }
    if original_storage.load(Ordering::Acquire) != 0 {
        return true;
    }

    let target = module + rva;
    let Some(trampoline) = create_trampoline(target, stolen_len) else {
        log::write_line(format!(
            "{label} hook skipped target=game+0x{rva:x} reason=trampoline_failed"
        ));
        return false;
    };
    if !patch_absolute_jump(target, replacement, stolen_len) {
        log::write_line(format!(
            "{label} hook skipped target=game+0x{rva:x} reason=patch_failed"
        ));
        return false;
    }

    original_storage.store(trampoline, Ordering::Release);
    log::write_line(format!(
        "{label} hook installed target=game+0x{rva:x} trampoline=0x{trampoline:x}"
    ));
    true
}

unsafe fn create_trampoline(target: usize, stolen_len: usize) -> Option<usize> {
    let mut original = vec![0u8; stolen_len];
    std::ptr::copy_nonoverlapping(target as *const u8, original.as_mut_ptr(), stolen_len);

    let trampoline_size = stolen_len + ABSOLUTE_JUMP_LEN;
    let trampoline = win::allocate_executable_memory(trampoline_size)?;
    std::ptr::copy_nonoverlapping(original.as_ptr(), trampoline as *mut u8, stolen_len);
    let jump_back = absolute_jump_bytes(target + stolen_len);
    std::ptr::copy_nonoverlapping(
        jump_back.as_ptr(),
        (trampoline + stolen_len) as *mut u8,
        jump_back.len(),
    );
    let _ = win::flush_instruction_cache(trampoline, trampoline_size);
    Some(trampoline)
}

unsafe fn patch_absolute_jump(target: usize, replacement: usize, stolen_len: usize) -> bool {
    let mut patch = vec![0x90; stolen_len];
    patch[..ABSOLUTE_JUMP_LEN].copy_from_slice(&absolute_jump_bytes(replacement));
    let ok = matches!(win::write_process_memory(target, &patch), Some(written) if written == patch.len());
    let _ = win::flush_instruction_cache(target, patch.len());
    ok
}

fn absolute_jump_bytes(destination: usize) -> [u8; ABSOLUTE_JUMP_LEN] {
    let mut bytes = [0u8; ABSOLUTE_JUMP_LEN];
    bytes[0] = 0xff;
    bytes[1] = 0x25;
    bytes[2..6].copy_from_slice(&0u32.to_le_bytes());
    bytes[6..14].copy_from_slice(&(destination as u64).to_le_bytes());
    bytes
}
