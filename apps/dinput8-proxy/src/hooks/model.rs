unsafe extern "system" fn hooked_model_resource_get(manager: usize, resource_id: u32) -> usize {
    let original_address = MODEL_RESOURCE_GET_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: ModelResourceGetFn = std::mem::transmute(original_address);
    let mapped_resource_id =
        law_private_model_manager_resource_id_for_manager(manager, resource_id);
    let mirror = mirror_law_private_model_manager_slot_for_request(manager, resource_id);
    let call_resource_id =
        law_private_model_manager_call_resource_id(resource_id, mapped_resource_id, mirror);
    let result = original(manager, call_resource_id);
    remember_law_private_model_resource_pointer(resource_id, result, mirror);
    log_law_private_model_manager_alias(
        "get",
        manager,
        resource_id,
        mapped_resource_id,
        format!(
            "call={} result=0x{result:x} {}",
            call_resource_id,
            format_model_resource_slot_mirror_detail(mirror)
        ),
    );
    result
}

unsafe extern "system" fn hooked_model_resource_loaded_check(
    manager: usize,
    resource_id: u32,
) -> u64 {
    let original_address = MODEL_RESOURCE_LOADED_CHECK_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: ModelResourceCheckFn = std::mem::transmute(original_address);
    let mapped_resource_id =
        law_private_model_manager_resource_id_for_manager(manager, resource_id);
    let mirror = mirror_law_private_model_manager_slot_for_request(manager, resource_id);
    let call_resource_id =
        law_private_model_manager_call_resource_id(resource_id, mapped_resource_id, mirror);
    let result = original(manager, call_resource_id);
    remember_law_private_model_resource_pointer(resource_id, 0, mirror);
    log_law_private_model_manager_alias(
        "loaded-check",
        manager,
        resource_id,
        mapped_resource_id,
        format!(
            "call={} result={result} {}",
            call_resource_id,
            format_model_resource_slot_mirror_detail(mirror)
        ),
    );
    result
}

unsafe extern "system" fn hooked_model_resource_can_start_load(
    manager: usize,
    resource_id: u32,
) -> u64 {
    let original_address = MODEL_RESOURCE_CAN_START_LOAD_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: ModelResourceCheckFn = std::mem::transmute(original_address);
    let mapped_resource_id =
        law_private_model_manager_resource_id_for_manager(manager, resource_id);
    let mirror = mirror_law_private_model_manager_slot_for_request(manager, resource_id);
    let call_resource_id =
        law_private_model_manager_call_resource_id(resource_id, mapped_resource_id, mirror);
    let result = original(manager, call_resource_id);
    let (effective_result, forced_can_start) = law_private_model_can_start_effective_result(
        resource_id,
        mapped_resource_id,
        mirror,
        result,
    );
    remember_law_private_model_resource_pointer(resource_id, 0, mirror);
    log_law_private_model_manager_alias(
        "can-start-load",
        manager,
        resource_id,
        mapped_resource_id,
        format!(
            "call={} original_result={result} result={effective_result} forced_can_start={} {}",
            call_resource_id,
            forced_can_start,
            format_model_resource_slot_mirror_detail(mirror)
        ),
    );
    if resource_id == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID) {
        log_law_ready_timeline(
            "can-start",
            format!(
                "slot_label=slot5-custom manager=0x{manager:x} requested={resource_id} mapped={mapped_resource_id} call={call_resource_id} original_result={result} result={effective_result} forced_can_start={} {}",
                forced_can_start,
                format_model_resource_slot_mirror_detail(mirror)
            ),
        );
    }
    effective_result
}

unsafe extern "system" fn hooked_model_resource_busy_check(
    manager: usize,
    resource_id: u32,
) -> u64 {
    let original_address = MODEL_RESOURCE_BUSY_CHECK_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: ModelResourceCheckFn = std::mem::transmute(original_address);
    let mapped_resource_id =
        law_private_model_manager_resource_id_for_manager(manager, resource_id);
    let mirror = mirror_law_private_model_manager_slot_for_request(manager, resource_id);
    let call_resource_id =
        law_private_model_manager_call_resource_id(resource_id, mapped_resource_id, mirror);
    let result = original(manager, call_resource_id);
    remember_law_private_model_resource_pointer(resource_id, 0, mirror);
    log_law_private_model_manager_alias(
        "busy-check",
        manager,
        resource_id,
        mapped_resource_id,
        format!(
            "call={} result={result} {}",
            call_resource_id,
            format_model_resource_slot_mirror_detail(mirror)
        ),
    );
    result
}

