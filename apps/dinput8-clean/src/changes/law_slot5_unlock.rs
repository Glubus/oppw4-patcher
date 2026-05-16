// apps/dinput8-clean/src/changes/law_slot5_unlock.rs
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Once,
};

use crate::{
    changes::{internal_hook, law_slot5_runtime},
    log, win,
};

const COSTUME_VARIANT_UNLOCK_CHECK_RVA: usize = 0x12f52a0;
const COSTUME_VARIANT_UNLOCK_CHECK_STOLEN_LEN: usize = 16;
const LAW_CATEGORY_ID: u32 = 26;
const LAW_HIDDEN_VARIANT_ID: u32 = 555;
const LAW_SLOT5_VARIANT_ID: u32 = 699;
const LAW_SLOT5_INDEX: u32 = 4;
const LOG_LIMIT: usize = 48;

type UnlockCheckFn = unsafe extern "system" fn(u32, u32, u32, i32) -> u64;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static LOGS: AtomicUsize = AtomicUsize::new(0);
static INSTALL_ONCE: Once = Once::new();

pub(crate) fn install() {
    INSTALL_ONCE.call_once(|| {
        let module = win::main_module() as usize;
        if module == 0 {
            log::write_line("law-slot5 unlock hook skipped reason=main_module_missing");
            return;
        }
        unsafe {
            internal_hook::install_absolute_jump_hook(
                module,
                COSTUME_VARIANT_UNLOCK_CHECK_RVA,
                COSTUME_VARIANT_UNLOCK_CHECK_STOLEN_LEN,
                hooked_unlock_check as usize,
                &ORIGINAL,
                "law-slot5 unlock-check",
            );
        }
    });
}

unsafe extern "system" fn hooked_unlock_check(
    category: u32,
    variant: u32,
    slot_index: u32,
    strict: i32,
) -> u64 {
    let original_address = ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: UnlockCheckFn = std::mem::transmute(original_address);
    let original_result = original(category, variant, slot_index, strict);
    let result =
        unlock_override(category, variant, slot_index, original_result).unwrap_or(original_result);
    law_slot5_runtime::update_activity_from_unlock(category, variant, slot_index, result);
    log_unlock_check(
        category,
        variant,
        slot_index,
        strict,
        original_result,
        result,
    );
    result
}

fn unlock_override(category: u32, variant: u32, slot_index: u32, result: u64) -> Option<u64> {
    if result != 0 || category != LAW_CATEGORY_ID {
        return None;
    }
    if variant == LAW_HIDDEN_VARIANT_ID {
        return Some(1);
    }
    if variant == LAW_SLOT5_VARIANT_ID && slot_index == LAW_SLOT5_INDEX {
        return Some(1);
    }
    None
}

fn log_unlock_check(
    category: u32,
    variant: u32,
    slot_index: u32,
    strict: i32,
    original_result: u64,
    result: u64,
) {
    let relevant = category == LAW_CATEGORY_ID
        && (variant == LAW_HIDDEN_VARIANT_ID || variant == LAW_SLOT5_VARIANT_ID);
    if !relevant {
        return;
    }
    let index = LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= LOG_LIMIT {
        return;
    }
    log::write_line(format!(
        "law-slot5-unlock-check category={category} variant={variant} slot={slot_index} strict={strict} original_result={original_result} result={result} forced={}",
        original_result != result
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlock_override_is_law_only() {
        assert_eq!(unlock_override(26, 555, 2, 0), Some(1));
        assert_eq!(unlock_override(26, 699, 4, 0), Some(1));
        assert_eq!(unlock_override(26, 699, 3, 0), None);
        assert_eq!(unlock_override(25, 699, 4, 0), None);
        assert_eq!(unlock_override(26, 699, 4, 1), None);
    }
}
