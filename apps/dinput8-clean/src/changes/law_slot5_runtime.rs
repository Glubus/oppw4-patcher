// apps/dinput8-clean/src/changes/law_slot5_runtime.rs
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use oppw4_research::costume_table::{
    self, CostumeLayoutTableDump, CostumeTableMemory, EMPTY_LAYOUT_VARIANT_ID,
    LAW_DUPLICATE_VARIANT_ACTIVE_COUNT, LAW_DUPLICATE_VARIANT_SLOT_INDEX, LAW_MASTER_LAYOUT_ID,
};

use crate::{changes::law_slot5_model_mode, log, win};

const SLOT5_VARIANT_ID: u16 = 699;
const SOURCE_VARIANT_ID: u16 = 57;
const SOURCE_SLOT_INDEX: usize = 0;
const TARGET_MODEL_RESOURCE_ID: u16 = 730;
const TARGET_PREVIEW_MAPPING_ID: u16 = 294;
const VARIANT_METADATA_BASE_OFFSET: usize = 0xd92c;
const VARIANT_METADATA_STRIDE: usize = 0x1e;
const VARIANT_METADATA_COPY_SIZE: usize = 0x1e;
const VARIANT_METADATA_MODEL_RESOURCE_OFFSET: usize = 0x00;
const VARIANT_METADATA_PREVIEW_MAPPING_OFFSET: usize = 0x06;
const VARIANT_METADATA_FLAGS_OFFSET: usize = 0x14;
const VARIANT_METADATA_ENABLED_FLAG: u8 = 0x01;
const VARIANT_METADATA_DLC_ENTITLEMENT_FLAG: u8 = 0x02;
const CATEGORY_RUNTIME_SLOT_BASE: usize = 10;
const RUNTIME_UNLOCK_SLOT_FLAGS_OFFSET: usize = 0x160;
const RUNTIME_UNLOCK_DIRECT_FLAG: u8 = 0x01;
const RUNTIME_UNLOCK_OWNED_HINT_FLAGS: u8 = 0x0a;
const UI_SELECTED_SLOT_LIMIT_RVA: usize = 0x155eca3;
const UI_SLOT_VARIANT_LIMIT_RVA: usize = 0x1559c9f;
const UI_SLOT_VISIBILITY_MAX_SLOT_RVA: usize = 0x1559d08;
const UI_SLOT_VISIBILITY_COUNT_RVA: usize = 0x1559d11;
const LOG_LIMIT: usize = 64;

static PATCH_DONE: AtomicBool = AtomicBool::new(false);
static PATCH_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static LOGS: AtomicUsize = AtomicUsize::new(0);
static SLOT5_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn try_install(trigger: &str) {
    if PATCH_DONE.load(Ordering::Relaxed) {
        return;
    }
    let attempt = PATCH_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
    if attempt > 96 {
        return;
    }

    let mut memory = ProcessMemory;
    match costume_table::dump_costume_layout_table(win::main_module() as usize, &mut memory) {
        Ok(dump) => {
            if patch_layout_slot(&dump)
                && patch_variant_metadata(&dump)
                && patch_runtime_unlock_slot(&dump)
                && patch_law_ui_slot_bounds()
            {
                PATCH_DONE.store(true, Ordering::Relaxed);
                log_limited(format!(
                    "law-slot5-runtime-ready trigger={trigger} variant={SLOT5_VARIANT_ID} source_variant={SOURCE_VARIANT_ID} source_slot={SOURCE_SLOT_INDEX} model_mode={}",
                    law_slot5_model_mode::label()
                ));
            }
        }
        Err(error) if attempt < 8 => {
            log_limited(format!(
                "law-slot5-runtime-skip trigger={trigger} reason=dump_failed error={error:?}"
            ));
        }
        Err(_) => {}
    }
}