unsafe extern "system" fn hooked_model_resource_enqueue_load(
    manager: usize,
    resource_id: u32,
    task: usize,
    extra: usize,
) {
    let original_address = MODEL_RESOURCE_ENQUEUE_LOAD_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let original: ModelResourceEnqueueLoadFn = std::mem::transmute(original_address);
    let mapped_resource_id =
        law_private_model_manager_resource_id_for_manager(manager, resource_id);
    let task_restore = if mapped_resource_id != resource_id {
        patch_model_resource_task_id(task, resource_id, mapped_resource_id)
    } else {
        None
    };
    log_law_private_model_manager_alias(
        "enqueue-load",
        manager,
        resource_id,
        mapped_resource_id,
        format!("task=0x{task:x} task_patched={}", task_restore.is_some()),
    );
    original(manager, mapped_resource_id, task, extra);
    let mirror = mirror_law_private_model_manager_slot_for_request(manager, resource_id);
    remember_law_private_model_resource_pointer(resource_id, 0, mirror);
    if mirror.is_some() {
        log_law_private_model_manager_alias(
            "enqueue-load-mirror",
            manager,
            resource_id,
            mapped_resource_id,
            format_model_resource_slot_mirror_detail(mirror),
        );
    }
    if let Some(previous) = task_restore {
        restore_model_resource_task_id(task, previous);
    }
}

unsafe extern "system" fn hooked_model_resource_status_check(
    manager: usize,
    resource_id: u32,
) -> u64 {
    let original_address = MODEL_RESOURCE_STATUS_CHECK_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let interesting = is_interesting_model_resource_status_check(resource_id);
    let before = if interesting {
        Some(format_model_resource_manager_entry(manager, resource_id))
    } else {
        None
    };
    let original: ModelResourceCheckFn = std::mem::transmute(original_address);
    let result = original(manager, resource_id);
    if interesting {
        log_model_resource_status_check(
            manager,
            resource_id,
            result,
            before,
            format_model_resource_manager_entry(manager, resource_id),
        );
    }
    result
}

unsafe extern "system" fn hooked_model_load_state_step(object: usize) -> u64 {
    let original_address = MODEL_LOAD_STATE_STEP_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let load_model = read_u32_field(object, 0x18);
    let interesting = load_model.is_some_and(is_interesting_model_ready_resource);
    let before = interesting.then(|| format_model_load_state_step_trace(object, load_model));
    let before_timeline = read_model_load_timeline_state(object);
    let original: ModelLoadStateStepFn = std::mem::transmute(original_address);
    let result = original(object);
    let after_timeline = read_model_load_timeline_state(object);
    if interesting {
        log_model_load_state_step(
            object,
            result,
            before,
            format_model_load_state_step_trace(object, load_model),
        );
    }
    log_law_ready_timeline_model_load_state_step(object, result, before_timeline, after_timeline);
    result
}

unsafe extern "system" fn hooked_model_ready_wait_check(object: usize) -> u64 {
    let original_address = MODEL_READY_WAIT_CHECK_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let interesting = is_interesting_model_ready_wait_check(object);
    let before = interesting.then(|| format_model_ready_wait_check_object(object));
    let original: ModelReadyWaitCheckFn = std::mem::transmute(original_address);
    let result = original(object);
    if interesting {
        log_model_ready_wait_check(
            object,
            result,
            before,
            format_model_ready_wait_check_object(object),
        );
    }
    result
}

