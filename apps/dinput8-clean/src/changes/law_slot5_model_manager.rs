use std::{
    mem::size_of,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use crate::{changes::internal_hook, log, win};

const SOURCE_MODEL_RESOURCE_ID: u16 = 26;
const TARGET_MODEL_RESOURCE_ID: u16 = 730;
const MODEL_RESOURCE_MANAGER_RVA: usize = 0x1eba7a0;
const MODEL_RESOURCE_MANAGER_ENTRY_OFFSET: usize = 0x28;
const MODEL_RESOURCE_MANAGER_ENTRY_STRIDE: usize = 0x20;
const MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET: usize = 0x00;
const MODEL_RESOURCE_MANAGER_ENTRY_STATE_OFFSET: usize = 0x08;
const MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE: usize = 0x20;
const MODEL_RESOURCE_GET_RVA: usize = 0x016ce30;
const MODEL_RESOURCE_GET_STOLEN_LEN: usize = 15;
const MODEL_RESOURCE_LOADED_CHECK_RVA: usize = 0x016ceb0;
const MODEL_RESOURCE_LOADED_CHECK_STOLEN_LEN: usize = 15;
const MODEL_RESOURCE_CAN_START_LOAD_RVA: usize = 0x016cf30;
const MODEL_RESOURCE_CAN_START_LOAD_STOLEN_LEN: usize = 15;
const MODEL_RESOURCE_BUSY_CHECK_RVA: usize = 0x005ad10;
const MODEL_RESOURCE_BUSY_CHECK_STOLEN_LEN: usize = 15;
const MODEL_RESOURCE_ENQUEUE_LOAD_RVA: usize = 0x016dc20;
const MODEL_RESOURCE_ENQUEUE_LOAD_STOLEN_LEN: usize = 21;
const MODEL_RESOURCE_KEY_FROM_ID_RVA: usize = 0x138c070;
const MODEL_RESOURCE_KEY_FROM_ID_STOLEN_LEN: usize = 15;
const LOG_LIMIT: usize = 160;
const SNAPSHOT_LOG_LIMIT: usize = 80;
const KEY_LOG_LIMIT: usize = 80;

type ModelResourceGetFn = unsafe extern "system" fn(usize, u32) -> usize;
type ModelResourceCheckFn = unsafe extern "system" fn(usize, u32) -> u64;
type ModelResourceEnqueueLoadFn = unsafe extern "system" fn(usize, u32, usize, usize);
type ModelResourceKeyFromIdFn = unsafe extern "system" fn(usize) -> usize;

static INSTALLED: AtomicBool = AtomicBool::new(false);
static MODEL_RESOURCE_GET_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_LOADED_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_CAN_START_LOAD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_BUSY_CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_ENQUEUE_LOAD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MODEL_RESOURCE_KEY_FROM_ID_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static LOGS: AtomicUsize = AtomicUsize::new(0);
static SNAPSHOT_LOGS: AtomicUsize = AtomicUsize::new(0);
static KEY_LOGS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
struct ModelResourceSlotSnapshot {
    source_pointer: usize,
    source_state: u32,
    target_pointer: usize,
    target_state: u32,
}

pub(crate) fn install() {
    if INSTALLED.swap(true, Ordering::AcqRel) {
        return;
    }

    let module = win::main_module() as usize;
    if module == 0 {
        log::write_line("law-slot5-model-manager install skipped reason=main_module_missing");
        return;
    }

    let mut installed = 0;
    unsafe {
        installed += internal_hook::install_absolute_jump_hook(
            module,
            MODEL_RESOURCE_GET_RVA,
            MODEL_RESOURCE_GET_STOLEN_LEN,
            hooked_model_resource_get as usize,
            &MODEL_RESOURCE_GET_ORIGINAL,
            "law-slot5 model-resource-get alias",
        ) as usize;
        installed += internal_hook::install_absolute_jump_hook(
            module,
            MODEL_RESOURCE_LOADED_CHECK_RVA,
            MODEL_RESOURCE_LOADED_CHECK_STOLEN_LEN,
            hooked_model_resource_loaded_check as usize,
            &MODEL_RESOURCE_LOADED_CHECK_ORIGINAL,
            "law-slot5 model-resource-loaded alias",
        ) as usize;
        installed += internal_hook::install_absolute_jump_hook(
            module,
            MODEL_RESOURCE_CAN_START_LOAD_RVA,
            MODEL_RESOURCE_CAN_START_LOAD_STOLEN_LEN,
            hooked_model_resource_can_start_load as usize,
            &MODEL_RESOURCE_CAN_START_LOAD_ORIGINAL,
            "law-slot5 model-resource-can-start alias",
        ) as usize;
        installed += internal_hook::install_absolute_jump_hook(
            module,
            MODEL_RESOURCE_BUSY_CHECK_RVA,
            MODEL_RESOURCE_BUSY_CHECK_STOLEN_LEN,
            hooked_model_resource_busy_check as usize,
            &MODEL_RESOURCE_BUSY_CHECK_ORIGINAL,
            "law-slot5 model-resource-busy alias",
        ) as usize;
        installed += internal_hook::install_absolute_jump_hook(
            module,
            MODEL_RESOURCE_ENQUEUE_LOAD_RVA,
            MODEL_RESOURCE_ENQUEUE_LOAD_STOLEN_LEN,
            hooked_model_resource_enqueue_load as usize,
            &MODEL_RESOURCE_ENQUEUE_LOAD_ORIGINAL,
            "law-slot5 model-resource-enqueue alias",
        ) as usize;
        installed += internal_hook::install_absolute_jump_hook(
            module,
            MODEL_RESOURCE_KEY_FROM_ID_RVA,
            MODEL_RESOURCE_KEY_FROM_ID_STOLEN_LEN,
            hooked_model_resource_key_from_id as usize,
            &MODEL_RESOURCE_KEY_FROM_ID_ORIGINAL,
            "law-slot5 model-resource-key diagnostic",
        ) as usize;
    }

    log::write_line(format!(
        "law-slot5-model-manager hooks installed={installed} source={SOURCE_MODEL_RESOURCE_ID} target={TARGET_MODEL_RESOURCE_ID}"
    ));
}

unsafe extern "system" fn hooked_model_resource_get(manager: usize, resource_id: u32) -> usize {
    let Some(original) = original_get() else {
        return 0;
    };
    let snapshot = snapshot_target_if_needed(manager, resource_id);
    let result = original(manager, resource_id);
    log_alias(
        "get",
        manager,
        resource_id,
        format!("result=0x{result:x}"),
        snapshot,
    );
    result
}

unsafe extern "system" fn hooked_model_resource_loaded_check(
    manager: usize,
    resource_id: u32,
) -> u64 {
    let Some(original) = original_loaded_check() else {
        return 0;
    };
    let snapshot = snapshot_target_if_needed(manager, resource_id);
    let result = original(manager, resource_id);
    log_alias(
        "loaded-check",
        manager,
        resource_id,
        format!("result={result}"),
        snapshot,
    );
    result
}

unsafe extern "system" fn hooked_model_resource_can_start_load(
    manager: usize,
    resource_id: u32,
) -> u64 {
    let Some(original) = original_can_start_load() else {
        return 0;
    };
    let snapshot = snapshot_target_if_needed(manager, resource_id);
    let result = original(manager, resource_id);
    log_alias(
        "can-start-load",
        manager,
        resource_id,
        format!("result={result}"),
        snapshot,
    );
    result
}

unsafe extern "system" fn hooked_model_resource_busy_check(
    manager: usize,
    resource_id: u32,
) -> u64 {
    let Some(original) = original_busy_check() else {
        return 0;
    };
    let snapshot = snapshot_target_if_needed(manager, resource_id);
    let result = original(manager, resource_id);
    log_alias(
        "busy-check",
        manager,
        resource_id,
        format!("result={result}"),
        snapshot,
    );
    result
}

unsafe extern "system" fn hooked_model_resource_enqueue_load(
    manager: usize,
    resource_id: u32,
    task: usize,
    extra: usize,
) {
    let Some(original) = original_enqueue_load() else {
        return;
    };
    let snapshot = snapshot_target_if_needed(manager, resource_id);
    log_alias(
        "enqueue-load",
        manager,
        resource_id,
        format!("task=0x{task:x} extra=0x{extra:x}"),
        snapshot,
    );
    original(manager, resource_id, task, extra);
}

unsafe extern "system" fn hooked_model_resource_key_from_id(task_resource: usize) -> usize {
    let Some(original) = original_key_from_id() else {
        return 0;
    };
    let resource_id = read_u32(task_resource.saturating_add(8)).unwrap_or(u32::MAX);
    let result = original(task_resource);
    if is_interesting_resource(resource_id) {
        log_key_from_id(task_resource, resource_id, result);
    }
    result
}

fn snapshot_target_if_needed(
    manager: usize,
    resource_id: u32,
) -> Option<ModelResourceSlotSnapshot> {
    if resource_id != u32::from(TARGET_MODEL_RESOURCE_ID) || !is_model_resource_manager(manager) {
        return None;
    }
    snapshot_model_resource_manager_entries(
        manager,
        SOURCE_MODEL_RESOURCE_ID,
        TARGET_MODEL_RESOURCE_ID,
    )
}

fn is_model_resource_manager(manager: usize) -> bool {
    manager != 0 && read_game_global_pointer(MODEL_RESOURCE_MANAGER_RVA) == Some(manager)
}

fn snapshot_model_resource_manager_entries(
    manager: usize,
    source_id: u16,
    target_id: u16,
) -> Option<ModelResourceSlotSnapshot> {
    let source_address = model_resource_manager_entry_address(manager, u32::from(source_id))?;
    let target_address = model_resource_manager_entry_address(manager, u32::from(target_id))?;
    let mut source = [0u8; MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE];
    if !read_exact(source_address, &mut source) {
        log_snapshot(format!(
            "failed source_id={source_id} target_id={target_id} source=0x{source_address:x} target=0x{target_address:x} error=read_source_failed"
        ));
        return None;
    }
    let mut target = [0u8; MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE];
    if !read_exact(target_address, &mut target) {
        log_snapshot(format!(
            "failed source_id={source_id} target_id={target_id} source=0x{source_address:x} target=0x{target_address:x} error=read_target_failed"
        ));
        return None;
    }

    let snapshot = ModelResourceSlotSnapshot {
        source_pointer: model_resource_manager_entry_pointer(&source),
        source_state: model_resource_manager_entry_state(&source),
        target_pointer: model_resource_manager_entry_pointer(&target),
        target_state: model_resource_manager_entry_state(&target),
    };
    log_snapshot(format!(
        "source_id={source_id} target_id={target_id} source=0x{source_address:x} target=0x{target_address:x} {}",
        format_snapshot(Some(snapshot))
    ));
    Some(snapshot)
}

fn model_resource_manager_entry_address(manager: usize, resource_id: u32) -> Option<usize> {
    let resource_id = usize::try_from(resource_id).ok()?;
    manager
        .checked_add(MODEL_RESOURCE_MANAGER_ENTRY_OFFSET)?
        .checked_add(resource_id.checked_mul(MODEL_RESOURCE_MANAGER_ENTRY_STRIDE)?)
}

fn model_resource_manager_entry_pointer(
    bytes: &[u8; MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE],
) -> usize {
    let mut value = [0u8; size_of::<usize>()];
    value.copy_from_slice(
        &bytes[MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET
            ..MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET + size_of::<usize>()],
    );
    usize::from_le_bytes(value)
}

fn model_resource_manager_entry_state(bytes: &[u8; MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE]) -> u32 {
    let mut value = [0u8; size_of::<u32>()];
    value.copy_from_slice(
        &bytes[MODEL_RESOURCE_MANAGER_ENTRY_STATE_OFFSET
            ..MODEL_RESOURCE_MANAGER_ENTRY_STATE_OFFSET + size_of::<u32>()],
    );
    u32::from_le_bytes(value)
}

fn read_game_global_pointer(rva: usize) -> Option<usize> {
    let address = (win::main_module() as usize).checked_add(rva)?;
    read_usize(address)
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

fn read_usize(address: usize) -> Option<usize> {
    let mut bytes = [0u8; size_of::<usize>()];
    read_exact(address, &mut bytes).then(|| usize::from_le_bytes(bytes))
}

fn read_u32(address: usize) -> Option<u32> {
    let mut bytes = [0u8; size_of::<u32>()];
    read_exact(address, &mut bytes).then(|| u32::from_le_bytes(bytes))
}

fn original_get() -> Option<ModelResourceGetFn> {
    let address = MODEL_RESOURCE_GET_ORIGINAL.load(Ordering::Acquire);
    (address != 0).then(|| unsafe { std::mem::transmute(address) })
}

fn original_loaded_check() -> Option<ModelResourceCheckFn> {
    let address = MODEL_RESOURCE_LOADED_CHECK_ORIGINAL.load(Ordering::Acquire);
    (address != 0).then(|| unsafe { std::mem::transmute(address) })
}

fn original_can_start_load() -> Option<ModelResourceCheckFn> {
    let address = MODEL_RESOURCE_CAN_START_LOAD_ORIGINAL.load(Ordering::Acquire);
    (address != 0).then(|| unsafe { std::mem::transmute(address) })
}

fn original_busy_check() -> Option<ModelResourceCheckFn> {
    let address = MODEL_RESOURCE_BUSY_CHECK_ORIGINAL.load(Ordering::Acquire);
    (address != 0).then(|| unsafe { std::mem::transmute(address) })
}

fn original_enqueue_load() -> Option<ModelResourceEnqueueLoadFn> {
    let address = MODEL_RESOURCE_ENQUEUE_LOAD_ORIGINAL.load(Ordering::Acquire);
    (address != 0).then(|| unsafe { std::mem::transmute(address) })
}

fn original_key_from_id() -> Option<ModelResourceKeyFromIdFn> {
    let address = MODEL_RESOURCE_KEY_FROM_ID_ORIGINAL.load(Ordering::Acquire);
    (address != 0).then(|| unsafe { std::mem::transmute(address) })
}

fn log_alias(
    phase: &str,
    manager: usize,
    resource_id: u32,
    detail: String,
    snapshot: Option<ModelResourceSlotSnapshot>,
) {
    if !is_interesting_resource(resource_id) {
        return;
    }
    let index = LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= LOG_LIMIT {
        return;
    }
    log::write_line(format!(
        "law-slot5-model-manager phase={phase} manager=0x{manager:x} resource={resource_id} is_model_manager={} {detail} {}",
        is_model_resource_manager(manager),
        format_snapshot(snapshot)
    ));
}

fn log_snapshot(message: String) {
    let index = SNAPSHOT_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < SNAPSHOT_LOG_LIMIT {
        log::write_line(format!("law-slot5-model-manager snapshot {message}"));
    }
}

fn log_key_from_id(task_resource: usize, resource_id: u32, result: usize) {
    let index = KEY_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < KEY_LOG_LIMIT {
        log::write_line(format!(
            "law-slot5-model-key resource={resource_id} task_resource=0x{task_resource:x} result=0x{result:x}"
        ));
    }
}

fn is_interesting_resource(resource_id: u32) -> bool {
    resource_id == u32::from(TARGET_MODEL_RESOURCE_ID)
        || resource_id == u32::from(SOURCE_MODEL_RESOURCE_ID)
}

fn format_snapshot(snapshot: Option<ModelResourceSlotSnapshot>) -> String {
    let Some(snapshot) = snapshot else {
        return "snapshot=none".to_string();
    };
    format!(
        "snapshot=source(ptr=0x{:x},state={}) target(ptr=0x{:x},state={})",
        snapshot.source_pointer,
        snapshot.source_state,
        snapshot.target_pointer,
        snapshot.target_state
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_resource_manager_entry_address_uses_game_stride() {
        assert_eq!(
            model_resource_manager_entry_address(0x1000, 26),
            Some(
                0x1000
                    + MODEL_RESOURCE_MANAGER_ENTRY_OFFSET
                    + 26 * MODEL_RESOURCE_MANAGER_ENTRY_STRIDE
            )
        );
        assert_eq!(MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE, 0x20);
    }
}