pub fn update_activity_from_path(path: &str) {
    let Some(file_name) = std::path::Path::new(path).file_name() else {
        return;
    };
    let file_name = file_name.to_string_lossy();
    if file_name.eq_ignore_ascii_case(&slot5_dlc_file_name()) {
        set_slot5_active(true, "dlc-request");
    } else if is_law_costume_dlc_file_name(&file_name) {
        set_slot5_active(false, "other-law-dlc-request");
    }
}

pub fn slot5_active() -> bool {
    SLOT5_ACTIVE.load(Ordering::Relaxed)
}

pub(crate) fn update_activity_from_unlock(
    category: u32,
    variant: u32,
    slot_index: u32,
    result: u64,
) {
    if category != u32::from(LAW_MASTER_LAYOUT_ID) {
        return;
    }

    if variant == u32::from(SLOT5_VARIANT_ID)
        && slot_index == LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32
        && result != 0
    {
        set_slot5_active(true, "unlock-check-slot5");
        return;
    }

    if variant == u32::from(SLOT5_VARIANT_ID) && result == 0 {
        set_slot5_active(false, "unlock-check-slot5-miss");
        return;
    }

    if result != 0 && matches!(variant as u16, 57 | 58 | 555 | 586) {
        set_slot5_active(false, "unlock-check-official-law");
    }
}

pub fn slot5_dlc_alias_path(path: &str) -> Option<String> {
    let file_name = std::path::Path::new(path).file_name()?.to_string_lossy();
    if !file_name.eq_ignore_ascii_case(&slot5_dlc_file_name()) {
        return None;
    }
    set_slot5_active(true, "dlc-alias");
    let parent = std::path::Path::new(path).parent()?;
    Some(
        parent
            .join("DLC_COSTUME_006_586_026_003.bin")
            .to_string_lossy()
            .into_owned(),
    )
}

fn patch_layout_slot(dump: &CostumeLayoutTableDump) -> bool {
    let Some(slot_address) = dump.law_duplicate_variant_address() else {
        log_limited("law-slot5-layout-patch skipped reason=slot_address_missing".to_string());
        return false;
    };
    let Some(count_address) = dump.law_duplicate_variant_active_count_address() else {
        log_limited("law-slot5-layout-patch skipped reason=count_address_missing".to_string());
        return false;
    };

    let before_slot = read_u16(slot_address).unwrap_or_default();
    if before_slot != SLOT5_VARIANT_ID && before_slot != EMPTY_LAYOUT_VARIANT_ID {
        log_limited(format!(
            "law-slot5-layout-patch skipped reason=slot4_not_empty before={before_slot}"
        ));
        return false;
    }

    let slot_ok = before_slot == SLOT5_VARIANT_ID
        || write_exact(slot_address, &SLOT5_VARIANT_ID.to_le_bytes());
    let count_before = read_u8(count_address).unwrap_or_default();
    let count_ok = count_before >= LAW_DUPLICATE_VARIANT_ACTIVE_COUNT
        || write_exact(count_address, &[LAW_DUPLICATE_VARIANT_ACTIVE_COUNT]);
    let after_slot = read_u16(slot_address).unwrap_or_default();
    let after_count = read_u8(count_address).unwrap_or_default();
    log_limited(format!(
        "law-slot5-layout-patch layout={LAW_MASTER_LAYOUT_ID} slot={} before={before_slot} after={after_slot} count_before={count_before} count_after={after_count}",
        LAW_DUPLICATE_VARIANT_SLOT_INDEX
    ));
    slot_ok
        && count_ok
        && after_slot == SLOT5_VARIANT_ID
        && after_count >= LAW_DUPLICATE_VARIANT_ACTIVE_COUNT
}