unsafe extern "system" fn hooked_model_render_attach(
    object: usize,
    variant_context: usize,
    model_object: usize,
    attach_state: usize,
    flags: u32,
) -> usize {
    let original_address = MODEL_RENDER_ATTACH_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: ModelRenderAttachFn = std::mem::transmute(original_address);
    let last_private_resource = LAST_LAW_PRIVATE_MODEL_RESOURCE_POINTER.load(Ordering::Relaxed);
    let global_log_index = if last_private_resource == 0 {
        Some(MODEL_RENDER_ATTACH_GLOBAL_LOGS.fetch_add(1, Ordering::Relaxed))
    } else {
        None
    };
    let private_log_index = if last_private_resource != 0 {
        Some(MODEL_RENDER_ATTACH_LOGS.fetch_add(1, Ordering::Relaxed))
    } else {
        None
    };
    let log_scope =
        model_render_attach_log_scope(last_private_resource, global_log_index, private_log_index);
    let model_object_payload = read_usize_field(model_object, 0x20);
    let model_object_nested = read_usize_field(model_object_payload.unwrap_or(0), 0x30);
    let attach_first = read_usize_field(attach_state, 0x00);
    let result = original(object, variant_context, model_object, attach_state, flags);
    if let Some((label, trace)) = last_law_ready_timeline_trace_label() {
        let frames = capture_stack_trace();
        log_law_ready_timeline(
            "render-attach",
            format!(
                "slot_label={label} object=0x{object:x} variant_context=0x{variant_context:x} model_object=0x{model_object:x} model_payload={} model_nested={} attach_state=0x{attach_state:x} attach_first={} flags=0x{flags:x} result=0x{result:x} last_private_resource=0x{last_private_resource:x} context=[{}] frames={}",
                format_optional_address(model_object_payload),
                format_optional_address(model_object_nested),
                format_optional_address(attach_first),
                format_costume_object_update_trace(trace),
                format_stack_frames(&frames, 6)
            ),
        );
        if let Some(render_label) = preview_render_attach_label(trace) {
            let index = MODEL_RENDER_ATTACH_SLOT_CONTEXT_LOGS.fetch_add(1, Ordering::Relaxed);
            if index < MAX_MODEL_RENDER_ATTACH_SLOT_CONTEXT_LOGS {
                log::write_line(format!(
                    "{render_label} object=0x{object:x} variant_context=0x{variant_context:x} model_object=0x{model_object:x} model_payload={} model_nested={} attach_state=0x{attach_state:x} attach_first={} flags=0x{flags:x} result=0x{result:x} last_private_resource=0x{last_private_resource:x} render_probe=[{}] context=[{}] frames={}",
                    format_optional_address(model_object_payload),
                    format_optional_address(model_object_nested),
                    format_optional_address(attach_first),
                    format_preview_render_attach_probe(trace),
                    format_costume_object_update_trace(trace),
                    format_stack_frames(&frames, 7)
                ));
            }
        }
    }
    if let Some(scope) = log_scope {
        let frames = capture_stack_trace();
        log::write_line(format!(
            "Model render attach trace scope={} object=0x{object:x} variant_context=0x{variant_context:x} model_object=0x{model_object:x} model_payload={} model_nested={} attach_state=0x{attach_state:x} attach_first={} flags=0x{flags:x} result=0x{result:x} last_private_resource=0x{last_private_resource:x} frames={}",
            model_render_attach_log_scope_label(scope),
            format_optional_address(model_object_payload),
            format_optional_address(model_object_nested),
            format_optional_address(attach_first),
            format_stack_frames(&frames, 6)
        ));
    }
    result
}

unsafe extern "system" fn hooked_model_color_apply(state: usize, color_id: u32) {
    let original_address = MODEL_COLOR_APPLY_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let model_id = read_u32_field(state, 0x30);
    let interesting = is_interesting_model_color_apply(model_id, color_id);
    let before = interesting.then(|| format_model_color_apply_state(state));
    let original: ModelColorApplyFn = std::mem::transmute(original_address);
    original(state, color_id);
    if !interesting {
        return;
    }
    let index = MODEL_COLOR_APPLY_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_MODEL_COLOR_APPLY_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    let after_model_id = read_u32_field(state, 0x30);
    let asset_label = preview_asset_state_label(after_model_id.or(model_id)).unwrap_or("other");
    log::write_line(format!(
        "Model color apply trace label={asset_label} state=0x{state:x} color_arg={} before=[{}] after=[{}] frames={}",
        color_id,
        before.unwrap_or_else(|| "none".to_string()),
        format_model_color_apply_state(state),
        format_stack_frames(&frames, 8)
    ));
    log_preview_asset_state(asset_label, state, color_id, &frames);
}

fn is_interesting_model_color_apply(model_id: Option<u32>, color_id: u32) -> bool {
    model_id.is_some_and(is_interesting_model_ready_resource) || color_id == u32::from(u16::MAX)
}

fn format_model_color_apply_state(state: usize) -> String {
    format!(
        "ptr08={} ptr10={} ptr18={} model_object20={} field28={} field2c={} model30={} color34={} field38={} field3c={} ptr40={} ptr48={} ptr50={} ptr58={}",
        format_optional_address(read_usize_field(state, 0x08).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(state, 0x10).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(state, 0x18).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(state, 0x20).filter(|address| *address != 0)),
        format_optional_u32(read_u32_field(state, 0x28)),
        format_optional_u32(read_u32_field(state, 0x2c)),
        format_optional_u32(read_u32_field(state, 0x30)),
        format_optional_u16_decimal(read_u16_field(state, 0x34)),
        format_optional_u32(read_u32_field(state, 0x38)),
        format_optional_u32(read_u32_field(state, 0x3c)),
        format_optional_address(read_usize_field(state, 0x40).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(state, 0x48).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(state, 0x50).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(state, 0x58).filter(|address| *address != 0))
    )
}

fn preview_asset_state_label(model_id: Option<u32>) -> Option<&'static str> {
    match model_id {
        Some(308) => Some("oni"),
        Some(model) if model == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID) => {
            Some("slot5")
        }
        Some(model) if model == u32::from(LAW_MASTER_LAYOUT_ID) => Some("law-base"),
        _ => None,
    }
}

fn log_preview_asset_state(label: &str, state: usize, color_arg: u32, frames: &[usize]) {
    if !matches!(label, "oni" | "slot5" | "law-base") {
        return;
    }
    let index = PREVIEW_ASSET_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_PREVIEW_ASSET_STATE_LOGS {
        return;
    }

    let model_object = read_usize_field(state, 0x20).filter(|address| *address != 0);
    let ptr50 = read_usize_field(state, 0x50).filter(|address| *address != 0);
    let ptr58 = read_usize_field(state, 0x58).filter(|address| *address != 0);
    let model_id = read_u32_field(state, 0x30);
    let color_id = read_u16_field(state, 0x34);
    let loaded_hint = model_object.is_some();
    let attached_hint = ptr50.is_some() || ptr58.is_some();

    log::write_line(format!(
        "preview-asset-state label={label} state=0x{state:x} model={} color_arg={color_arg} material={} loaded_hint={loaded_hint} attached_hint={attached_hint} model_object={} field38={} field3c={} ptr50={} ptr58={} resource911_live=[{}] frames={}",
        format_optional_u32(model_id),
        format_optional_u16_decimal(color_id),
        format_optional_address(model_object),
        format_optional_u32(read_u32_field(state, 0x38)),
        format_optional_u32(read_u32_field(state, 0x3c)),
        format_optional_address(ptr50),
        format_optional_address(ptr58),
        format_preview_resource_911_live_state(),
        format_stack_frames(frames, 8)
    ));
    log_preview_render_asset_state(label, state, color_arg, frames);
}

fn format_preview_render_asset_pointer_candidate(pointer: Option<usize>) -> String {
    let Some(pointer) = pointer else {
        return "none".to_string();
    };

    format!(
        "ptr=0x{pointer:x} q00={} q08={} q10={} q18={} model30={} color34={} field38={} field3c={} ptr50={} ptr58={}",
        format_optional_address(read_usize_field(pointer, 0x00).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(pointer, 0x08).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(pointer, 0x10).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(pointer, 0x18).filter(|address| *address != 0)),
        format_optional_u32(read_u32_field(pointer, 0x30)),
        format_optional_u16_decimal(read_u16_field(pointer, 0x34)),
        format_optional_u32(read_u32_field(pointer, 0x38)),
        format_optional_u32(read_u32_field(pointer, 0x3c)),
        format_optional_address(read_usize_field(pointer, 0x50).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(pointer, 0x58).filter(|address| *address != 0))
    )
}