fn patch_variant_metadata(dump: &CostumeLayoutTableDump) -> bool {
    let Some(source) = variant_metadata_address(dump.static_layouts, SOURCE_VARIANT_ID) else {
        return false;
    };
    let Some(target) = variant_metadata_address(dump.static_layouts, SLOT5_VARIANT_ID) else {
        return false;
    };

    let mut source_raw = [0u8; VARIANT_METADATA_COPY_SIZE];
    if !read_exact(source, &mut source_raw) {
        log_limited("law-slot5-metadata-patch skipped reason=source_read_failed".to_string());
        return false;
    }
    source_raw[VARIANT_METADATA_FLAGS_OFFSET] = (source_raw[VARIANT_METADATA_FLAGS_OFFSET]
        | VARIANT_METADATA_ENABLED_FLAG)
        & !VARIANT_METADATA_DLC_ENTITLEMENT_FLAG;
    source_raw[VARIANT_METADATA_MODEL_RESOURCE_OFFSET..VARIANT_METADATA_MODEL_RESOURCE_OFFSET + 2]
        .copy_from_slice(&TARGET_MODEL_RESOURCE_ID.to_le_bytes());
    source_raw
        [VARIANT_METADATA_PREVIEW_MAPPING_OFFSET..VARIANT_METADATA_PREVIEW_MAPPING_OFFSET + 2]
        .copy_from_slice(&TARGET_PREVIEW_MAPPING_ID.to_le_bytes());

    let mut before = [0u8; VARIANT_METADATA_COPY_SIZE];
    let _ = read_exact(target, &mut before);
    if !write_exact(target, &source_raw) {
        log_limited("law-slot5-metadata-patch failed reason=write_failed".to_string());
        return false;
    }

    let after_model = read_u16(target + VARIANT_METADATA_MODEL_RESOURCE_OFFSET);
    let after_preview = read_u16(target + VARIANT_METADATA_PREVIEW_MAPPING_OFFSET);
    log_limited(format!(
        "law-slot5-metadata-patch source_variant={SOURCE_VARIANT_ID} target_variant={SLOT5_VARIANT_ID} before_model={} after_model={} after_preview={} model_mode={}",
        read_u16_from(&before, VARIANT_METADATA_MODEL_RESOURCE_OFFSET).unwrap_or_default(),
        after_model.unwrap_or_default(),
        after_preview.unwrap_or_default(),
        law_slot5_model_mode::label()
    ));
    after_model == read_u16_from(&source_raw, VARIANT_METADATA_MODEL_RESOURCE_OFFSET)
        && after_preview == read_u16_from(&source_raw, VARIANT_METADATA_PREVIEW_MAPPING_OFFSET)
}

fn patch_runtime_unlock_slot(dump: &CostumeLayoutTableDump) -> bool {
    let Some(runtime_root) = dump.runtime_root else {
        log_limited("law-slot5-unlock-patch skipped reason=runtime_root_missing".to_string());
        return true;
    };
    let Some(source) = runtime_slot_flag_address(runtime_root, SOURCE_SLOT_INDEX) else {
        return false;
    };
    let Some(target) = runtime_slot_flag_address(runtime_root, LAW_DUPLICATE_VARIANT_SLOT_INDEX)
    else {
        return false;
    };
    let source_value = read_u8(source).unwrap_or(0);
    let target_value =
        (source_value | RUNTIME_UNLOCK_OWNED_HINT_FLAGS) | RUNTIME_UNLOCK_DIRECT_FLAG;
    if target_value == 0 {
        return false;
    }
    let before = read_u8(target).unwrap_or(0);
    let ok = before == target_value || write_exact(target, &[target_value]);
    let after = read_u8(target).unwrap_or(0);
    log_limited(format!(
        "law-slot5-unlock-patch source_slot={SOURCE_SLOT_INDEX} target_slot={} source=0x{source_value:02x} before=0x{before:02x} after=0x{after:02x}",
        LAW_DUPLICATE_VARIANT_SLOT_INDEX
    ));
    ok && after == target_value
}