fn log_preview_render_asset_state(label: &str, state: usize, color_arg: u32, frames: &[usize]) {
    if !matches!(label, "oni" | "slot5" | "law-base") {
        return;
    }
    let index = PREVIEW_RENDER_ASSET_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_PREVIEW_RENDER_ASSET_LOGS {
        return;
    }

    let model_object = read_usize_field(state, 0x20).filter(|address| *address != 0);
    let material_candidate = read_usize_field(state, 0x50).filter(|address| *address != 0);
    let texture_candidate = read_usize_field(state, 0x58).filter(|address| *address != 0);
    let model_id = read_u32_field(state, 0x30);
    let material_id = read_u16_field(state, 0x34);
    let loaded = model_object.is_some();
    let attached = material_candidate.is_some() || texture_candidate.is_some();

    log::write_line(format!(
        "preview-render-asset label={label} state=0x{state:x} model={} color_arg={color_arg} material={} loaded={loaded} attached={attached} instance={} model_object={} material_candidate={} texture_candidate={} field38={} field3c={} resource911_live=[{}] frames={}",
        format_optional_u32(model_id),
        format_optional_u16_decimal(material_id),
        format_preview_render_asset_pointer_candidate(Some(state)),
        format_preview_render_asset_pointer_candidate(model_object),
        format_preview_render_asset_pointer_candidate(material_candidate),
        format_preview_render_asset_pointer_candidate(texture_candidate),
        format_optional_u32(read_u32_field(state, 0x38)),
        format_optional_u32(read_u32_field(state, 0x3c)),
        format_preview_resource_911_live_state(),
        format_stack_frames(frames, 8)
    ));
}

fn model_render_attach_log_scope(
    last_private_resource: usize,
    global_log_index: Option<usize>,
    private_log_index: Option<usize>,
) -> Option<ModelRenderAttachLogScope> {
    if last_private_resource != 0 {
        return private_log_index
            .is_some_and(|index| index < MAX_MODEL_RENDER_ATTACH_LOGS)
            .then_some(ModelRenderAttachLogScope::Private);
    }
    global_log_index
        .is_some_and(|index| index < MAX_MODEL_RENDER_ATTACH_GLOBAL_LOGS)
        .then_some(ModelRenderAttachLogScope::Global)
}

fn model_render_attach_log_scope_label(scope: ModelRenderAttachLogScope) -> &'static str {
    match scope {
        ModelRenderAttachLogScope::Global => "global",
        ModelRenderAttachLogScope::Private => "private",
    }
}

fn remember_law_private_model_resource_pointer(
    requested_resource_id: u32,
    result: usize,
    mirror: Option<ModelResourceSlotMirror>,
) {
    if let Some(pointer) =
        law_private_model_pointer_to_remember(requested_resource_id, result, mirror)
    {
        LAST_LAW_PRIVATE_MODEL_RESOURCE_POINTER.store(pointer, Ordering::Relaxed);
    }
}

fn law_private_model_pointer_to_remember(
    requested_resource_id: u32,
    result: usize,
    mirror: Option<ModelResourceSlotMirror>,
) -> Option<usize> {
    if requested_resource_id != u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID) {
        return None;
    }
    if result != 0 {
        return Some(result);
    }
    mirror
        .map(|mirror| mirror.after_pointer)
        .filter(|pointer| *pointer != 0)
}

fn law_private_model_manager_alias_ready() -> bool {
    LAW_EXTRA_SLOT_PRIVATE_MODEL_MANAGER_ALIAS_ENABLED
        && LAW_PRIVATE_MODEL_MANAGER_ALIAS_HOOK_ACTIVE.load(Ordering::Relaxed)
}

fn law_private_model_manager_resource_id_for_manager(manager: usize, resource_id: u32) -> u32 {
    if is_model_resource_manager(manager) {
        law_private_model_manager_resource_id(resource_id)
    } else {
        resource_id
    }
}

fn law_private_model_manager_resource_id(resource_id: u32) -> u32 {
    if LAW_EXTRA_SLOT_PRIVATE_MODEL_MANAGER_ALIAS_ENABLED
        && resource_id == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
    {
        u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)
    } else {
        resource_id
    }
}

fn law_private_model_manager_call_resource_id(
    requested: u32,
    mapped: u32,
    mirror: Option<ModelResourceSlotMirror>,
) -> u32 {
    if requested != mapped && mirror.is_some_and(|mirror| mirror.after_matches) {
        requested
    } else {
        mapped
    }
}

fn law_private_model_can_start_effective_result(
    requested: u32,
    mapped: u32,
    mirror: Option<ModelResourceSlotMirror>,
    original_result: u64,
) -> (u64, bool) {
    if !LAW_EXTRA_SLOT_PRIVATE_MODEL_CAN_START_DIAGNOSTIC_ENABLED
        || original_result != 0
        || requested != u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || mapped != u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)
    {
        return (original_result, false);
    }

    let Some(mirror) = mirror else {
        return (original_result, false);
    };

    if mirror.write_ok
        && mirror.after_matches
        && mirror.before_state == 0
        && mirror.after_state == 1
        && matches!(mirror.source_state, 1 | 2)
    {
        (1, true)
    } else {
        (original_result, false)
    }
}

fn is_model_resource_manager(manager: usize) -> bool {
    manager != 0 && read_game_global_pointer(MODEL_RESOURCE_MANAGER_RVA) == Some(manager)
}

fn mirror_law_private_model_manager_slot_for_request(
    manager: usize,
    resource_id: u32,
) -> Option<ModelResourceSlotMirror> {
    if !LAW_EXTRA_SLOT_PRIVATE_MODEL_MANAGER_ALIAS_ENABLED
        || resource_id != u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || !is_model_resource_manager(manager)
    {
        return None;
    }
    mirror_model_resource_manager_entry(
        manager,
        LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID,
        LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID,
    )
}

fn mirror_model_resource_manager_entry(
    manager: usize,
    source_id: u16,
    target_id: u16,
) -> Option<ModelResourceSlotMirror> {
    let source_address = model_resource_manager_entry_address(manager, u32::from(source_id))?;
    let target_address = model_resource_manager_entry_address(manager, u32::from(target_id))?;
    let mut source = [0u8; MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE];
    if !read_exact_process_memory(source_address, &mut source) {
        log_law_private_model_manager_slot_mirror(format!(
            "failed source_id={} target_id={} source=0x{:x} target=0x{:x} error=read_source_failed",
            source_id, target_id, source_address, target_address
        ));
        return None;
    }
    let mut before = [0u8; MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE];
    if !read_exact_process_memory(target_address, &mut before) {
        log_law_private_model_manager_slot_mirror(format!(
            "failed source_id={} target_id={} source=0x{:x} target=0x{:x} error=read_target_failed",
            source_id, target_id, source_address, target_address
        ));
        return None;
    }

    let write_attempted = before != source;
    let write_ok = if write_attempted {
        matches!(
            win::write_process_memory(target_address, &source),
            Some(written) if written == source.len()
        )
    } else {
        true
    };

    let mut after = [0u8; MODEL_RESOURCE_MANAGER_ENTRY_COPY_SIZE];
    let after_matches = read_exact_process_memory(target_address, &mut after) && after == source;
    let mirror = ModelResourceSlotMirror {
        source_pointer: model_resource_manager_entry_pointer(&source),
        source_state: model_resource_manager_entry_state(&source),
        before_pointer: model_resource_manager_entry_pointer(&before),
        before_state: model_resource_manager_entry_state(&before),
        after_pointer: model_resource_manager_entry_pointer(&after),
        after_state: model_resource_manager_entry_state(&after),
        write_attempted,
        write_ok,
        after_matches,
    };
    log_law_private_model_manager_slot_mirror(format!(
        "source_id={} target_id={} source=0x{:x} target=0x{:x} {}",
        source_id,
        target_id,
        source_address,
        target_address,
        format_model_resource_slot_mirror_detail(Some(mirror))
    ));
    Some(mirror)
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

fn is_interesting_model_resource_status_check(resource_id: u32) -> bool {
    matches!(resource_id, 356_022 | 420_000)
}

fn is_interesting_model_ready_wait_check(object: usize) -> bool {
    if object == 0 {
        return false;
    }
    read_u32_field(object, 0x18).is_some_and(is_interesting_model_ready_resource)
        || read_u32_field(object, 0x378).is_some_and(is_interesting_model_ready_resource)
}

fn format_model_resource_manager_entry(manager: usize, resource_id: u32) -> String {
    let Some(address) = model_resource_manager_entry_address(manager, resource_id) else {
        return format!("manager=0x{manager:x} id={resource_id} entry=unavailable");
    };
    format!(
        "manager=0x{manager:x} id={resource_id} entry=0x{address:x} ptr={} state={} raw_state={}",
        format_optional_address(read_usize_field(
            address,
            MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET
        )),
        format_optional_u32(read_u32_field(
            address,
            MODEL_RESOURCE_MANAGER_ENTRY_STATE_OFFSET
        )),
        format_optional_u32(read_u32_field(address, 0x08)),
    )
}

fn log_model_resource_status_check(
    manager: usize,
    resource_id: u32,
    result: u64,
    before: Option<String>,
    after: String,
) {
    let index = MODEL_RESOURCE_STATUS_CHECK_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_MODEL_RESOURCE_STATUS_CHECK_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Model resource status-check manager=0x{manager:x} id={resource_id} result={result} before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 8)
    ));
}