fn patch_law_ui_slot_bounds() -> bool {
    let module = win::main_module() as usize;
    let patches = [
        UiBytePatch::new(
            "selected_slot_limit",
            UI_SELECTED_SLOT_LIMIT_RVA,
            0x03,
            0x04,
        ),
        UiBytePatch::new("slot_variant_limit", UI_SLOT_VARIANT_LIMIT_RVA, 0x03, 0x04),
        UiBytePatch::new(
            "slot_visibility_max_slot",
            UI_SLOT_VISIBILITY_MAX_SLOT_RVA,
            0x03,
            0x04,
        ),
        UiBytePatch::new(
            "slot_visibility_count",
            UI_SLOT_VISIBILITY_COUNT_RVA,
            0x04,
            0x05,
        ),
    ];
    let mut all_ok = true;
    for patch in patches {
        let address = module + patch.rva;
        let before = read_u8(address).unwrap_or_default();
        let ok = if before == patch.after {
            true
        } else if before == patch.before {
            write_exact(address, &[patch.after])
        } else {
            false
        };
        let after = read_u8(address).unwrap_or_default();
        log_limited(format!(
            "law-slot5-ui-bound-patch name={} rva=game+0x{:x} before=0x{before:02x} after=0x{after:02x} ok={ok}",
            patch.name, patch.rva
        ));
        all_ok &= ok && after == patch.after;
    }
    all_ok
}

#[derive(Clone, Copy)]
struct UiBytePatch {
    name: &'static str,
    rva: usize,
    before: u8,
    after: u8,
}

impl UiBytePatch {
    const fn new(name: &'static str, rva: usize, before: u8, after: u8) -> Self {
        Self {
            name,
            rva,
            before,
            after,
        }
    }
}

fn variant_metadata_address(static_layouts: usize, variant: u16) -> Option<usize> {
    Some(
        static_layouts
            + VARIANT_METADATA_BASE_OFFSET
            + usize::from(variant) * VARIANT_METADATA_STRIDE,
    )
}

fn runtime_slot_flag_address(runtime_root: usize, slot_index: usize) -> Option<usize> {
    let pointer_address = runtime_root + (26 + CATEGORY_RUNTIME_SLOT_BASE) * 8;
    let slot_base = read_usize(pointer_address)?;
    (slot_base != 0).then_some(slot_base + RUNTIME_UNLOCK_SLOT_FLAGS_OFFSET + slot_index)
}

fn slot5_dlc_file_name() -> String {
    format!("DLC_COSTUME_006_{SLOT5_VARIANT_ID:03}_026_004.bin")
}

fn is_law_costume_dlc_file_name(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    lower.starts_with("dlc_costume_006_") && lower.contains("_026_") && lower.ends_with(".bin")
}

fn set_slot5_active(active: bool, reason: &str) {
    let before = SLOT5_ACTIVE.swap(active, Ordering::Relaxed);
    if before != active {
        log_limited(format!("law-slot5-context active={active} reason={reason}"));
    }
}

fn read_exact(address: usize, buffer: &mut [u8]) -> bool {
    if address == 0 || buffer.is_empty() {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(address as *const u8, buffer.as_mut_ptr(), buffer.len());
    }
    true
}

fn write_exact(address: usize, bytes: &[u8]) -> bool {
    matches!(win::write_process_memory(address, bytes), Some(written) if written == bytes.len())
}

fn read_u8(address: usize) -> Option<u8> {
    let mut bytes = [0u8; 1];
    read_exact(address, &mut bytes).then_some(bytes[0])
}

fn read_u16(address: usize) -> Option<u16> {
    let mut bytes = [0u8; 2];
    read_exact(address, &mut bytes).then(|| u16::from_le_bytes(bytes))
}

fn read_usize(address: usize) -> Option<usize> {
    let mut bytes = [0u8; std::mem::size_of::<usize>()];
    read_exact(address, &mut bytes).then(|| usize::from_le_bytes(bytes))
}

fn read_u16_from(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
    ]))
}

fn log_limited(message: String) {
    if LOGS.fetch_add(1, Ordering::Relaxed) < LOG_LIMIT {
        log::write_line(message);
    }
}

struct ProcessMemory;

impl CostumeTableMemory for ProcessMemory {
    fn read_exact(&mut self, address: usize, buffer: &mut [u8]) -> bool {
        read_exact(address, buffer)
    }
}