fn format_model_ready_wait_check_object(object: usize) -> String {
    if object == 0 {
        return "object=0x0".to_string();
    }
    let manager = read_game_global_pointer(MODEL_RESOURCE_STATUS_MANAGER_RVA).unwrap_or_default();
    let wait3c4 = read_u32_field(object, 0x3c4);
    let wait3c8 = read_u32_field(object, 0x3c8);
    format!(
        "object=0x{object:x} detail=[{}] compare_ids={}/{}/{} wait3c4_status=[{}] wait3c8_status=[{}]",
        format_model_ready_object_detail(object),
        format_optional_u32(read_u32_field(object, 0x378)),
        format_optional_u32(read_u32_field(object, 0x37c)),
        format_optional_u32(read_u32_field(object, 0x384)),
        wait3c4
            .map(|id| format_model_resource_manager_entry(manager, id))
            .unwrap_or_else(|| "none".to_string()),
        wait3c8
            .map(|id| format_model_resource_manager_entry(manager, id))
            .unwrap_or_else(|| "none".to_string()),
    )
}

fn log_model_ready_wait_check(
    object: usize,
    result: u64,
    before: Option<String>,
    after: String,
) {
    let index = MODEL_READY_WAIT_CHECK_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_MODEL_READY_WAIT_CHECK_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Model ready wait-check object=0x{object:x} result={result} before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 8)
    ));
}

fn format_model_load_state_step_trace(object: usize, load_model_hint: Option<u32>) -> String {
    let load_model = load_model_hint.or_else(|| read_u32_field(object, 0x18));
    let manager = read_game_global_pointer(MODEL_RESOURCE_MANAGER_RVA).unwrap_or_default();
    let manager_entry = load_model
        .map(|model| format_model_resource_manager_entry(manager, model))
        .unwrap_or_else(|| "manager_entry=none".to_string());
    format!(
        "detail=[{}] compare_ids={}/{}/{} manager=[{}]",
        format_model_ready_object_detail(object),
        format_optional_u32(read_u32_field(object, 0x378)),
        format_optional_u32(read_u32_field(object, 0x37c)),
        format_optional_u32(read_u32_field(object, 0x384)),
        manager_entry,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelLoadTimelineState {
    load_model: Option<u32>,
    load_color: Option<u32>,
    flags20: Option<u8>,
    state28: Option<u32>,
    phase2c: Option<u32>,
    compare_model: Option<u32>,
    compare_arg1: Option<u32>,
    compare_arg2: Option<u32>,
}

fn read_model_load_timeline_state(object: usize) -> ModelLoadTimelineState {
    ModelLoadTimelineState {
        load_model: read_u32_field(object, 0x18),
        load_color: read_u32_field(object, 0x1c),
        flags20: read_u8_field(object, 0x20),
        state28: read_u32_field(object, 0x28),
        phase2c: read_u32_field(object, 0x2c),
        compare_model: read_u32_field(object, 0x378),
        compare_arg1: read_u32_field(object, 0x37c),
        compare_arg2: read_u32_field(object, 0x384),
    }
}

fn law_ready_timeline_model_label(state: ModelLoadTimelineState) -> Option<&'static str> {
    match state.load_model {
        Some(308) => Some("slot3-oni"),
        Some(model) if model == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID) => {
            Some("slot5-custom")
        }
        _ => None,
    }
}

fn log_law_ready_timeline_model_load_state_step(
    object: usize,
    result: u64,
    before: ModelLoadTimelineState,
    after: ModelLoadTimelineState,
) {
    let Some(label) =
        law_ready_timeline_model_label(before).or_else(|| law_ready_timeline_model_label(after))
    else {
        return;
    };
    if before == after {
        return;
    }
    log_law_ready_timeline(
        "loader-step",
        format!(
            "slot_label={label} object=0x{object:x} result={result} before=[{}] after=[{}]",
            format_model_load_timeline_state(before),
            format_model_load_timeline_state(after)
        ),
    );
}

fn format_model_load_timeline_state(state: ModelLoadTimelineState) -> String {
    format!(
        "load={}/{} flags20={} state28={} phase2c={} compare={}/{}/{}",
        format_optional_u32(state.load_model),
        format_optional_u32(state.load_color),
        format_optional_u8(state.flags20),
        format_optional_u32(state.state28),
        format_optional_u32(state.phase2c),
        format_optional_u32(state.compare_model),
        format_optional_u32(state.compare_arg1),
        format_optional_u32(state.compare_arg2),
    )
}

fn log_model_load_state_step(object: usize, result: u64, before: Option<String>, after: String) {
    let index = MODEL_LOAD_STATE_STEP_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_MODEL_LOAD_STATE_STEP_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Model load state-step object=0x{object:x} result={result} before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 8)
    ));
}

fn format_model_resource_slot_mirror_detail(mirror: Option<ModelResourceSlotMirror>) -> String {
    match mirror {
        Some(mirror) => format!(
            "mirror=present attempted={} write_ok={} match={} source_ptr=0x{:x} source_state={} before_ptr=0x{:x} before_state={} after_ptr=0x{:x} after_state={}",
            mirror.write_attempted,
            mirror.write_ok,
            mirror.after_matches,
            mirror.source_pointer,
            mirror.source_state,
            mirror.before_pointer,
            mirror.before_state,
            mirror.after_pointer,
            mirror.after_state
        ),
        None => "mirror=none".to_string(),
    }
}

fn log_law_private_model_manager_slot_mirror(detail: String) {
    let index = MODEL_RESOURCE_SLOT_MIRROR_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= 64 {
        return;
    }
    log::write_line(format!("Law private model manager slot mirror {detail}"));
}

fn patch_model_resource_task_id(task: usize, from: u32, to: u32) -> Option<u32> {
    if task == 0 || read_u32_field(task, 0x08)? != from {
        return None;
    }
    let bytes = to.to_le_bytes();
    matches!(win::write_process_memory(task + 0x08, &bytes), Some(written) if written == bytes.len())
        .then_some(from)
}

fn restore_model_resource_task_id(task: usize, previous: u32) {
    if task == 0 {
        return;
    }
    let bytes = previous.to_le_bytes();
    let _ = win::write_process_memory(task + 0x08, &bytes);
}

fn log_law_private_model_manager_alias(
    action: &str,
    manager: usize,
    requested: u32,
    mapped: u32,
    detail: String,
) {
    let interesting = requested == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || mapped != requested;
    if !interesting {
        return;
    }
    let index = MODEL_RESOURCE_ALIAS_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= 96 {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Law private model manager alias action={action} manager=0x{manager:x} model_manager={} requested={} mapped={} {detail} frames={}",
        is_model_resource_manager(manager),
        requested,
        mapped,
        format_stack_frames(&frames, 8)
    ));
}
