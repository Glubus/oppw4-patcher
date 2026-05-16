unsafe extern "system" fn hooked_menu_visible_row_builder(state: usize, param2: u32) {
    let original_address = MENU_VISIBLE_ROW_BUILDER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let original: MenuVisibleRowBuilderFn = std::mem::transmute(original_address);
    original(state, param2);
    log_costume_menu_visible_rows(state, param2);
}

fn log_costume_menu_visible_rows(state: usize, param2: u32) {
    let Some(trace) = read_menu_state_trace(state) else {
        return;
    };

    let count = trace.visible_count.unwrap_or(u32::MAX);
    let entries = read_menu_visible_row_entries(state, count);
    let interesting =
        entries.iter().copied().any(is_law_menu_value) || is_interesting_menu_trace(trace);
    let logged = COSTUME_MENU_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= COSTUME_MENU_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = COSTUME_MENU_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_MENU_TRACE_LOGS {
        return;
    }

    log::write_line(format!(
        "Costume menu visible rows state=0x{state:x} p2={param2} {} entries={}",
        format_menu_state_trace(trace),
        format_u32_list(&entries)
    ));
}

fn read_menu_state_trace(state: usize) -> Option<MenuStateTrace> {
    if state == 0 {
        return None;
    }

    let visible_count = read_u32_field(state, MENU_VISIBLE_ROW_COUNT_OFFSET);
    let selected_index = read_u32_field(state, MENU_SELECTED_INDEX_OFFSET);
    let selected_layout = read_u32_field(state, MENU_SELECTED_LAYOUT_OFFSET);
    let selected_variant = read_u32_field(state, MENU_SELECTED_VARIANT_OFFSET);
    let entries = visible_count
        .map(|count| read_menu_visible_row_entries(state, count))
        .unwrap_or_default();
    let selected_row = selected_index
        .and_then(|index| entries.get(index as usize).copied())
        .filter(|row| *row != u32::MAX);

    Some(MenuStateTrace {
        mode: read_i32_field(state, MENU_MODE_OFFSET),
        category_mode: read_u32_field(state, MENU_CATEGORY_MODE_OFFSET),
        current_row: read_u32_field(state, MENU_CURRENT_ROW_OFFSET),
        slot_index: read_u32_field(state, MENU_SLOT_INDEX_OFFSET),
        row_override: read_u32_field(state, MENU_ROW_OVERRIDE_OFFSET),
        list_index: read_i32_field(state, MENU_LIST_INDEX_OFFSET),
        visible_count,
        selected_index,
        selected_row,
        selected_layout,
        selected_variant,
        focus_variant: read_focus_variant(selected_layout, selected_variant),
    })
}

fn format_menu_state_trace(trace: MenuStateTrace) -> String {
    format!(
        "mode={} category_mode={} current_row={} slot_index={} row_override={} list_index={} count={} selected_index={} selected_row={} selected_layout={} selected_variant={} focus_variant={}",
        format_optional_i32(trace.mode),
        format_optional_u32(trace.category_mode),
        format_optional_u32(trace.current_row),
        format_optional_u32(trace.slot_index),
        format_optional_u32(trace.row_override),
        format_optional_i32(trace.list_index),
        format_optional_u32(trace.visible_count),
        format_optional_u32(trace.selected_index),
        format_optional_u32(trace.selected_row),
        format_optional_u32(trace.selected_layout),
        format_optional_u32(trace.selected_variant),
        format_optional_u32(trace.focus_variant)
    )
}

fn is_interesting_menu_trace(trace: MenuStateTrace) -> bool {
    trace.current_row.is_some_and(is_law_menu_value)
        || trace.current_row == Some(LAW_MENU_ROW_ID as u32)
        || trace.slot_index.is_some_and(|value| value >= 3)
        || trace.row_override.is_some_and(is_law_menu_value)
        || trace.selected_row.is_some_and(is_law_menu_value)
        || trace.selected_layout.is_some_and(is_law_menu_value)
        || trace.selected_variant.is_some_and(is_law_menu_value)
        || trace.focus_variant.is_some_and(is_law_menu_value)
}

fn read_menu_visible_row_entries(state: usize, count: u32) -> Vec<u32> {
    if count == 0 || count > 0xfb {
        return Vec::new();
    }

    let entry_count = (count as usize).min(64);
    (0..entry_count)
        .filter_map(|index| {
            read_u32_field(
                state,
                MENU_VISIBLE_ROW_LIST_OFFSET + index * size_of::<u32>(),
            )
        })
        .collect()
}

fn is_law_menu_value(value: u32) -> bool {
    matches!(
        value,
        26 | 57 | 58 | 133 | 136 | 139 | 140 | 263 | 264 | 353 | 410 | 411 | 555 | 586
    ) || current_law_extra_slot_probe_variant_id()
        .is_some_and(|variant| value == u32::from(variant))
}

fn is_law_scene_bank_layout_id(value: u32) -> bool {
    matches!(value, 26 | 49 | 50 | 45 | 14 | 11 | 27 | 15 | 10 | 13)
}

fn is_law_menu_i32(value: i32) -> bool {
    value >= 0 && is_law_menu_value(value as u32)
}

unsafe extern "system" fn hooked_costume_selected_helper(state: usize, param2: u32) {
    let original_address = COSTUME_SELECTED_HELPER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_menu_state_trace(state);
    let original: CostumeSelectedHelperFn = std::mem::transmute(original_address);
    original(state, param2);
    let after = read_menu_state_trace(state);
    log_costume_selection_state_call("selected-helper", state, param2, before, after);
}

unsafe extern "system" fn hooked_costume_row_slot_ui_helper(state: usize, param2: u32) {
    let original_address = COSTUME_ROW_SLOT_UI_HELPER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_menu_state_trace(state);
    let original: CostumeRowSlotUiHelperFn = std::mem::transmute(original_address);
    original(state, param2);
    let after = read_menu_state_trace(state);
    log_costume_selection_state_call("row-slot-ui", state, param2, before, after);
}

unsafe extern "system" fn hooked_costume_slot_variant_helper(
    ui_state: usize,
    param2: usize,
    row_id: u32,
    slot_index: u32,
) {
    let original_address = COSTUME_SLOT_VARIANT_HELPER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let row_slot_before = read_costume_row_slot_value(row_id, slot_index);
    let original: CostumeSlotVariantHelperFn = std::mem::transmute(original_address);
    original(ui_state, param2, row_id, slot_index);
    let row_slot_after = read_costume_row_slot_value(row_id, slot_index);
    log_costume_slot_variant_helper_call(
        ui_state,
        param2,
        row_id,
        slot_index,
        row_slot_before,
        row_slot_after,
    );
}

fn log_costume_selection_state_call(
    label: &str,
    state: usize,
    param2: u32,
    before: Option<MenuStateTrace>,
    after: Option<MenuStateTrace>,
) {
    let interesting = before.is_some_and(is_interesting_menu_trace)
        || after.is_some_and(is_interesting_menu_trace)
        || is_law_menu_value(param2);
    let logged = COSTUME_SELECTION_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= COSTUME_SELECTION_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = COSTUME_SELECTION_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_SELECTION_TRACE_LOGS {
        return;
    }

    let before_text = before
        .map(format_menu_state_trace)
        .unwrap_or_else(|| "none".to_string());
    let after_text = after
        .map(format_menu_state_trace)
        .unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume selection {label} state=0x{state:x} p2={param2} before=[{before_text}] after=[{after_text}] frames={}",
        format_stack_frames(&frames, 6)
    ));
}

fn log_costume_slot_variant_helper_call(
    ui_state: usize,
    param2: usize,
    row_id: u32,
    slot_index: u32,
    row_slot_before: Option<u16>,
    row_slot_after: Option<u16>,
) {
    let interesting = is_law_menu_value(row_id)
        || slot_index >= 3
        || row_slot_before.is_some_and(|value| is_law_menu_value(value as u32))
        || row_slot_after.is_some_and(|value| is_law_menu_value(value as u32));
    let logged = COSTUME_SELECTION_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= COSTUME_SELECTION_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = COSTUME_SELECTION_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_SELECTION_TRACE_LOGS {
        return;
    }

    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume selection slot-variant ui=0x{ui_state:x} p2=0x{param2:x} row={} slot={} row_slot_before={} row_slot_after={} frames={}",
        row_id,
        slot_index,
        format_optional_u16(row_slot_before),
        format_optional_u16(row_slot_after),
        format_stack_frames(&frames, 6)
    ));
}

unsafe extern "system" fn hooked_costume_scene_presentation_helper(
    state: usize,
    param2: usize,
    param3: usize,
) {
    let original_address = COSTUME_SCENE_PRESENTATION_HELPER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_costume_scene_trace(state);
    patch_law_custom_scene_locked_flag("scene-presentation-enter", state, before);
    let original: CostumeScenePresentationHelperFn = std::mem::transmute(original_address);
    original(state, param2, param3);
    let after = read_costume_scene_trace(state);
    log_costume_scene_state_call(
        "scene-presentation",
        state,
        format!("p2=0x{param2:x} p3=0x{param3:x}"),
        before,
        after,
        false,
    );
}

unsafe extern "system" fn hooked_costume_scene_available_check(state: usize) -> u64 {
    let original_address = COSTUME_SCENE_AVAILABLE_CHECK_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let before = read_costume_scene_trace(state);
    let original: CostumeSceneAvailableCheckFn = std::mem::transmute(original_address);
    let result = original(state);
    let after = patch_law_custom_scene_locked_flag(
        "scene-available-check-leave",
        state,
        read_costume_scene_trace(state),
    );
    let (effective_result, forced) = law_custom_scene_available_check_result(
        result,
        before,
        after,
        current_law_extra_slot_probe_variant_id(),
    );
    log_costume_scene_state_call(
        "scene-available-check",
        state,
        format!("result={result} effective_result={effective_result} forced={forced}"),
        before,
        after,
        effective_result != 0 || forced,
    );
    effective_result
}

unsafe extern "system" fn hooked_costume_scene_apply_helper(state: usize) {
    let original_address = COSTUME_SCENE_APPLY_HELPER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_costume_scene_trace(state);
    let original: CostumeSceneApplyHelperFn = std::mem::transmute(original_address);
    original(state);
    let after = patch_law_custom_scene_locked_flag(
        "scene-apply-leave",
        state,
        read_costume_scene_trace(state),
    );
    log_costume_scene_state_call("scene-apply", state, String::new(), before, after, false);
}

unsafe extern "system" fn hooked_costume_scene_post_available_transition(state: usize) {
    let original_address = COSTUME_SCENE_POST_AVAILABLE_TRANSITION_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_costume_scene_trace(state);
    log_costume_scene_state_call(
        "scene-post-available-enter",
        state,
        read_costume_scene_post_available_detail(state),
        before,
        None,
        true,
    );
    let original: CostumeScenePostAvailableTransitionFn = std::mem::transmute(original_address);
    original(state);
    let after = read_costume_scene_trace(state);
    log_costume_scene_state_call(
        "scene-post-available-leave",
        state,
        read_costume_scene_post_available_detail(state),
        before,
        after,
        true,
    );
    log_costume_scene_list_probe("scene-post-available-leave", state, after);
}

unsafe extern "system" fn hooked_costume_scene_state_refresh(state: usize) {
    let original_address = COSTUME_SCENE_STATE_REFRESH_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_costume_scene_trace(state);
    let before_resources = read_tracked_preview_resource_states();
    poll_resource_911_watch(
        "FUN_141491120",
        "enter",
        format_scene_boundary_context(state, before),
    );
    poll_slot5_child8_watch_scene("FUN_141491120", "enter", state, before);
    let original: CostumeSceneStateRefreshFn = std::mem::transmute(original_address);
    original(state);
    let after = read_costume_scene_trace(state);
    let after_resources = read_tracked_preview_resource_states();
    poll_resource_911_watch(
        "FUN_141491120",
        "leave",
        format_scene_boundary_context(state, after),
    );
    poll_slot5_child8_watch_scene("FUN_141491120", "leave", state, after);
    log_costume_scene_state_call(
        "scene-state-refresh",
        state,
        String::new(),
        before,
        after,
        false,
    );
    let frames = capture_stack_trace();
    log_preview_resource_boundary(
        "FUN_141491120",
        state,
        before,
        after,
        before_resources,
        after_resources,
        &frames,
    );
}

unsafe extern "system" fn hooked_costume_scene_update_dispatcher(state: usize) {
    let original_address = COSTUME_SCENE_UPDATE_DISPATCHER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_costume_scene_trace(state);
    let before_resources = read_tracked_preview_resource_states();
    poll_resource_911_watch(
        "FUN_141491370",
        "enter",
        format_scene_boundary_context(state, before),
    );
    poll_slot5_child8_watch_scene("FUN_141491370", "enter", state, before);
    let original: CostumeSceneUpdateDispatcherFn = std::mem::transmute(original_address);
    original(state);
    let after = read_costume_scene_trace(state);
    let after_resources = read_tracked_preview_resource_states();
    poll_resource_911_watch(
        "FUN_141491370",
        "leave",
        format_scene_boundary_context(state, after),
    );
    poll_slot5_child8_watch_scene("FUN_141491370", "leave", state, after);
    log_costume_scene_state_call(
        "scene-update-dispatcher",
        state,
        String::new(),
        before,
        after,
        false,
    );
    let frames = capture_stack_trace();
    log_preview_resource_boundary(
        "FUN_141491370",
        state,
        before,
        after,
        before_resources,
        after_resources,
        &frames,
    );
    log_costume_scene_list_probe("scene-update-dispatcher", state, after);
}

unsafe extern "system" fn hooked_costume_scene_preview_refresh(state: usize, param2: i32) -> u64 {
    let original_address = COSTUME_SCENE_PREVIEW_REFRESH_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let before = read_costume_scene_trace(state);
    poll_resource_911_watch(
        "FUN_1414926a0",
        "enter",
        format_scene_boundary_context(state, before),
    );
    poll_slot5_child8_watch_scene("FUN_1414926a0", "enter", state, before);
    patch_law_custom_scene_list_preview_flag("scene-preview-refresh-enter", state, before);
    patch_law_custom_scene_locked_flag("scene-preview-refresh-enter", state, before);
    let original: CostumeScenePreviewRefreshFn = std::mem::transmute(original_address);
    let result = original(state, param2);
    let after = read_costume_scene_trace(state);
    poll_resource_911_watch(
        "FUN_1414926a0",
        "leave",
        format_scene_boundary_context(state, after),
    );
    poll_slot5_child8_watch_scene("FUN_1414926a0", "leave", state, after);
    log_costume_scene_state_call(
        "scene-preview-refresh",
        state,
        format!("p2={param2} result={result}"),
        before,
        after,
        result != 0,
    );
    log_costume_scene_list_probe("scene-preview-refresh", state, after);
    log_costume_preview_visible_decision("scene-preview-refresh", state, param2, result, after);
    result
}

unsafe extern "system" fn hooked_costume_scene_list_build(
    list_base: usize,
    param2: usize,
    param3: u32,
    param4: u32,
) {
    let original_address = COSTUME_SCENE_LIST_BUILD_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let state = scene_state_from_list_base(list_base);
    let before = state.and_then(read_costume_scene_trace);
    let before_build = read_costume_scene_list_rebuild_snapshot(list_base, param2);
    let before_source = format_costume_scene_list_source(param2);
    if let Some(state) = state {
        poll_resource_911_watch(
            "FUN_141493220",
            "enter",
            format_scene_boundary_context(state, before),
        );
        poll_slot5_child8_watch_scene("FUN_141493220", "enter", state, before);
    }
    let original: CostumeSceneListBuildFn = std::mem::transmute(original_address);
    original(list_base, param2, param3, param4);
    let after = state.and_then(read_costume_scene_trace);
    if let Some(state) = state {
        poll_resource_911_watch(
            "FUN_141493220",
            "leave",
            format_scene_boundary_context(state, after),
        );
        poll_slot5_child8_watch_scene("FUN_141493220", "leave", state, after);
    }
    let after_build = read_costume_scene_list_rebuild_snapshot(list_base, param2);
    let after_source = format_costume_scene_list_source(param2);
    let banks = format_costume_scene_list_bank_summary(list_base);
    if let Some(state) = state {
        log_costume_scene_list_build_detail(
            state,
            list_base,
            param2,
            [param3, param4],
            before,
            after,
            before_build,
            after_build,
            &before_source,
            &after_source,
            &banks,
        );
    }
}

unsafe extern "system" fn hooked_costume_scene_list_rebuild(
    list_base: usize,
    param2: usize,
    param3: u32,
    param4: u32,
    param5: u32,
    param6: u32,
) {
    let original_address = COSTUME_SCENE_LIST_REBUILD_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let state = scene_state_from_list_base(list_base);
    let before = state.and_then(read_costume_scene_trace);
    let before_rebuild = read_costume_scene_list_rebuild_snapshot(list_base, param2);
    if let Some(state) = state {
        poll_resource_911_watch(
            "FUN_141493820",
            "enter",
            format_scene_boundary_context(state, before),
        );
        poll_slot5_child8_watch_scene("FUN_141493820", "enter", state, before);
    }
    let original: CostumeSceneListRebuildFn = std::mem::transmute(original_address);
    original(list_base, param2, param3, param4, param5, param6);
    let after = state.and_then(read_costume_scene_trace);
    if let Some(state) = state {
        poll_resource_911_watch(
            "FUN_141493820",
            "leave",
            format_scene_boundary_context(state, after),
        );
        poll_slot5_child8_watch_scene("FUN_141493820", "leave", state, after);
    }
    let after_rebuild = read_costume_scene_list_rebuild_snapshot(list_base, param2);
    if let Some(state) = state {
        patch_law_custom_scene_list_preview_flag("scene-list-rebuild-leave", state, after);
        log_costume_scene_list_rebuild_detail(
            state,
            list_base,
            param2,
            [param3, param4, param5, param6],
            before,
            after,
            before_rebuild,
            after_rebuild,
        );
        log_costume_scene_state_call(
            "scene-list-rebuild",
            state,
            format!(
                "list_base=0x{list_base:x} p2=0x{param2:x} p3={param3} p4={param4} p5={param5} p6={param6}"
            ),
            before,
            after,
            false,
        );
        log_costume_scene_list_probe("scene-list-rebuild", state, after);
    }
}

unsafe extern "system" fn hooked_costume_scene_source_apply(state: usize, source: usize) {
    let original_address = COSTUME_SCENE_SOURCE_APPLY_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_costume_scene_trace(state);
    poll_resource_911_watch(
        "FUN_141493b30",
        "enter",
        format_scene_boundary_context(state, before),
    );
    poll_slot5_child8_watch_scene("FUN_141493b30", "enter", state, before);
    let original: CostumeSceneSourceApplyFn = std::mem::transmute(original_address);
    original(state, source);
    let after = read_costume_scene_trace(state);
    poll_resource_911_watch(
        "FUN_141493b30",
        "leave",
        format_scene_boundary_context(state, after),
    );
    poll_slot5_child8_watch_scene("FUN_141493b30", "leave", state, after);
    log_costume_scene_state_call(
        "scene-source-apply",
        state,
        format!("source=0x{source:x}"),
        before,
        after,
        false,
    );
}

unsafe extern "system" fn hooked_costume_object_update(state: usize) {
    let original_address = COSTUME_OBJECT_UPDATE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_costume_object_update_trace(state);
    update_law_custom_slot_activity_from_object_update(before);
    let before_frames = capture_stack_trace();
    poll_slot5_pre_controller_loss_window("object-update", "before", before, &before_frames);
    poll_slot5_controller_state("object-update", "before", before, &before_frames);
    poll_resource_911_watch(
        "FUN_141498580",
        "enter",
        preview_resource_trace_context(before),
    );
    let original: CostumeObjectUpdateFn = std::mem::transmute(original_address);
    original(state);
    let after = read_costume_object_update_trace(state);
    update_law_custom_slot_activity_from_object_update(after);
    let after_frames = capture_stack_trace();
    poll_slot5_pre_controller_loss_window("object-update", "after", after, &after_frames);
    poll_slot5_controller_state("object-update", "after", after, &after_frames);
    poll_resource_911_watch(
        "FUN_141498580",
        "leave",
        preview_resource_trace_context(after),
    );
    if let Some(object) = after.and_then(|trace| trace.object) {
        LAST_COSTUME_OBJECT_UPDATE_STATE.store(state, Ordering::Relaxed);
        LAST_COSTUME_OBJECT_UPDATE_OBJECT.store(object, Ordering::Relaxed);
    }
    update_law_private_model_late_ready_gate(after);
    log_costume_object_pending_change("object-update", before, after);
    maybe_run_slot5_refresh_after_911_ready_with_checkpoint("object-update", state, after);
    log_costume_object_update(state, before, after);
}

unsafe extern "system" fn hooked_costume_object_apply_ready(object: usize, slot: u32, ready: u32) {
    let original_address = COSTUME_OBJECT_APPLY_READY_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let interesting = is_interesting_costume_object_apply_ready(object, slot, ready);
    let before = if interesting {
        Some(format_costume_object_apply_ready_object(object, slot))
    } else {
        None
    };
    let before_pending = interesting
        .then(|| read_u8_field(object, COSTUME_OBJECT_UPDATE_OBJECT_PENDING_OFFSET))
        .flatten();
    let original: CostumeObjectApplyReadyFn = std::mem::transmute(original_address);
    original(object, slot, ready);
    if interesting {
        let after_pending = read_u8_field(object, COSTUME_OBJECT_UPDATE_OBJECT_PENDING_OFFSET);
        let trace = last_costume_object_update_trace_for_object(object);
        let frames = capture_stack_trace();
        poll_slot5_pre_controller_loss_window("apply-ready", "after", trace, &frames);
        poll_slot5_controller_state("apply-ready", "after", trace, &frames);
        log_costume_object_pending_value_change(
            "apply-ready",
            trace,
            before_pending,
            after_pending,
        );
        log_costume_object_apply_ready(
            object,
            slot,
            ready,
            before,
            format_costume_object_apply_ready_object(object, slot),
        );
        if let Some(trace) = trace {
            let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
            maybe_run_slot5_refresh_after_911_ready_with_checkpoint(
                "apply-ready",
                state,
                Some(trace),
            );
        }
    }
}

unsafe extern "system" fn hooked_costume_object_model_ready_check(
    loader: usize,
    model_resource: u32,
    arg1: u32,
    arg2: u32,
) -> u64 {
    let original_address = COSTUME_OBJECT_MODEL_READY_CHECK_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let interesting = is_interesting_costume_object_model_ready_check(model_resource, arg1, arg2);
    let before = if interesting {
        Some(format_costume_object_model_ready_loader(
            loader,
            model_resource,
            arg1,
            arg2,
        ))
    } else {
        None
    };
    let patch_detail =
        patch_law_private_model_ready_flag_if_selected(loader, model_resource, arg1, arg2);
    let original: CostumeObjectModelReadyCheckFn = std::mem::transmute(original_address);
    let original_result = original(loader, model_resource, arg1, arg2);
    let ready_override_detail =
        law_private_model_ready_override(loader, model_resource, arg1, arg2);
    let result = if ready_override_detail.is_some() && original_result == 0 {
        1
    } else {
        original_result
    };
    record_last_costume_object_busy_check(
        loader,
        model_resource,
        arg1,
        arg2,
        original_result,
        result,
    );
    let detail = format_model_ready_hook_detail(patch_detail, ready_override_detail);
    if interesting {
        log_costume_object_model_ready_check(
            loader,
            model_resource,
            arg1,
            arg2,
            original_result,
            result,
            detail,
            before,
            format_costume_object_model_ready_loader(loader, model_resource, arg1, arg2),
        );
    }
    log_law_ready_timeline_model_ready_check(
        loader,
        model_resource,
        arg1,
        arg2,
        original_result,
        result,
    );
    result
}

unsafe extern "system" fn hooked_costume_object_refresh_preview(state: usize, preview_arg: u32) {
    let original_address = COSTUME_OBJECT_REFRESH_PREVIEW_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let update_trace = read_costume_object_update_trace(state);
    try_preview_table_clone_292(update_trace, preview_arg);
    poll_resource_911_watch(
        "FUN_141497f50",
        "enter-direct",
        preview_resource_trace_context(update_trace),
    );
    let before = read_costume_object_refresh_preview_trace(state, preview_arg);
    let original: CostumeObjectRefreshPreviewFn = std::mem::transmute(original_address);
    original(state, preview_arg);
    poll_resource_911_watch(
        "FUN_141497f50",
        "leave-direct",
        preview_resource_trace_context(read_costume_object_update_trace(state)),
    );
    let after = read_costume_object_refresh_preview_trace(state, preview_arg);
    log_costume_object_refresh_preview(state, preview_arg, before, after);
}

unsafe extern "system" fn hooked_costume_object_refresh_preview_callsite(
    state: usize,
    preview_arg: u32,
) {
    let original_address = COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let update_trace = read_costume_object_update_trace(state);
    log_preview_table_before_original(preview_arg);
    try_preview_table_clone_292(update_trace, preview_arg);
    poll_resource_911_watch(
        "game+0x14986ce",
        "enter-callsite",
        preview_resource_trace_context(update_trace),
    );
    let before = read_costume_object_refresh_preview_trace(state, preview_arg);
    let before_update_trace = read_costume_object_update_trace(state);
    let original: CostumeObjectRefreshPreviewFn = std::mem::transmute(original_address);
    original(state, preview_arg);
    let after_update_trace = read_costume_object_update_trace(state);
    poll_resource_911_watch(
        "game+0x14986ce",
        "leave-callsite",
        preview_resource_trace_context(after_update_trace),
    );
    log_costume_object_pending_change("refresh-preview", before_update_trace, after_update_trace);
    log_preview_table_after_original(preview_arg);
    let after = read_costume_object_refresh_preview_trace(state, preview_arg);
    log_costume_object_refresh_preview_callsite(state, preview_arg, before, after);
}

unsafe extern "system" fn hooked_costume_preview_model_update(
    widget: usize,
    preview_variant: u32,
    visible: i32,
    layout_id: i32,
    fallback: u32,
) {
    let original_address = COSTUME_PREVIEW_MODEL_UPDATE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let interesting =
        is_interesting_preview_model_update(preview_variant, visible, layout_id, fallback);
    let before = if interesting {
        let trace = read_preview_model_widget_trace(widget);
        remember_preview_widget_child(trace);
        Some(trace)
    } else {
        None
    };
    let before_resource = if interesting {
        Some(format_preview_model_resource_probe(widget))
    } else {
        None
    };
    let before_preview_resources = interesting.then(read_tracked_preview_resource_states);
    let before_child8 = interesting.then(|| read_preview_child8_snapshot(widget));
    if interesting {
        let context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
        let frames = capture_stack_trace();
        poll_slot5_pre_controller_loss_window("FUN_14148b5f0", "enter", context, &frames);
        poll_slot5_controller_state("FUN_14148b5f0", "enter", context, &frames);
        poll_resource_911_watch(
            "FUN_14148b5f0",
            "enter",
            preview_resource_trace_context(context),
        );
        let render_context = preview_resource_trace_context(context);
        log_preview_child8_state(
            preview_model_update_child8_label(preview_variant, &render_context),
            "FUN_14148b5f0",
            "enter",
            widget,
            &render_context,
            &frames,
        );
    }
    let original: CostumePreviewModelUpdateFn = std::mem::transmute(original_address);
    let (effective_visible, forced_visible) = law_custom_preview_model_visible_arg(
        preview_variant,
        current_law_extra_slot_probe_variant_id(),
        visible,
        layout_id,
    );
    let forced_conditional = law_custom_preview_conditional_branch_diagnostic(
        preview_variant,
        current_law_extra_slot_probe_variant_id(),
        visible,
        layout_id,
        fallback,
    );
    if forced_conditional {
        let conditional_original =
            COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_ORIGINAL.load(Ordering::Acquire);
        let tail_original = COSTUME_PREVIEW_TAIL_UPDATE_ORIGINAL.load(Ordering::Acquire);
        if conditional_original != 0 && tail_original != 0 {
            let conditional: CostumePreviewConditionalVisibleBranchFn =
                std::mem::transmute(conditional_original);
            let tail: CostumePreviewTailUpdateFn = std::mem::transmute(tail_original);
            conditional(widget, layout_id as u32);
            tail(widget, preview_variant, visible);
        } else {
            original(
                widget,
                preview_variant,
                effective_visible,
                layout_id,
                fallback,
            );
        }
    } else {
        original(
            widget,
            preview_variant,
            effective_visible,
            layout_id,
            fallback,
        );
    }
    if interesting {
        let after = read_preview_model_widget_trace(widget);
        let after_child8 = read_preview_child8_snapshot(widget);
        remember_preview_widget_child(after);
        let after_resource = format_preview_model_resource_probe(widget);
        let after_preview_resources = read_tracked_preview_resource_states();
        let context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
        poll_resource_911_watch(
            "FUN_14148b5f0",
            "leave",
            preview_resource_trace_context(context),
        );
        let render_context = preview_resource_trace_context(context);
        let frames = capture_stack_trace();
        poll_slot5_pre_controller_loss_window("FUN_14148b5f0", "leave", context, &frames);
        poll_slot5_controller_state("FUN_14148b5f0", "leave", context, &frames);
        log_preview_child8_state(
            preview_model_update_child8_label(preview_variant, &render_context),
            "FUN_14148b5f0",
            "leave",
            widget,
            &render_context,
            &frames,
        );
        log_preview_child_render_candidate(
            "FUN_14148b5f0",
            "leave",
            widget,
            None,
            &render_context,
            &frames,
        );
        log_costume_preview_model_update(
            widget,
            preview_variant,
            visible,
            effective_visible,
            forced_visible,
            forced_conditional,
            layout_id,
            fallback,
            before,
            after,
            before_resource,
            after_resource,
            before_preview_resources,
            Some(after_preview_resources),
            before_child8,
            Some(after_child8),
        );
    }
}

unsafe extern "system" fn hooked_costume_preview_widget_resource_update(widget: usize) {
    let original_address = COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }

    let before_resources = read_tracked_preview_resource_states();
    let before_state = format_preview_widget_resource_update_state(widget);
    let before_context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let before_context = preview_resource_trace_context(before_context);
    poll_resource_911_watch("FUN_14148b800", "enter", before_context.clone());
    let before_frames = capture_stack_trace();
    log_slot5_ui_label_state(
        "FUN_14148b800",
        "enter",
        widget,
        LAST_SLOT5_COMPANION_PREVIEW_WIDGET.load(Ordering::Relaxed),
        &before_context,
        &before_frames,
    );
    log_preview_child8_state(
        preview_child_render_candidate_label(&before_context, widget),
        "FUN_14148b800",
        "enter",
        widget,
        &before_context,
        &before_frames,
    );
    log_preview_child_render_candidate(
        "FUN_14148b800",
        "enter",
        widget,
        None,
        &before_context,
        &before_frames,
    );
    maybe_reactivate_911_before_render_candidate(
        "FUN_14148b800",
        "enter",
        widget,
        &before_context,
        &before_frames,
    );

    let original: CostumePreviewWidgetResourceUpdateFn = std::mem::transmute(original_address);
    original(widget);

    let after_context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let after_context = preview_resource_trace_context(after_context);
    poll_resource_911_watch("FUN_14148b800", "leave", after_context.clone());
    let after_resources = read_tracked_preview_resource_states();
    let after_state = format_preview_widget_resource_update_state(widget);
    let after_frames = capture_stack_trace();
    log_slot5_ui_label_state(
        "FUN_14148b800",
        "leave",
        widget,
        LAST_SLOT5_COMPANION_PREVIEW_WIDGET.load(Ordering::Relaxed),
        &after_context,
        &after_frames,
    );
    log_preview_child8_state(
        preview_child_render_candidate_label(&after_context, widget),
        "FUN_14148b800",
        "leave",
        widget,
        &after_context,
        &after_frames,
    );
    log_preview_child_render_candidate(
        "FUN_14148b800",
        "leave",
        widget,
        None,
        &after_context,
        &after_frames,
    );
    log_preview_widget_resource_update(
        widget,
        before_state,
        after_state,
        before_resources,
        after_resources,
        before_context,
        after_context,
    );
}

unsafe extern "system" fn hooked_costume_preview_page_reset_callsite(widget: usize) {
    let original_address = COSTUME_PREVIEW_PAGE_RESET_CALLSITE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }

    let before_resources = read_tracked_preview_resource_states();
    let before_state = format_preview_widget_resource_update_state(widget);
    let before_context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let before_context = preview_resource_trace_context(before_context);
    poll_resource_911_watch("FUN_141488570-callsite", "before", before_context.clone());
    let before_frames = capture_stack_trace();
    log_preview_child_render_candidate(
        "FUN_141488570-callsite",
        "before",
        widget,
        None,
        &before_context,
        &before_frames,
    );

    let original: CostumePreviewPageResetFn = std::mem::transmute(original_address);
    original(widget);

    let after_context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let after_context = preview_resource_trace_context(after_context);
    poll_resource_911_watch("FUN_141488570-callsite", "after", after_context.clone());
    let after_resources = read_tracked_preview_resource_states();
    let after_state = format_preview_widget_resource_update_state(widget);
    let after_frames = capture_stack_trace();
    log_preview_child_render_candidate(
        "FUN_141488570-callsite",
        "after",
        widget,
        None,
        &after_context,
        &after_frames,
    );
    log_preview_page_reset_callsite(
        widget,
        before_state,
        after_state,
        before_resources,
        after_resources,
        before_context,
        after_context,
    );
}

unsafe extern "system" fn hooked_costume_preview_page_reset_tailjump(widget: usize) {
    let original_address = COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }

    let before_resources = read_tracked_preview_resource_states();
    let before_context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let before_context = preview_resource_trace_context(before_context);
    poll_resource_911_watch("FUN_141488570-tailjump", "before", before_context.clone());
    let before_frames = capture_stack_trace();
    log_preview_child_render_candidate(
        "FUN_141488570-tailjump",
        "before",
        widget,
        None,
        &before_context,
        &before_frames,
    );

    let original: CostumePreviewPageResetFn = std::mem::transmute(original_address);
    original(widget);

    let after_context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let after_context = preview_resource_trace_context(after_context);
    poll_resource_911_watch("FUN_141488570-tailjump", "after", after_context.clone());
    let after_resources = read_tracked_preview_resource_states();
    let after_frames = capture_stack_trace();
    log_preview_child_render_candidate(
        "FUN_141488570-tailjump",
        "after",
        widget,
        None,
        &after_context,
        &after_frames,
    );
    log_preview_page_reset_tailjump(
        widget,
        before_resources,
        after_resources,
        before_context,
        after_context,
    );
}

unsafe extern "system" fn hooked_costume_preview_resource_candidate_update(widget: usize) -> usize {
    let original_address =
        COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }

    let before_resources = read_tracked_preview_resource_states();
    let before_context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let before_context = preview_resource_trace_context(before_context);
    poll_resource_911_watch(
        "FUN_141488610-candidate-update",
        "before",
        before_context.clone(),
    );
    let before_frames = capture_stack_trace();
    log_slot5_ui_label_state(
        "FUN_141488610-candidate-update",
        "before",
        LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed),
        widget,
        &before_context,
        &before_frames,
    );
    log_preview_child8_state(
        preview_child_render_candidate_label(&before_context, widget),
        "FUN_141488610-candidate-update",
        "before",
        widget,
        &before_context,
        &before_frames,
    );
    log_preview_child_render_candidate(
        "FUN_141488610-candidate-update",
        "before",
        widget,
        None,
        &before_context,
        &before_frames,
    );
    maybe_reactivate_911_before_render_candidate(
        "FUN_141488610-candidate-update",
        "before",
        widget,
        &before_context,
        &before_frames,
    );
    maybe_restore_slot5_companion_mapped2d8_4713(
        "FUN_141488610-candidate-update",
        "before",
        widget,
        &before_context,
        &before_frames,
    );
    maybe_reactivate_stale_companion_1957(
        "FUN_141488610-candidate-update",
        "before",
        widget,
        &before_context,
        &before_frames,
    );

    let original: CostumePreviewResourceCandidateUpdateFn = std::mem::transmute(original_address);
    let result = original(widget);

    let after_context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let after_context = preview_resource_trace_context(after_context);
    poll_resource_911_watch(
        "FUN_141488610-candidate-update",
        "after",
        after_context.clone(),
    );
    let after_resources = read_tracked_preview_resource_states();
    let after_frames = capture_stack_trace();
    log_slot5_ui_label_state(
        "FUN_141488610-candidate-update",
        "after",
        LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed),
        widget,
        &after_context,
        &after_frames,
    );
    log_preview_child8_state(
        preview_child_render_candidate_label(&after_context, widget),
        "FUN_141488610-candidate-update",
        "after",
        widget,
        &after_context,
        &after_frames,
    );
    log_preview_child_render_candidate(
        "FUN_141488610-candidate-update",
        "after",
        widget,
        Some(result),
        &after_context,
        &after_frames,
    );
    if matches!(
        preview_child_render_candidate_label(&after_context, widget),
        Some("slot5" | "slot5-widget")
    ) {
        SLOT5_RENDER_CANDIDATE_SEQ.fetch_add(1, Ordering::Relaxed);
    }
    log_oni_final_render_result_baseline(
        "FUN_141488610-candidate-update",
        widget,
        result,
        &after_context,
        &after_frames,
    );
    log_preview_render_result(
        "FUN_141488610-candidate-update",
        widget,
        result,
        &after_context,
        &after_frames,
    );
    log_slot5_final_render_result_after_active_resources(
        "FUN_141488610-candidate-update",
        widget,
        result,
        &after_context,
        &after_frames,
    );
    log_slot5_result1957_consumer_boundary(
        "FUN_141488610-candidate-update",
        "after",
        &after_frames,
    );
    log_preview_candidate_result_probe(
        "FUN_141488610-candidate-update",
        widget,
        result,
        &after_context,
        &after_frames,
    );
    log_preview_resource_candidate_update(
        widget,
        result,
        before_resources,
        after_resources,
        before_context,
        after_context,
    );
    result
}

unsafe extern "system" fn hooked_costume_companion_preview_update(
    widget: usize,
    layout_id: u32,
    force_refresh: i32,
    scene_available: i32,
) -> usize {
    let original_address = COSTUME_COMPANION_PREVIEW_UPDATE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let interesting =
        is_interesting_companion_preview_update(widget, layout_id, force_refresh, scene_available);
    let before = if interesting {
        Some(format_companion_preview_widget_probe(widget))
    } else {
        None
    };
    let original: CostumeCompanionPreviewUpdateFn = std::mem::transmute(original_address);
    let result = original(widget, layout_id, force_refresh, scene_available);
    if interesting {
        let context = preview_resource_trace_context(
            last_law_ready_timeline_trace_label().map(|(_, trace)| trace),
        );
        remember_slot5_companion_preview_widget(
            widget,
            layout_id,
            force_refresh,
            scene_available,
            &context,
        );
        let frames = capture_stack_trace();
        log_preview_child_render_candidate(
            "FUN_141489700-companion",
            "leave",
            widget,
            Some(result),
            &context,
            &frames,
        );
        log_companion_resource_flow(
            widget,
            layout_id,
            force_refresh,
            scene_available,
            result,
            &context,
            &frames,
        );
        maybe_reactivate_stale_companion_1957(
            "FUN_141489700-companion",
            "leave",
            widget,
            &context,
            &frames,
        );
        log_slot5_result1957_consumer_boundary("FUN_141489700-companion", "leave", &frames);
        log_costume_companion_preview_update(
            widget,
            layout_id,
            force_refresh,
            scene_available,
            result,
            before,
            format_companion_preview_widget_probe(widget),
        );
    }
    result
}

unsafe extern "system" fn hooked_costume_preview_global_dlc_gate() -> u64 {
    let original_address = COSTUME_PREVIEW_GLOBAL_DLC_GATE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: CostumePreviewGlobalDlcGateFn = std::mem::transmute(original_address);
    let result = original();
    log_costume_preview_global_dlc_gate(result);
    result
}

unsafe extern "system" fn hooked_costume_preview_layout_mode_gate(
    layout_preview: u32,
    mode: u32,
) -> u64 {
    let original_address = COSTUME_PREVIEW_LAYOUT_MODE_GATE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: CostumePreviewLayoutModeGateFn = std::mem::transmute(original_address);
    let result = original(layout_preview, mode);
    log_costume_preview_layout_mode_gate(layout_preview, mode, result);
    result
}

#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
unsafe extern "system" fn hooked_costume_layout_availability_check(_layout_id: i32) -> u64 {
    std::arch::naked_asm!(
        "push r9",
        "push r10",
        "push r11",
        "sub rsp, 0x20",
        "call {inner}",
        "add rsp, 0x20",
        "pop r11",
        "pop r10",
        "pop r9",
        "ret",
        inner = sym hooked_costume_layout_availability_check_inner,
    );
}

#[cfg(not(target_arch = "x86_64"))]
unsafe extern "system" fn hooked_costume_layout_availability_check(layout_id: i32) -> u64 {
    hooked_costume_layout_availability_check_inner(layout_id)
}

unsafe extern "system" fn hooked_costume_layout_availability_check_inner(layout_id: i32) -> u64 {
    let trace = costume_layout_availability_check_reimplemented(layout_id);
    log_costume_layout_availability_check(trace);
    trace.result
}

unsafe extern "system" fn hooked_costume_preview_visible_branch(widget: usize, layout_id: u32) {
    let original_address = COSTUME_PREVIEW_VISIBLE_BRANCH_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let interesting = preview_model_branch_trace_candidate(Some(layout_id), None);
    let before = if interesting {
        Some(read_preview_model_widget_trace(widget))
    } else {
        None
    };
    let before_resource = if interesting {
        Some(format_preview_model_resource_probe(widget))
    } else {
        None
    };
    let original: CostumePreviewVisibleBranchFn = std::mem::transmute(original_address);
    original(widget, layout_id);
    if interesting {
        let after = read_preview_model_widget_trace(widget);
        let after_resource = format_preview_model_resource_probe(widget);
        log_costume_preview_branch_call(
            "visible-branch",
            widget,
            Some(layout_id),
            None,
            None,
            before,
            after,
            before_resource,
            after_resource,
        );
    }
}

unsafe extern "system" fn hooked_costume_preview_hidden_branch(widget: usize, fallback: i32) {
    let original_address = COSTUME_PREVIEW_HIDDEN_BRANCH_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let fallback_u32 = u32::try_from(fallback).ok();
    let interesting = preview_model_branch_trace_candidate(None, fallback_u32);
    let before = if interesting {
        Some(read_preview_model_widget_trace(widget))
    } else {
        None
    };
    let before_resource = if interesting {
        Some(format_preview_model_resource_probe(widget))
    } else {
        None
    };
    let original: CostumePreviewHiddenBranchFn = std::mem::transmute(original_address);
    original(widget, fallback);
    if interesting {
        let after = read_preview_model_widget_trace(widget);
        let after_resource = format_preview_model_resource_probe(widget);
        let label = format!("hidden-branch fallback={fallback}");
        log_costume_preview_branch_call(
            &label,
            widget,
            None,
            fallback_u32,
            None,
            before,
            after,
            before_resource,
            after_resource,
        );
    }
}

unsafe extern "system" fn hooked_costume_preview_conditional_visible_branch(
    widget: usize,
    layout_id: u32,
) {
    let original_address =
        COSTUME_PREVIEW_CONDITIONAL_VISIBLE_BRANCH_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let interesting = preview_model_branch_trace_candidate(Some(layout_id), None);
    let before = if interesting {
        Some(read_preview_model_widget_trace(widget))
    } else {
        None
    };
    let before_resource = if interesting {
        Some(format_preview_model_resource_probe(widget))
    } else {
        None
    };
    let original: CostumePreviewConditionalVisibleBranchFn = std::mem::transmute(original_address);
    original(widget, layout_id);
    if interesting {
        let after = read_preview_model_widget_trace(widget);
        let after_resource = format_preview_model_resource_probe(widget);
        log_costume_preview_branch_call(
            "conditional-visible-branch",
            widget,
            Some(layout_id),
            None,
            None,
            before,
            after,
            before_resource,
            after_resource,
        );
    }
}

unsafe extern "system" fn hooked_costume_preview_tail_update(
    widget: usize,
    preview_variant: u32,
    visible: i32,
) {
    let original_address = COSTUME_PREVIEW_TAIL_UPDATE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let interesting = preview_model_branch_trace_candidate(None, Some(preview_variant));
    let before = if interesting {
        Some(read_preview_model_widget_trace(widget))
    } else {
        None
    };
    let before_resource = if interesting {
        Some(format_preview_model_resource_probe(widget))
    } else {
        None
    };
    let original: CostumePreviewTailUpdateFn = std::mem::transmute(original_address);
    original(widget, preview_variant, visible);
    if interesting {
        let after = read_preview_model_widget_trace(widget);
        let after_resource = format_preview_model_resource_probe(widget);
        log_costume_preview_branch_call(
            "tail-update",
            widget,
            None,
            Some(preview_variant),
            Some(visible),
            before,
            after,
            before_resource,
            after_resource,
        );
    }
}

unsafe extern "system" fn hooked_costume_preview_resource_attach(
    queue: usize,
    mapped_resource_id: u32,
    render_context: u32,
    widget_context: i32,
) -> usize {
    let original_address = COSTUME_PREVIEW_RESOURCE_ATTACH_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let interesting = preview_resource_attach_trace_candidate(mapped_resource_id);
    let before = if interesting {
        Some(format_preview_resource_attach_queue(queue))
    } else {
        None
    };
    let before_queue = if tracked_preview_resource_id(mapped_resource_id) {
        Some(preview_resource_attach_queue_len_and_contains(
            queue,
            mapped_resource_id,
        ))
    } else {
        None
    };
    let before_queue_probe = if mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
    {
        Some(preview_resource_attach_queue_probe(
            queue,
            mapped_resource_id,
        ))
    } else {
        None
    };
    let before_911 = if tracked_preview_resource_id(mapped_resource_id) {
        read_tracked_preview_resource_states().resource_911
    } else {
        None
    };
    if interesting {
        let context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
        poll_resource_911_watch(
            "FUN_141613030",
            "enter",
            preview_resource_trace_context(context),
        );
    }
    let original: CostumePreviewResourceAttachFn = std::mem::transmute(original_address);
    let result = original(queue, mapped_resource_id, render_context, widget_context);
    let after_queue = if tracked_preview_resource_id(mapped_resource_id) {
        Some(preview_resource_attach_queue_len_and_contains(
            queue,
            mapped_resource_id,
        ))
    } else {
        None
    };
    let after_queue_probe = if mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID {
        Some(preview_resource_attach_queue_probe(
            queue,
            mapped_resource_id,
        ))
    } else {
        None
    };
    let after_911 = if tracked_preview_resource_id(mapped_resource_id) {
        read_tracked_preview_resource_states().resource_911
    } else {
        None
    };
    if interesting {
        let context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
        poll_resource_911_watch(
            "FUN_141613030",
            "leave",
            preview_resource_trace_context(context),
        );
    }
    if interesting {
        log_costume_preview_resource_attach(
            queue,
            mapped_resource_id,
            render_context,
            widget_context,
            result,
            before,
            format_preview_resource_attach_queue(queue),
            before_queue,
            after_queue,
            before_queue_probe.as_ref(),
            after_queue_probe.as_ref(),
            before_911,
            after_911,
        );
    }
    result
}

unsafe extern "system" fn hooked_costume_preview_resource_lookup_scan(
    table: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> usize {
    let original_address = COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }

    let before_resources = read_tracked_preview_resource_states();
    let before_context = preview_resource_lookup_scan_context(table, arg2, arg3, arg4);
    poll_resource_911_watch(
        "FUN_141582c30_contains_1582c6d",
        "enter",
        before_context.clone(),
    );
    RESOURCE_911_WRITER_BLOCK_SEQUENCE_START.store(
        RESOURCE_911_WRITER_BLOCK_SNAPSHOT_SEQ.load(Ordering::Acquire),
        Ordering::Release,
    );
    if let Some(entry) = before_resources
        .resource_911
        .filter(|entry| resource_911_watch_arm_candidate(*entry))
    {
        log_resource_911_writer_code_bytes_once();
        try_arm_resource_911_page_guard(entry, &before_context, "FUN_141582c30-enter");
    }

    let original: CostumePreviewResourceLookupScanFn = std::mem::transmute(original_address);
    let result = original(table, arg2, arg3, arg4);

    let after_context = preview_resource_lookup_scan_context(table, arg2, arg3, arg4);
    poll_resource_911_watch(
        "FUN_141582c30_contains_1582c6d",
        "leave",
        after_context.clone(),
    );
    disarm_resource_911_page_guard("FUN_141582c30-leave", &after_context);
    let after_resources = read_tracked_preview_resource_states();
    log_resource_911_writer_block_sequence(
        "FUN_141582c30-leave",
        before_resources,
        after_resources,
        &after_context,
    );
    log_costume_preview_resource_lookup_scan(
        table,
        arg2,
        arg3,
        arg4,
        result,
        before_resources,
        after_resources,
        before_context,
        after_context,
    );
    result
}

unsafe extern "system" fn hooked_costume_preview_resource_writer_block(
    rdi: usize,
    rbx: usize,
    r12: usize,
) {
    log_resource_911_writer_block(rdi, rbx, r12);
}

unsafe extern "system" fn hooked_costume_preview_resource_resolve(
    table: usize,
    mapped_resource_id: u32,
    render_context: u32,
    widget_context: i32,
    param5: i32,
) -> usize {
    let original_address = COSTUME_PREVIEW_RESOURCE_RESOLVE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let interesting = preview_resource_resolve_trace_candidate(mapped_resource_id);
    if tracked_preview_resource_id(mapped_resource_id) {
        LAST_PREVIEW_RESOURCE_TABLE.store(table, Ordering::Relaxed);
    }
    let before = if interesting {
        Some(format_preview_resource_resolve_table(
            table,
            mapped_resource_id,
        ))
    } else {
        None
    };
    let before_match = if tracked_preview_resource_id(mapped_resource_id) {
        preview_resource_resolve_match(table, mapped_resource_id)
    } else {
        None
    };
    let before_911 = if tracked_preview_resource_id(mapped_resource_id) {
        preview_resource_resolve_match(table, LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    } else {
        None
    };
    if interesting {
        let context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
        poll_resource_911_watch(
            "FUN_141582f50",
            "enter",
            preview_resource_trace_context(context),
        );
    }
    let original: CostumePreviewResourceResolveFn = std::mem::transmute(original_address);
    let result = original(
        table,
        mapped_resource_id,
        render_context,
        widget_context,
        param5,
    );
    let after_match = if tracked_preview_resource_id(mapped_resource_id) {
        preview_resource_resolve_match(table, mapped_resource_id)
    } else {
        None
    };
    let after_911 = if tracked_preview_resource_id(mapped_resource_id) {
        preview_resource_resolve_match(table, LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    } else {
        None
    };
    if interesting {
        let context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
        poll_resource_911_watch(
            "FUN_141582f50",
            "leave",
            preview_resource_trace_context(context),
        );
    }
    if interesting {
        log_costume_preview_resource_resolve(
            table,
            mapped_resource_id,
            render_context,
            widget_context,
            param5,
            result,
            before,
            format_preview_resource_resolve_table(table, mapped_resource_id),
            before_match,
            after_match,
            before_911,
            after_911,
        );
    }
    result
}

unsafe extern "system" fn hooked_costume_preview_resource_reset_boundary(owner: usize) {
    let original_address = COSTUME_PREVIEW_RESOURCE_RESET_BOUNDARY_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before_context = preview_resource_reset_boundary_context(owner);
    log_resource_911_watch_boundary("FUN_141614f40_contains_16150de", "enter", &before_context);
    poll_resource_911_watch("FUN_141614f40_contains_16150de", "enter", before_context);
    let original: CostumePreviewResourceResetBoundaryFn = std::mem::transmute(original_address);
    original(owner);
    let after_context = preview_resource_reset_boundary_context(owner);
    poll_resource_911_watch(
        "FUN_141614f40_contains_16150de",
        "leave",
        after_context.clone(),
    );
    log_resource_911_watch_boundary("FUN_141614f40_contains_16150de", "leave", &after_context);
}

unsafe extern "system" fn hooked_costume_preview_resource_virtual_update_callsite_before() {
    let context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let context = preview_resource_trace_context(context);
    log_resource_911_watch_boundary("game+0x133c19c", "before-vcall", &context);
    poll_resource_911_watch("game+0x133c19c", "before-vcall", context);
}

unsafe extern "system" fn hooked_costume_preview_resource_virtual_update_callsite_after() {
    let context = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    let context = preview_resource_trace_context(context);
    poll_resource_911_watch("game+0x133c19c", "after-vcall", context.clone());
    log_resource_911_watch_boundary("game+0x133c19c", "after-vcall", &context);
}

unsafe extern "system" fn hooked_ui_child_toggle(child: usize, index: u32, value: u32) -> usize {
    let original_address = UI_CHILD_TOGGLE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let interesting = is_interesting_ui_child_toggle(child, index);
    let before = if interesting {
        Some(format_ui_child_toggle_state(child, index))
    } else {
        None
    };
    let original: UiChildToggleFn = std::mem::transmute(original_address);
    let result = original(child, index, value);
    if interesting {
        log_ui_child_toggle(
            child,
            index,
            value,
            result,
            before,
            format_ui_child_toggle_state(child, index),
        );
    }
    result
}

unsafe extern "system" fn hooked_launch_costume_state_consumer(state: usize) -> usize {
    let original_address = LAUNCH_COSTUME_STATE_CONSUMER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let trace = patch_launch_costume_state_private_model_alias(state);
    log_launch_costume_state_call("enter", state, trace, None);
    let original: LaunchCostumeStateConsumerFn = std::mem::transmute(original_address);
    let result = original(state);
    let after = read_launch_costume_state_trace(state).or(trace);
    log_launch_costume_state_call("leave", state, after, Some(result));
    result
}

unsafe extern "system" fn hooked_costume_scene_fallback_layout(
    state: usize,
    layout_id: u32,
) -> u32 {
    let original_address = COSTUME_SCENE_FALLBACK_LAYOUT_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return layout_id;
    }
    let before = read_costume_scene_trace(state);
    let original: CostumeSceneFallbackLayoutFn = std::mem::transmute(original_address);
    let result = original(state, layout_id);
    let after = read_costume_scene_trace(state);
    log_costume_scene_state_call(
        "scene-fallback-layout",
        state,
        format!("layout={layout_id} result={result}"),
        before,
        after,
        is_law_menu_value(layout_id) || is_law_menu_value(result),
    );
    result
}

unsafe extern "system" fn hooked_costume_selection_lookup(
    table: usize,
    category: i32,
    layout_id: u32,
) -> u16 {
    let original_address = COSTUME_SELECTION_LOOKUP_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0x7fff;
    }
    let original: CostumeSelectionLookupFn = std::mem::transmute(original_address);
    let original_result = original(table, category, layout_id);
    let override_result = if DUPLICATE_LAW_VARIANT_SLOT_ENABLED.load(Ordering::Relaxed) {
        law_selection_lookup_override(category, layout_id, original_result)
    } else {
        None
    };
    let result = override_result.unwrap_or(original_result);
    update_law_custom_slot_activity_from_selection(category, layout_id, result);
    log_costume_selection_lookup(table, category, layout_id, original_result, result);
    result
}

unsafe extern "system" fn hooked_costume_selection_setter(
    table: usize,
    category: i32,
    variant: u16,
    layout_id: u32,
) {
    let original_address = COSTUME_SELECTION_SETTER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return;
    }
    let before = read_selection_cache_value(table, category, layout_id);
    let original: CostumeSelectionSetterFn = std::mem::transmute(original_address);
    original(table, category, variant, layout_id);
    let after = read_selection_cache_value(table, category, layout_id);
    log_costume_selection_setter(table, category, variant, layout_id, before, after);
}

unsafe extern "system" fn hooked_costume_variant_unlock_check(
    category: u32,
    variant: u32,
    slot_index: u32,
    strict: i32,
) -> u64 {
    let original_address = COSTUME_VARIANT_UNLOCK_CHECK_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: CostumeVariantUnlockCheckFn = std::mem::transmute(original_address);
    let original_result = original(category, variant, slot_index, strict);
    let override_result = law_variant_unlock_override_if_enabled(
        category,
        variant,
        slot_index,
        original_result,
    );
    let result = override_result.unwrap_or(original_result);
    log_costume_variant_unlock_check(
        category,
        variant,
        slot_index,
        strict,
        original_result,
        result,
    );
    result
}

fn law_selection_lookup_override(category: i32, layout_id: u32, result: u16) -> Option<u16> {
    let _ = (category, layout_id, result);
    None
}

fn update_law_custom_slot_activity_from_selection(category: i32, layout_id: u32, variant: u16) {
    if category != LAW_MASTER_CATEGORY_ID || layout_id != u32::from(LAW_MASTER_LAYOUT_ID) {
        return;
    }
    let Some(custom_variant) = current_law_extra_slot_probe_variant_id() else {
        return;
    };
    set_law_custom_slot_active(variant == custom_variant, "selection-lookup", Some(variant));
}

fn update_law_custom_slot_activity_from_object_update(trace: Option<CostumeObjectUpdateTrace>) {
    let Some(trace) = trace else {
        return;
    };
    if trace.selected_layout != Some(u32::from(LAW_MASTER_LAYOUT_ID)) {
        return;
    }
    let active = trace.selected_slot == Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32)
        && trace.selected_variant == Some(LAW_CUSTOM_LAYOUT_ID)
        && trace.selected_model_resource == Some(LAW_SLOT5_LINKDATA_MODEL_RESOURCE_ID);
    let official_law_slot = trace
        .selected_slot
        .is_some_and(|slot| slot != LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32)
        && trace
            .selected_variant
            .is_some_and(|variant| LAW_OFFICIAL_LAYOUT_VARIANTS.contains(&variant));
    if active {
        set_law_custom_slot_active(true, "object-update-selected-slot", trace.selected_variant);
    } else if official_law_slot {
        set_law_custom_slot_active(false, "object-update-official-slot", trace.selected_variant);
    }
}

fn set_law_custom_slot_active(active: bool, reason: &str, variant: Option<u16>) {
    let previous = LAW_EXTRA_SLOT_CUSTOM_ACTIVE.swap(active, Ordering::Relaxed);
    if previous == active {
        return;
    }
    log::write_line(format!(
        "Law custom slot active state active={} reason={} variant={}",
        active,
        reason,
        variant
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    ));
}

fn law_custom_slot_active() -> bool {
    LAW_EXTRA_SLOT_CUSTOM_ACTIVE.load(Ordering::Relaxed)
}

fn law_variant_unlock_override(category: u32, variant: u32, result: u64) -> Option<u64> {
    if category == LAW_MASTER_CATEGORY_ID as u32 && variant == LAW_HIDDEN_VARIANT_ID && result == 0
    {
        Some(1)
    } else {
        None
    }
}

fn law_variant_unlock_override_if_enabled(
    category: u32,
    variant: u32,
    slot_index: u32,
    result: u64,
) -> Option<u64> {
    if let Some(override_result) =
        law_linkdata_only_variant699_unlock_override(category, variant, slot_index, result)
    {
        return Some(override_result);
    }
    if UNLOCK_LAW_HIDDEN_VARIANT_ENABLED.load(Ordering::Relaxed) {
        if let Some(override_result) = law_variant_unlock_override(category, variant, result) {
            return Some(override_result);
        }
    }
    if DUPLICATE_LAW_VARIANT_SLOT_ENABLED.load(Ordering::Relaxed) {
        law_extra_slot_probe_unlock_override(category, variant, result)
    } else {
        None
    }
}

fn law_linkdata_only_variant699_unlock_override(
    category: u32,
    variant: u32,
    slot_index: u32,
    result: u64,
) -> Option<u64> {
    if !LAW_LINKDATA_ONLY_UNLOCK_VARIANT699_ENABLED || result != 0 {
        return None;
    }
    if category != LAW_MASTER_CATEGORY_ID as u32
        || variant != u32::from(LAW_CUSTOM_LAYOUT_ID)
        || slot_index != LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32
    {
        return None;
    }
    if read_law_layout_active_count() != Some(5)
        || read_layout_slot_variant(
            u32::from(LAW_MASTER_LAYOUT_ID),
            LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32,
        ) != Some(LAW_CUSTOM_LAYOUT_ID)
        || read_variant_model_resource(LAW_CUSTOM_LAYOUT_ID)
            != Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || read_variant_preview_mapping(LAW_CUSTOM_LAYOUT_ID)
            != Some(LAW_EXTRA_SLOT_PREVIEW_TABLE_MAPPING_SOURCE_ID as u16)
    {
        return None;
    }
    Some(1)
}

fn law_extra_slot_probe_unlock_override(category: u32, variant: u32, result: u64) -> Option<u64> {
    let allocated_variant = current_law_extra_slot_probe_variant_id()?;
    law_extra_slot_probe_unlock_override_for_variant(category, variant, result, allocated_variant)
}

fn law_extra_slot_probe_unlock_override_for_variant(
    category: u32,
    variant: u32,
    result: u64,
    allocated_variant: u16,
) -> Option<u64> {
    if !LAW_EXTRA_SLOT_FORCE_UNLOCK_ENABLED {
        return None;
    }
    if category == LAW_MASTER_CATEGORY_ID as u32
        && variant == u32::from(allocated_variant)
        && result == 0
    {
        Some(1)
    } else {
        None
    }
}

fn read_selection_cache_value(table: usize, category: i32, layout_id: u32) -> Option<u16> {
    let address = selection_cache_value_address(table, category, layout_id)?;
    let mut bytes = [0u8; 2];
    if read_exact_process_memory(address, &mut bytes) {
        Some(u16::from_le_bytes(bytes))
    } else {
        None
    }
}

fn selection_cache_value_address(table: usize, category: i32, layout_id: u32) -> Option<usize> {
    if table == 0 {
        return None;
    }
    if layout_id < COSTUME_LAYOUT_ROW_COUNT && layout_id >= 0x45 && layout_id - 0x45 < 0x0f {
        return Some(table + 0x1448 + (layout_id as usize - 0x45) * 2);
    }
    if category < 0 {
        return None;
    }
    Some(table + 0x1148 + category as usize * 2)
}

fn log_costume_selection_lookup(
    table: usize,
    category: i32,
    layout_id: u32,
    original_result: u16,
    result: u16,
) {
    let interesting = category == LAW_MASTER_CATEGORY_ID
        || is_law_menu_value(layout_id)
        || is_law_menu_value(original_result as u32)
        || is_law_menu_value(result as u32);
    if !interesting {
        return;
    }
    let index = COSTUME_SELECTION_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_SELECTION_TRACE_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume selection lookup table=0x{table:x} category={} layout={} original_result={} result={} forced={} frames={}",
        category,
        layout_id,
        original_result,
        result,
        original_result != result,
        format_stack_frames(&frames, 6)
    ));
}

fn log_costume_selection_setter(
    table: usize,
    category: i32,
    variant: u16,
    layout_id: u32,
    before: Option<u16>,
    after: Option<u16>,
) {
    let interesting = category == LAW_MASTER_CATEGORY_ID
        || is_law_menu_value(layout_id)
        || is_law_menu_value(variant as u32)
        || before.is_some_and(|value| is_law_menu_value(value as u32))
        || after.is_some_and(|value| is_law_menu_value(value as u32));
    let logged = COSTUME_SELECTION_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= COSTUME_SELECTION_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = COSTUME_SELECTION_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_SELECTION_TRACE_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume selection setter table=0x{table:x} category={} layout={} value={} before={} after={} changed={} frames={}",
        category,
        layout_id,
        variant,
        format_optional_u16_decimal(before),
        format_optional_u16_decimal(after),
        before != after,
        format_stack_frames(&frames, 6)
    ));
}

fn log_costume_variant_unlock_check(
    category: u32,
    variant: u32,
    slot_index: u32,
    strict: i32,
    original_result: u64,
    result: u64,
) {
    let interesting = category == LAW_MASTER_CATEGORY_ID as u32
        || is_law_menu_value(category)
        || is_law_menu_value(variant);
    if !interesting {
        return;
    }
    let index = COSTUME_UNLOCK_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_UNLOCK_TRACE_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume variant unlock-check category={} variant={} slot={} strict={} original_result={} result={} forced={} frames={}",
        category,
        variant,
        slot_index,
        strict,
        original_result,
        result,
        original_result != result,
        format_stack_frames(&frames, 6)
    ));
}

fn read_costume_scene_trace(state: usize) -> Option<CostumeSceneTrace> {
    if state == 0 {
        return None;
    }

    let layout = read_u32_field(state, COSTUME_SCENE_LAYOUT_OFFSET);
    let row = read_u32_field(state, COSTUME_SCENE_ROW_OFFSET);
    let selected_variant = read_u16_field(state, COSTUME_SCENE_SELECTED_VARIANT_OFFSET);
    let selected_object = read_usize_field(state, COSTUME_SCENE_SELECTED_OBJECT_OFFSET)
        .filter(|address| *address != 0);
    let object_layout = selected_object
        .and_then(|address| read_u32_field(address, COSTUME_SCENE_SELECTED_OBJECT_LAYOUT_OFFSET));
    let object_variant = selected_object
        .and_then(|address| read_u32_field(address, COSTUME_SCENE_SELECTED_OBJECT_VARIANT_OFFSET));

    Some(CostumeSceneTrace {
        layout,
        row,
        mode: read_u32_field(state, COSTUME_SCENE_MODE_OFFSET),
        selected_variant,
        selected_slot: layout
            .zip(selected_variant)
            .and_then(|(layout, variant)| find_layout_variant_index(layout, variant)),
        list_bank: read_u32_field(state, COSTUME_SCENE_LIST_BANK_OFFSET),
        list_index: read_u32_field(state, COSTUME_SCENE_LIST_INDEX_OFFSET),
        refresh_flag: read_u8_field(state, COSTUME_SCENE_FLAG_REFRESH_OFFSET),
        special_flag: read_u8_field(state, COSTUME_SCENE_FLAG_SPECIAL_OFFSET),
        locked_flag: read_u8_field(state, COSTUME_SCENE_FLAG_LOCKED_OFFSET),
        selected_object,
        object_layout,
        object_variant,
        object_slot: object_layout
            .zip(object_variant)
            .and_then(|(layout, variant)| {
                u16::try_from(variant)
                    .ok()
                    .and_then(|variant| find_layout_variant_index(layout, variant))
            }),
    })
}

fn scene_state_from_list_base(list_base: usize) -> Option<usize> {
    list_base.checked_sub(COSTUME_SCENE_LIST_BASE_OFFSET)
}

fn read_costume_scene_list_rebuild_snapshot(
    list_base: usize,
    param2: usize,
) -> Option<CostumeSceneListRebuildSnapshot> {
    if list_base == 0 {
        return None;
    }

    let bank = read_u32_field(list_base, COSTUME_SCENE_LIST_REBUILD_BANK_OFFSET);
    let active_entry = bank
        .filter(|bank| (*bank as usize) < COSTUME_SCENE_LIST_BANK_COUNT)
        .and_then(|bank| {
            read_scene_list_bank_starts_from_list_base(list_base).and_then(|starts| {
                let start = starts[bank as usize] as usize;
                start
                    .checked_mul(COSTUME_SCENE_LIST_ENTRY_STRIDE)
                    .and_then(|offset| list_base.checked_add(offset))
            })
        });
    let active_layouts = active_entry.and_then(read_scene_list_entry_layouts);
    let active_flags = active_entry.and_then(read_scene_list_entry_flags);
    Some(CostumeSceneListRebuildSnapshot {
        count: read_u32_field(list_base, COSTUME_SCENE_LIST_REBUILD_COUNT_OFFSET),
        selected_layout: read_u32_field(
            list_base,
            COSTUME_SCENE_LIST_REBUILD_SELECTED_LAYOUT_OFFSET,
        ),
        bank,
        row: read_u32_field(list_base, COSTUME_SCENE_LIST_REBUILD_ROW_OFFSET),
        index: read_u32_field(list_base, COSTUME_SCENE_LIST_REBUILD_INDEX_OFFSET),
        owner: read_usize_field(list_base, COSTUME_SCENE_LIST_REBUILD_OWNER_OFFSET)
            .filter(|address| *address != 0),
        widget: read_usize_field(list_base, COSTUME_SCENE_LIST_REBUILD_WIDGET_OFFSET)
            .filter(|address| *address != 0),
        flag: read_u8_field(list_base, COSTUME_SCENE_LIST_REBUILD_FLAG_OFFSET),
        param2_source: (param2 != 0)
            .then(|| read_usize_field(param2, COSTUME_SCENE_LIST_REBUILD_PARAM2_SOURCE_SLOT_OFFSET))
            .flatten()
            .filter(|address| *address != 0),
        active_entry,
        active_layouts,
        active_flags,
    })
}

fn format_costume_scene_list_rebuild_snapshot(
    snapshot: Option<CostumeSceneListRebuildSnapshot>,
) -> String {
    let Some(snapshot) = snapshot else {
        return "none".to_string();
    };
    format!(
        "count={} selected_layout={} bank={} row={} index={} owner={} widget={} flag={} param2_source={} active_entry={} active_layouts={} active_flags={}",
        format_optional_u32(snapshot.count),
        format_optional_u32(snapshot.selected_layout),
        format_optional_u32(snapshot.bank),
        format_optional_u32(snapshot.row),
        format_optional_u32(snapshot.index),
        format_optional_address(snapshot.owner),
        format_optional_address(snapshot.widget),
        format_optional_u8(snapshot.flag),
        format_optional_address(snapshot.param2_source),
        format_optional_address(snapshot.active_entry),
        snapshot
            .active_layouts
            .map(|layouts| format_u32_list(&layouts))
            .unwrap_or_else(|| "none".to_string()),
        snapshot
            .active_flags
            .map(|flags| format_bytes(&flags))
            .unwrap_or_else(|| "none".to_string())
    )
}

fn format_costume_scene_list_source(param2: usize) -> String {
    if param2 == 0 {
        return "param2=0x0 source=none".to_string();
    }
    let source = read_usize_field(param2, COSTUME_SCENE_LIST_REBUILD_PARAM2_SOURCE_SLOT_OFFSET)
        .filter(|address| *address != 0);
    let vtable = source
        .and_then(|address| read_usize_field(address, 0))
        .filter(|address| *address != 0);
    let call00 = vtable
        .and_then(|address| read_usize_field(address, 0))
        .filter(|address| *address != 0);
    let call10 = vtable
        .and_then(|address| read_usize_field(address, 0x10))
        .filter(|address| *address != 0);
    let call20 = vtable
        .and_then(|address| read_usize_field(address, 0x20))
        .filter(|address| *address != 0);
    format!(
        "param2=0x{param2:x} source={} vtable={} call00={} call10={} call20={}",
        format_optional_address(source),
        format_optional_address(vtable),
        format_optional_address(call00),
        format_optional_address(call10),
        format_optional_address(call20)
    )
}

fn format_costume_scene_list_bank_summary(list_base: usize) -> String {
    let Some(starts) = read_scene_list_bank_starts_from_list_base(list_base) else {
        return "none".to_string();
    };
    let mut banks = Vec::new();
    for (bank, start) in starts.iter().copied().enumerate() {
        let Some(entry) = start
            .checked_mul(COSTUME_SCENE_LIST_ENTRY_STRIDE as u32)
            .and_then(|offset| list_base.checked_add(offset as usize))
        else {
            continue;
        };
        let layouts = read_scene_list_entry_layouts(entry)
            .map(|layouts| format_u32_list(&layouts))
            .unwrap_or_else(|| "none".to_string());
        let flags = read_scene_list_entry_flags(entry)
            .map(|flags| format_bytes(&flags))
            .unwrap_or_else(|| "none".to_string());
        banks.push(format!(
            "{bank}:start={start}:entry=0x{entry:x}:layouts={layouts}:flags={flags}"
        ));
    }
    banks.join(";")
}

fn format_costume_scene_trace(trace: CostumeSceneTrace) -> String {
    format!(
        "layout={} row={} mode={} selected_variant={} selected_slot={} list_bank={} list_index={} refresh={} special={} locked={} object={} object_layout={} object_variant={} object_slot={}",
        format_optional_u32(trace.layout),
        format_optional_u32(trace.row),
        format_optional_u32(trace.mode),
        format_optional_u16_decimal(trace.selected_variant),
        format_optional_u32(trace.selected_slot),
        format_optional_u32(trace.list_bank),
        format_optional_u32(trace.list_index),
        format_optional_u8(trace.refresh_flag),
        format_optional_u8(trace.special_flag),
        format_optional_u8(trace.locked_flag),
        format_optional_address(trace.selected_object),
        format_optional_u32(trace.object_layout),
        format_optional_u32(trace.object_variant),
        format_optional_u32(trace.object_slot)
    )
}

fn read_preview_model_widget_trace(widget: usize) -> PreviewModelWidgetTrace {
    let child58 = read_usize_field(widget, 0x58).filter(|address| *address != 0);
    PreviewModelWidgetTrace {
        widget,
        child58,
        field10: read_u32_field(widget, 0x10),
        mapped294: read_u32_field(widget, 0x294),
        current_layout2dc: read_u32_field(widget, 0x2dc),
        visible2a0: read_u8_field(widget, 0x2a0),
        active2a1: read_u8_field(widget, 0x2a1),
        child_b4: child58.and_then(|address| read_u32_field(address, 0xb4)),
        child_28: child58.and_then(|address| read_usize_field(address, 0x28)),
        child_30: child58.and_then(|address| read_usize_field(address, 0x30)),
        child_38: child58.and_then(|address| read_usize_field(address, 0x38)),
        child_48: child58.and_then(|address| read_usize_field(address, 0x48)),
    }
}

fn format_preview_model_widget_trace(trace: PreviewModelWidgetTrace) -> String {
    format!(
        "widget=0x{:x} child58={} field10={} mapped294={} current_layout2dc={} visible2a0={} active2a1={} child_b4={} child28={} child30={} child38={} child48={}",
        trace.widget,
        format_optional_address(trace.child58),
        format_optional_u32(trace.field10),
        format_optional_u32(trace.mapped294),
        format_optional_u32(trace.current_layout2dc),
        format_optional_u8(trace.visible2a0),
        format_optional_u8(trace.active2a1),
        format_optional_u32(trace.child_b4),
        format_optional_address(trace.child_28),
        format_optional_address(trace.child_30),
        format_optional_address(trace.child_38),
        format_optional_address(trace.child_48)
    )
}

fn remember_preview_widget_child(trace: PreviewModelWidgetTrace) {
    if let Some(child) = trace.child58 {
        LAST_COSTUME_PREVIEW_WIDGET_CHILD.store(child, Ordering::Relaxed);
    }
}

fn is_interesting_ui_child_toggle(parent: usize, child_id: u32) -> bool {
    if parent == 0 || child_id != 0x37 {
        return false;
    }
    let Some(trace) = last_costume_object_update_trace_for_parent(parent) else {
        return false;
    };
    law_ready_timeline_costume_trace_label(trace).is_some()
}

fn last_costume_object_child58() -> usize {
    let object = LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed);
    if object == 0 {
        return 0;
    }
    read_usize_field(object, COSTUME_OBJECT_UPDATE_OBJECT_CHILD_OFFSET)
        .filter(|address| *address != 0)
        .unwrap_or(0)
}

fn format_ui_child_toggle_state(parent: usize, child_id: u32) -> String {
    if parent == 0 {
        return "parent=0x0".to_string();
    }

    let direct_lookup = lookup_ui_child_entry(parent, child_id, None);
    let table_start = ui_child_table_start(parent);
    let table_lookup = lookup_ui_child_entry(parent, child_id, table_start);
    let preview_child = LAST_COSTUME_PREVIEW_WIDGET_CHILD.load(Ordering::Relaxed);
    let object_child = last_costume_object_child58();
    let relation = if parent == object_child && parent == preview_child {
        "object+preview"
    } else if parent == object_child {
        "object"
    } else if parent == preview_child {
        "preview"
    } else {
        "other"
    };
    format!(
        "parent=0x{parent:x} relation={relation} child_id=0x{child_id:x} child28={} child30={} child38={} child48={} child54={} childb4={} table_start={} direct=[{}] table=[{}] context=[{}]",
        format_optional_address(read_usize_field(parent, 0x28)),
        format_optional_address(read_usize_field(parent, 0x30)),
        format_optional_address(read_usize_field(parent, 0x38)),
        format_optional_address(read_usize_field(parent, 0x48)),
        format_optional_u32(read_u32_field(parent, 0x54)),
        format_optional_u32(read_u32_field(parent, 0xb4)),
        format_optional_u32(table_start),
        format_ui_child_lookup(direct_lookup),
        format_ui_child_lookup(table_lookup),
        format_last_costume_object_update_context(LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed)),
    )
}

fn ui_child_table_start(parent: usize) -> Option<u32> {
    let state = read_u32_field(parent, 0xb4)?;
    if state >= 0x1269 {
        return Some(0);
    }
    let module = win::main_module() as usize;
    if module == 0 {
        return None;
    }
    read_u32_absolute(
        module
            + UI_CHILD_INDEX_START_TABLE_RVA
            + state as usize * UI_CHILD_INDEX_START_TABLE_STRIDE,
    )
}

fn lookup_ui_child_entry(parent: usize, child_id: u32, table_start: Option<u32>) -> Option<usize> {
    let (collection, index) = match table_start {
        Some(start) if start <= child_id => {
            let index = child_id - start;
            let count = read_u32_field(parent, 0x54)?;
            if index >= count {
                return None;
            }
            (read_usize_field(parent, 0x48)?, index)
        }
        _ => (read_usize_field(parent, 0x38)?, child_id),
    };
    read_usize_field(collection, index as usize * size_of::<usize>())
        .filter(|address| *address != 0)
}

fn format_ui_child_lookup(child: Option<usize>) -> String {
    let Some(child) = child else {
        return "missing".to_string();
    };
    format!(
        "ok {}",
        format_preview_model_child_entry_text(
            0x37,
            child,
            read_u32_field(child, 0x30),
            read_u8_field(child, 0x112),
            read_u32_field(child, 0x34),
            read_u32_field(child, 0x3c),
            read_u32_field(child, 0x44),
        )
    )
}

fn format_preview_widget_child37(trace: PreviewModelWidgetTrace) -> String {
    let Some(child58) = trace.child58 else {
        return "parent=none child37=missing".to_string();
    };
    let direct_lookup = lookup_ui_child_entry(child58, 0x37, None);
    let table_start = ui_child_table_start(child58);
    let table_lookup = lookup_ui_child_entry(child58, 0x37, table_start);
    format!(
        "parent=0x{child58:x} child28={} child30={} child38={} child48={} child54={} childb4={} table_start={} direct=[{}] table=[{}]",
        format_optional_address(read_usize_field(child58, 0x28)),
        format_optional_address(read_usize_field(child58, 0x30)),
        format_optional_address(read_usize_field(child58, 0x38)),
        format_optional_address(read_usize_field(child58, 0x48)),
        format_optional_u32(read_u32_field(child58, 0x54)),
        format_optional_u32(read_u32_field(child58, 0xb4)),
        format_optional_u32(table_start),
        format_ui_child_lookup(direct_lookup),
        format_ui_child_lookup(table_lookup),
    )
}

fn slot5_ui_child37_visibility_hint(widget: usize) -> &'static str {
    let trace = read_preview_model_widget_trace(widget);
    let Some(child58) = trace.child58 else {
        return "missing_parent";
    };
    let child = lookup_ui_child_entry(child58, 0x37, ui_child_table_start(child58))
        .or_else(|| lookup_ui_child_entry(child58, 0x37, None));
    let Some(child) = child else {
        return "missing_child37";
    };
    match read_u8_field(child, 0x112) {
        Some(1) => "visible_hint",
        Some(0) => "hidden_hint",
        Some(_) => "unknown_byte112",
        None => "unknown",
    }
}

fn log_slot5_ui_label_state(
    boundary: &str,
    phase: &str,
    main_widget: usize,
    companion_widget: usize,
    context: &str,
    frames: &[usize],
) {
    if !slot5_post_widget_cleanup_context_match(context) {
        return;
    }
    let index = SLOT5_UI_LABEL_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_UI_LABEL_STATE_LOGS {
        return;
    }

    let main = (main_widget != 0)
        .then(|| format_preview_model_widget_trace(read_preview_model_widget_trace(main_widget)))
        .unwrap_or_else(|| "none".to_string());
    let main_child37 = (main_widget != 0)
        .then(|| format_preview_widget_child37(read_preview_model_widget_trace(main_widget)))
        .unwrap_or_else(|| "none".to_string());
    let companion = (companion_widget != 0)
        .then(|| format_companion_preview_widget_probe(companion_widget))
        .unwrap_or_else(|| "none".to_string());
    let companion_child37 = (companion_widget != 0)
        .then(|| format_preview_widget_child37(read_preview_model_widget_trace(companion_widget)))
        .unwrap_or_else(|| "none".to_string());
    let child37_hint = if main_widget != 0 {
        slot5_ui_child37_visibility_hint(main_widget)
    } else {
        "main_widget_missing"
    };

    log::write_line(format!(
        "slot5-ui-label-state boundary={boundary} phase={phase} child37_hint={child37_hint} name_probe=not_identified main_widget={} companion_widget={} main=[{main}] main_child37=[{main_child37}] companion=[{companion}] companion_child37=[{companion_child37}] resource911_live=[{}] resource1957_live=[{}] context=[{context}] frames={}",
        format_optional_address((main_widget != 0).then_some(main_widget)),
        format_optional_address((companion_widget != 0).then_some(companion_widget)),
        format_preview_resource_911_live_state(),
        format_preview_resource_1957_live_state(),
        format_stack_frames(frames, 7)
    ));
}

fn log_ui_child_toggle(
    parent: usize,
    child_id: u32,
    enabled: u32,
    result: usize,
    before: Option<String>,
    after: String,
) {
    let index = UI_CHILD_TOGGLE_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_UI_CHILD_TOGGLE_TRACE_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    if let Some(trace) = last_costume_object_update_trace_for_parent(parent) {
        if let Some(label) = law_ready_timeline_costume_trace_label(trace) {
            log_law_ready_timeline(
                "child-toggle",
                format!(
                    "slot_label={label} parent=0x{parent:x} child=0x{child_id:x} enabled={enabled} result=0x{result:x} context=[{}] before=[{before}] after=[{after}] frames={}",
                    format_costume_object_update_trace(trace),
                    format_stack_frames(&frames, 6)
                ),
            );
            if label == "slot5-custom" {
                let index = SLOT5_CHILD_TOGGLE_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
                if index < MAX_SLOT5_CHILD_TOGGLE_STATE_LOGS {
                    log::write_line(format!(
                        "slot5-child-toggle-state parent=0x{parent:x} child=0x{child_id:x} enabled={enabled} result=0x{result:x} before=[{before}] after=[{after}] loader_object=[{}] alias=[{}] context=[{}] frames={}",
                        format_loader_object_final_state(
                            "slot5",
                            LAST_SLOT5_MODEL_READY_LOADER.load(Ordering::Relaxed),
                            LAST_SLOT5_MODEL_READY_OBJECT.load(Ordering::Relaxed),
                            u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                        ),
                        format_model_manager_alias_state(
                            "slot5",
                            u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                            Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)),
                        ),
                        format_costume_object_update_trace(trace),
                        format_stack_frames(&frames, 8)
                    ));
                }
            }
        }
    }
    log::write_line(format!(
        "UI child toggle parent=0x{parent:x} child=0x{child_id:x} enabled={enabled} result=0x{result:x} before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 8)
    ));
}

fn is_interesting_preview_model_update(
    preview_variant: u32,
    visible: i32,
    layout_id: i32,
    fallback: u32,
) -> bool {
    preview_model_update_global_trace_candidate(preview_variant, visible, layout_id, fallback)
        || preview_model_update_custom_trace_candidate(preview_variant, fallback)
        || last_law_ready_timeline_trace_label().is_some_and(|(label, _)| label == "slot5-custom")
}

fn preview_model_update_global_trace_candidate(
    preview_variant: u32,
    visible: i32,
    layout_id: i32,
    fallback: u32,
) -> bool {
    is_law_menu_value(preview_variant)
        || is_law_menu_i32(layout_id)
        || is_law_menu_value(fallback)
        || visible != 0
}

fn preview_model_update_custom_trace_candidate(preview_variant: u32, fallback: u32) -> bool {
    law_custom_slot_active()
        || current_law_extra_slot_probe_variant_id().is_some_and(|variant| {
            preview_variant == u32::from(variant) || fallback == u32::from(variant)
        })
}

fn preview_model_branch_trace_candidate(
    layout_id: Option<u32>,
    preview_variant: Option<u32>,
) -> bool {
    law_custom_slot_active()
        || layout_id.is_some_and(is_law_menu_value)
        || preview_variant.is_some_and(is_law_menu_value)
        || current_law_extra_slot_probe_variant_id().is_some_and(|variant| {
            preview_variant == Some(u32::from(variant))
                || layout_id == Some(u32::from(LAW_MASTER_LAYOUT_ID))
        })
}

fn is_interesting_companion_preview_update(
    widget: usize,
    layout_id: u32,
    _force_refresh: i32,
    _scene_available: i32,
) -> bool {
    widget != 0
        && (law_custom_slot_active()
            || is_law_menu_value(layout_id)
            || layout_id == u32::from(LAW_MASTER_LAYOUT_ID))
}

fn format_companion_preview_widget_probe(widget: usize) -> String {
    let queue = widget.checked_add(0x2c0);
    format!(
        "widget=0x{widget:x} child58={} mapped2d8={} layout2dc={} queue={} queue_state=[{}] children=[{}]",
        format_optional_address(read_usize_field(widget, 0x58).filter(|address| *address != 0)),
        format_optional_u32(read_u32_field(widget, 0x2d8)),
        format_optional_u32(read_u32_field(widget, 0x2dc)),
        format_optional_address(queue),
        queue
            .map(format_preview_resource_attach_queue)
            .unwrap_or_else(|| "none".to_string()),
        format_preview_model_child_collection_probe(widget)
    )
}

fn preview_companion_widget_shape_match(widget: usize) -> bool {
    if widget == 0 || read_u32_field(widget, 0x2dc) != Some(u32::from(LAW_MASTER_LAYOUT_ID)) {
        return false;
    }

    let queue_has_companion_resource = widget
        .checked_add(0x2c0)
        .is_some_and(|queue| preview_resource_attach_queue_len_and_contains(queue, 1957).1);
    let has_child6 = preview_model_child_object(widget, 6).is_some();
    let has_child7 = preview_model_child_object(widget, 7).is_some();
    let has_child8 = preview_model_child_object(widget, 8).is_some();

    queue_has_companion_resource && has_child6 && has_child7 && !has_child8
}

fn remember_slot5_companion_preview_widget(
    widget: usize,
    layout_id: u32,
    force_refresh: i32,
    scene_available: i32,
    _context: &str,
) {
    if widget == 0 || layout_id != u32::from(LAW_MASTER_LAYOUT_ID) {
        return;
    }

    LAST_SLOT5_COMPANION_PREVIEW_WIDGET.store(widget, Ordering::Relaxed);
    LAST_SLOT5_COMPANION_PREVIEW_LAYOUT.store(layout_id as usize, Ordering::Relaxed);
    LAST_SLOT5_COMPANION_PREVIEW_FORCE_REFRESH.store(force_refresh as usize, Ordering::Relaxed);
    LAST_SLOT5_COMPANION_PREVIEW_SCENE_AVAILABLE.store(scene_available as usize, Ordering::Relaxed);
    LAST_SLOT5_COMPANION_PREVIEW_WIDGET_SEQ.fetch_add(1, Ordering::Relaxed);
}

fn format_preview_model_resource_probe(widget: usize) -> String {
    format!(
        "res1d0_00={} res1d0_04={} res1d0_08={} res1d0_0c={} res1d0_10={} res1d0_14={} res1d0_18={} children=[{}]",
        format_optional_u32(read_u32_field(widget, 0x1d0)),
        format_optional_u32(read_u32_field(widget, 0x1d4)),
        format_optional_u32(read_u32_field(widget, 0x1d8)),
        format_optional_u32(read_u32_field(widget, 0x1dc)),
        format_optional_u32(read_u32_field(widget, 0x1e0)),
        format_optional_u32(read_u32_field(widget, 0x1e4)),
        format_optional_address(read_usize_field(widget, 0x1e8)),
        format_preview_model_child_collection_probe(widget)
    )
}

fn format_preview_model_child_collection_probe(widget: usize) -> String {
    const INTERESTING_CHILD_IDS: [u32; 7] = [6, 7, 8, 11, 15, 23, 31];
    const CHILD_ENTRY_STRIDE: usize = 0x10;
    const MAX_CHILD_ENTRIES_TO_SCAN: usize = 128;

    let Some(child58) = read_usize_field(widget, 0x58).filter(|address| *address != 0) else {
        return "child58=none".to_string();
    };
    let Some(collection) = read_usize_field(child58, 0x30).filter(|address| *address != 0) else {
        return format!("child58=0x{child58:x} collection=none");
    };
    let begin = read_usize_field(collection, 0x08);
    let end = read_usize_field(collection, 0x10);
    let len = begin
        .zip(end)
        .and_then(|(begin, end)| end.checked_sub(begin))
        .map(|bytes| bytes / CHILD_ENTRY_STRIDE);
    let mut found = Vec::new();

    if let Some((begin, len)) = begin.zip(len) {
        for index in 0..len.min(MAX_CHILD_ENTRIES_TO_SCAN) {
            let Some(entry) = begin.checked_add(index.saturating_mul(CHILD_ENTRY_STRIDE)) else {
                break;
            };
            let Some(id) = read_u32_field(entry, 0x00) else {
                continue;
            };
            if !INTERESTING_CHILD_IDS.contains(&id) {
                continue;
            }
            let object = read_usize_field(entry, 0x08).filter(|address| *address != 0);
            found.push(format_preview_model_child_entry_probe(id, object));
        }
    }

    format!(
        "child58=0x{child58:x} collection=0x{collection:x} begin={} end={} len={} found={}",
        format_optional_address(begin),
        format_optional_address(end),
        len.map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        if found.is_empty() {
            "none".to_string()
        } else {
            found.join(";")
        }
    )
}

fn preview_model_child_object(widget: usize, child_id: u32) -> Option<usize> {
    const CHILD_ENTRY_STRIDE: usize = 0x10;
    const MAX_CHILD_ENTRIES_TO_SCAN: usize = 128;

    let child58 = read_usize_field(widget, 0x58).filter(|address| *address != 0)?;
    let collection = read_usize_field(child58, 0x30).filter(|address| *address != 0)?;
    let begin = read_usize_field(collection, 0x08)?;
    let end = read_usize_field(collection, 0x10)?;
    let len = end.checked_sub(begin)? / CHILD_ENTRY_STRIDE;
    for index in 0..len.min(MAX_CHILD_ENTRIES_TO_SCAN) {
        let entry = begin.checked_add(index.saturating_mul(CHILD_ENTRY_STRIDE))?;
        if read_u32_field(entry, 0x00) == Some(child_id) {
            return read_usize_field(entry, 0x08).filter(|address| *address != 0);
        }
    }

    None
}

fn format_preview_children_678(widget: usize) -> String {
    [6, 7, 8]
        .into_iter()
        .map(|child_id| {
            format_preview_model_child_entry_probe(
                child_id,
                preview_model_child_object(widget, child_id),
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn format_preview_model_child_entry_probe(id: u32, object: Option<usize>) -> String {
    let Some(object) = object else {
        return format!("{id}:obj=none");
    };

    format_preview_model_child_entry_text(
        id,
        object,
        read_u32_field(object, 0x30),
        read_u8_field(object, 0x112),
        read_u32_field(object, 0x34),
        read_u32_field(object, 0x3c),
        read_u32_field(object, 0x44),
    )
}

fn format_preview_model_child_entry_text(
    id: u32,
    object: usize,
    flags30: Option<u32>,
    byte112: Option<u8>,
    value34: Option<u32>,
    value3c: Option<u32>,
    value44: Option<u32>,
) -> String {
    format!(
        "{id}:obj=0x{object:x},flags30={},b112={},v34={},v3c={},v44={}",
        format_optional_hex_u32(flags30),
        format_optional_u8(byte112),
        format_optional_hex_u32(value34),
        format_optional_hex_u32(value3c),
        format_optional_hex_u32(value44)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreviewChild8Snapshot {
    object: Option<usize>,
    flags30: Option<u32>,
    byte112: Option<u8>,
    value34: Option<u32>,
    value3c: Option<u32>,
    value44: Option<u32>,
}

impl PreviewChild8Snapshot {
    fn is_slot5_good(self) -> bool {
        self.object.is_some()
            && self.flags30 == Some(0x007f_ffd0)
            && self.value34 == Some(0x4400_0000)
            && self.value3c == Some(0x43fa_0000)
            && self.value44 == Some(0x3f80_0000)
    }
}

fn read_preview_child8_snapshot(widget: usize) -> PreviewChild8Snapshot {
    let object = preview_model_child_object(widget, 8);
    PreviewChild8Snapshot {
        object,
        flags30: object.and_then(|object| read_u32_field(object, 0x30)),
        byte112: object.and_then(|object| read_u8_field(object, 0x112)),
        value34: object.and_then(|object| read_u32_field(object, 0x34)),
        value3c: object.and_then(|object| read_u32_field(object, 0x3c)),
        value44: object.and_then(|object| read_u32_field(object, 0x44)),
    }
}

fn format_preview_child8_snapshot(snapshot: PreviewChild8Snapshot) -> String {
    let Some(object) = snapshot.object else {
        return "8:obj=none".to_string();
    };
    format_preview_model_child_entry_text(
        8,
        object,
        snapshot.flags30,
        snapshot.byte112,
        snapshot.value34,
        snapshot.value3c,
        snapshot.value44,
    )
}

fn read_preview_child8_object_snapshot(object: usize) -> PreviewChild8Snapshot {
    if object == 0 {
        return PreviewChild8Snapshot {
            object: None,
            flags30: None,
            byte112: None,
            value34: None,
            value3c: None,
            value44: None,
        };
    }
    PreviewChild8Snapshot {
        object: Some(object),
        flags30: read_u32_field(object, 0x30),
        byte112: read_u8_field(object, 0x112),
        value34: read_u32_field(object, 0x34),
        value3c: read_u32_field(object, 0x3c),
        value44: read_u32_field(object, 0x44),
    }
}

fn optional_u32_to_usize(value: Option<u32>) -> usize {
    value
        .map(|value| value as usize)
        .unwrap_or(u32::MAX as usize)
}

fn optional_u8_to_usize(value: Option<u8>) -> usize {
    value
        .map(|value| value as usize)
        .unwrap_or(u8::MAX as usize)
}

fn usize_to_optional_u32(value: usize) -> Option<u32> {
    (value != u32::MAX as usize).then_some(value as u32)
}

fn usize_to_optional_u8(value: usize) -> Option<u8> {
    (value != u8::MAX as usize).then_some(value as u8)
}

fn store_preview_child8_snapshot(label: &str, snapshot: PreviewChild8Snapshot) {
    let (valid, object, flags30, byte112, value34, value3c, value44) = if label == "oni" {
        (
            &LAST_ONI_PREVIEW_CHILD8_VALID,
            &LAST_ONI_PREVIEW_CHILD8_OBJECT,
            &LAST_ONI_PREVIEW_CHILD8_FLAGS30,
            &LAST_ONI_PREVIEW_CHILD8_B112,
            &LAST_ONI_PREVIEW_CHILD8_V34,
            &LAST_ONI_PREVIEW_CHILD8_V3C,
            &LAST_ONI_PREVIEW_CHILD8_V44,
        )
    } else {
        (
            &LAST_SLOT5_PREVIEW_CHILD8_VALID,
            &LAST_SLOT5_PREVIEW_CHILD8_OBJECT,
            &LAST_SLOT5_PREVIEW_CHILD8_FLAGS30,
            &LAST_SLOT5_PREVIEW_CHILD8_B112,
            &LAST_SLOT5_PREVIEW_CHILD8_V34,
            &LAST_SLOT5_PREVIEW_CHILD8_V3C,
            &LAST_SLOT5_PREVIEW_CHILD8_V44,
        )
    };

    object.store(snapshot.object.unwrap_or(0), Ordering::Relaxed);
    flags30.store(optional_u32_to_usize(snapshot.flags30), Ordering::Relaxed);
    byte112.store(optional_u8_to_usize(snapshot.byte112), Ordering::Relaxed);
    value34.store(optional_u32_to_usize(snapshot.value34), Ordering::Relaxed);
    value3c.store(optional_u32_to_usize(snapshot.value3c), Ordering::Relaxed);
    value44.store(optional_u32_to_usize(snapshot.value44), Ordering::Relaxed);
    valid.store(true, Ordering::Relaxed);
}

fn store_slot5_child8_watch_snapshot(snapshot: PreviewChild8Snapshot) {
    SLOT5_CHILD8_WATCH_OBJECT.store(snapshot.object.unwrap_or(0), Ordering::Relaxed);
    SLOT5_CHILD8_WATCH_FLAGS30.store(optional_u32_to_usize(snapshot.flags30), Ordering::Relaxed);
    SLOT5_CHILD8_WATCH_B112.store(optional_u8_to_usize(snapshot.byte112), Ordering::Relaxed);
    SLOT5_CHILD8_WATCH_V34.store(optional_u32_to_usize(snapshot.value34), Ordering::Relaxed);
    SLOT5_CHILD8_WATCH_V3C.store(optional_u32_to_usize(snapshot.value3c), Ordering::Relaxed);
    SLOT5_CHILD8_WATCH_V44.store(optional_u32_to_usize(snapshot.value44), Ordering::Relaxed);
}

fn load_slot5_child8_watch_snapshot() -> Option<PreviewChild8Snapshot> {
    SLOT5_CHILD8_WATCH_ARMED
        .load(Ordering::Relaxed)
        .then(|| PreviewChild8Snapshot {
            object: (SLOT5_CHILD8_WATCH_OBJECT.load(Ordering::Relaxed) != 0)
                .then_some(SLOT5_CHILD8_WATCH_OBJECT.load(Ordering::Relaxed)),
            flags30: usize_to_optional_u32(SLOT5_CHILD8_WATCH_FLAGS30.load(Ordering::Relaxed)),
            byte112: usize_to_optional_u8(SLOT5_CHILD8_WATCH_B112.load(Ordering::Relaxed)),
            value34: usize_to_optional_u32(SLOT5_CHILD8_WATCH_V34.load(Ordering::Relaxed)),
            value3c: usize_to_optional_u32(SLOT5_CHILD8_WATCH_V3C.load(Ordering::Relaxed)),
            value44: usize_to_optional_u32(SLOT5_CHILD8_WATCH_V44.load(Ordering::Relaxed)),
        })
}

fn maybe_arm_slot5_child8_watch(
    boundary: &str,
    phase: &str,
    widget: usize,
    snapshot: PreviewChild8Snapshot,
    context: &str,
    frames: &[usize],
) {
    if !snapshot.is_slot5_good() {
        return;
    }
    let object = snapshot.object.unwrap_or(0);
    if SLOT5_CHILD8_WATCH_ARMED.load(Ordering::Relaxed)
        && SLOT5_CHILD8_WATCH_OBJECT.load(Ordering::Relaxed) == object
    {
        store_slot5_child8_watch_snapshot(snapshot);
        return;
    }

    SLOT5_CHILD8_WATCH_ARMED.store(true, Ordering::Relaxed);
    store_slot5_child8_watch_snapshot(snapshot);
    let index = PREVIEW_CHILD8_WATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_PREVIEW_CHILD8_WATCH_LOGS {
        log::write_line(format!(
            "slot5-child8-watch-armed boundary={boundary} phase={phase} widget=0x{widget:x} child8=[{}] resource911_live=[{}] context=[{context}] frames={}",
            format_preview_child8_snapshot(snapshot),
            format_preview_resource_911_live_state(),
            format_stack_frames(frames, 8)
        ));
        log::write_line(format!(
            "slot5-child8-good-before-update boundary={boundary} phase={phase} widget=0x{widget:x} child8=[{}] resource911_live=[{}] context=[{context}] frames={}",
            format_preview_child8_snapshot(snapshot),
            format_preview_resource_911_live_state(),
            format_stack_frames(frames, 8)
        ));
    }
}

fn poll_slot5_child8_watch(boundary: &str, phase: &str, context: &str, frames: &[usize]) {
    let Some(previous) = load_slot5_child8_watch_snapshot() else {
        return;
    };
    let object = previous.object.unwrap_or(0);
    if object == 0 {
        return;
    }
    let current = read_preview_child8_object_snapshot(object);
    if current == previous {
        return;
    }
    store_slot5_child8_watch_snapshot(current);
    let index = PREVIEW_CHILD8_WATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_PREVIEW_CHILD8_WATCH_LOGS {
        log::write_line(format!(
            "preview-child8-hidden-change boundary={boundary} phase={phase} before=[{}] after=[{}] resource911_live=[{}] context=[{context}] frames={}",
            format_preview_child8_snapshot(previous),
            format_preview_child8_snapshot(current),
            format_preview_resource_911_live_state(),
            format_stack_frames(frames, 8)
        ));
    }
}

fn restore_slot5_child8_after_preview_update(
    widget: usize,
    context: &str,
    frames: &[usize],
) -> Option<PreviewChild8Snapshot> {
    if !LAW_SLOT5_RESTORE_CHILD8_AFTER_PREVIEW_UPDATE_ENABLED {
        return None;
    }
    if !slot5_post_widget_cleanup_context_match(context) {
        log_slot5_child8_restore_skip("context_mismatch", widget, None, context, frames);
        return None;
    }
    let Some(resource_911) = read_tracked_preview_resource_states().resource_911 else {
        log_slot5_child8_restore_skip("resource911_missing", widget, None, context, frames);
        return None;
    };
    if resource_911.state != Some(1) || resource_911.marker != Some(0) {
        log_slot5_child8_restore_skip(
            "resource911_not_active",
            widget,
            Some(resource_911),
            context,
            frames,
        );
        return None;
    }

    let before = read_preview_child8_snapshot(widget);
    let Some(object) = before.object else {
        log_slot5_child8_restore_skip(
            "child8_missing",
            widget,
            Some(resource_911),
            context,
            frames,
        );
        return None;
    };
    if before.is_slot5_good() {
        log_slot5_child8_restore_skip("already_good", widget, Some(resource_911), context, frames);
        return Some(before);
    }
    if !SLOT5_CHILD8_WATCH_ARMED.load(Ordering::Relaxed) {
        log_slot5_child8_restore_skip(
            "watch_not_armed",
            widget,
            Some(resource_911),
            context,
            frames,
        );
        return None;
    }
    if SLOT5_CHILD8_WATCH_OBJECT.load(Ordering::Relaxed) != object {
        log_slot5_child8_restore_skip(
            "object_mismatch",
            widget,
            Some(resource_911),
            context,
            frames,
        );
        return None;
    }

    let flags_written = write_u32_field_checked(object, 0x30, 0x007f_ffd0);
    let v34_written = write_u32_field_checked(object, 0x34, 0x4400_0000);
    let v3c_written = write_u32_field_checked(object, 0x3c, 0x43fa_0000);
    let v44_written = write_u32_field_checked(object, 0x44, 0x3f80_0000);
    let after = read_preview_child8_object_snapshot(object);
    store_slot5_child8_watch_snapshot(after);
    store_preview_child8_snapshot("slot5", after);

    let index = SLOT5_CHILD8_RESTORE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_CHILD8_RESTORE_LOGS {
        log::write_line(format!(
            "law-slot5-child8-restore-after-preview-update widget=0x{widget:x} writes=[flags30:{flags_written} v34:{v34_written} v3c:{v3c_written} v44:{v44_written}] before=[{}] after=[{}] resource911_live=[{}] context=[{context}] frames={}",
            format_preview_child8_snapshot(before),
            format_preview_child8_snapshot(after),
            format_preview_resource_match_state(Some(resource_911)),
            format_stack_frames(frames, 8)
        ));
    }

    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    maybe_run_slot5_refresh_after_911_ready_with_checkpoint(
        "child8-restore-after-preview-update",
        state,
        read_costume_object_update_trace(state),
    );

    Some(after)
}

fn log_slot5_child8_restore_skip(
    reason: &str,
    widget: usize,
    resource_911: Option<PreviewResourceResolveMatch>,
    context: &str,
    frames: &[usize],
) {
    let index = SLOT5_CHILD8_RESTORE_SKIP_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_CHILD8_RESTORE_SKIP_LOGS {
        return;
    }
    log::write_line(format!(
        "law-slot5-child8-restore-skip reason={reason} widget=0x{widget:x} child8=[{}] watch_armed={} watch_object=0x{:x} resource911_live=[{}] context=[{context}] frames={}",
        format_preview_child8_snapshot(read_preview_child8_snapshot(widget)),
        SLOT5_CHILD8_WATCH_ARMED.load(Ordering::Relaxed),
        SLOT5_CHILD8_WATCH_OBJECT.load(Ordering::Relaxed),
        resource_911
            .map(|state| format_preview_resource_match_state(Some(state)))
            .unwrap_or_else(format_preview_resource_911_live_state),
        format_stack_frames(frames, 8)
    ));
}

fn poll_slot5_child8_watch_scene(
    boundary: &str,
    phase: &str,
    state: usize,
    trace: Option<CostumeSceneTrace>,
) {
    if !SLOT5_CHILD8_WATCH_ARMED.load(Ordering::Relaxed) {
        return;
    }
    let context = format_scene_boundary_context(state, trace);
    let frames = capture_stack_trace();
    poll_slot5_child8_watch(boundary, phase, &context, &frames);
}

fn load_preview_child8_snapshot(label: &str) -> Option<PreviewChild8Snapshot> {
    let (valid, object, flags30, byte112, value34, value3c, value44) = if label == "oni" {
        (
            &LAST_ONI_PREVIEW_CHILD8_VALID,
            &LAST_ONI_PREVIEW_CHILD8_OBJECT,
            &LAST_ONI_PREVIEW_CHILD8_FLAGS30,
            &LAST_ONI_PREVIEW_CHILD8_B112,
            &LAST_ONI_PREVIEW_CHILD8_V34,
            &LAST_ONI_PREVIEW_CHILD8_V3C,
            &LAST_ONI_PREVIEW_CHILD8_V44,
        )
    } else {
        (
            &LAST_SLOT5_PREVIEW_CHILD8_VALID,
            &LAST_SLOT5_PREVIEW_CHILD8_OBJECT,
            &LAST_SLOT5_PREVIEW_CHILD8_FLAGS30,
            &LAST_SLOT5_PREVIEW_CHILD8_B112,
            &LAST_SLOT5_PREVIEW_CHILD8_V34,
            &LAST_SLOT5_PREVIEW_CHILD8_V3C,
            &LAST_SLOT5_PREVIEW_CHILD8_V44,
        )
    };

    valid
        .load(Ordering::Relaxed)
        .then(|| PreviewChild8Snapshot {
            object: (object.load(Ordering::Relaxed) != 0).then_some(object.load(Ordering::Relaxed)),
            flags30: usize_to_optional_u32(flags30.load(Ordering::Relaxed)),
            byte112: usize_to_optional_u8(byte112.load(Ordering::Relaxed)),
            value34: usize_to_optional_u32(value34.load(Ordering::Relaxed)),
            value3c: usize_to_optional_u32(value3c.load(Ordering::Relaxed)),
            value44: usize_to_optional_u32(value44.load(Ordering::Relaxed)),
        })
}

fn preview_model_update_child8_label(preview_variant: u32, context: &str) -> Option<&'static str> {
    if slot5_custom_preview_variant() == Some(preview_variant)
        || slot5_post_widget_cleanup_context_match(context)
    {
        return Some("slot5");
    }
    if preview_variant == u32::from(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID)
        || context.contains("selected_variant=586")
    {
        return Some("oni");
    }
    None
}

fn log_preview_child8_state(
    label: Option<&'static str>,
    boundary: &str,
    phase: &str,
    widget: usize,
    context: &str,
    frames: &[usize],
) {
    let Some(label) = label else {
        return;
    };
    let label = match label {
        "slot5" | "slot5-widget" => "slot5",
        "oni" => "oni",
        _ => return,
    };
    let snapshot = read_preview_child8_snapshot(widget);
    if label == "slot5" {
        poll_slot5_child8_watch(boundary, phase, context, frames);
        if boundary == "FUN_14148b800" && phase == "leave" {
            maybe_arm_slot5_child8_watch(boundary, phase, widget, snapshot, context, frames);
        }
    }
    let previous = load_preview_child8_snapshot(label);
    let changed = previous.is_some_and(|previous| previous != snapshot);

    let always_log = label == "oni"
        || boundary == "FUN_14148b5f0"
        || boundary == "preview-widget-final"
        || (boundary == "FUN_14148b800" && phase == "leave");
    let timeline_index = if always_log {
        PREVIEW_CHILD8_TIMELINE_LOGS.fetch_add(1, Ordering::Relaxed)
    } else {
        PREVIEW_CHILD8_TIMELINE_LOGS.load(Ordering::Relaxed)
    };
    if always_log && timeline_index < MAX_PREVIEW_CHILD8_TIMELINE_LOGS {
        log::write_line(format!(
            "preview-child8-state label={label} boundary={boundary} phase={phase} widget=0x{widget:x} child8=[{}] resource911_live=[{}] context=[{context}] frames={}",
            format_preview_child8_snapshot(snapshot),
            format_preview_resource_911_live_state(),
            format_stack_frames(frames, 7)
        ));
    }

    if changed && (label == "slot5" || always_log) {
        let change_index = PREVIEW_CHILD8_CHANGE_LOGS.fetch_add(1, Ordering::Relaxed);
        if change_index < MAX_PREVIEW_CHILD8_CHANGE_LOGS {
            let previous = previous.expect("checked by changed");
            log::write_line(format!(
                "preview-child8-change label={label} boundary={boundary} phase={phase} widget=0x{widget:x} before=[{}] after=[{}] resource911_live=[{}] context=[{context}] frames={}",
                format_preview_child8_snapshot(previous),
                format_preview_child8_snapshot(snapshot),
                format_preview_resource_911_live_state(),
                format_stack_frames(frames, 8)
            ));
        }
    }

    store_preview_child8_snapshot(label, snapshot);
}

fn format_preview_model_update_gate_probe(layout_id: i32) -> String {
    let layout_preview_value = preview_model_update_layout_preview_value(layout_id);
    let static_flag = layout_preview_value.and_then(preview_model_update_static_flag);
    let runtime_mode = preview_model_update_runtime_mode();
    format!(
        "layout_preview={} runtime_mode={} static_flag={} static_flag20={}",
        format_optional_u32(layout_preview_value),
        format_optional_u8(runtime_mode),
        format_optional_u8(static_flag),
        static_flag
            .map(|flag| ((flag & 0x20) != 0).to_string())
            .unwrap_or_else(|| "none".to_string())
    )
}

fn preview_model_update_layout_preview_value(layout_id: i32) -> Option<u32> {
    if layout_id == 0x221 {
        return Some(u32::MAX);
    }
    let layout_id = u32::try_from(layout_id).ok()?;
    if layout_id >= COSTUME_LAYOUT_ROW_COUNT {
        return None;
    }
    let address = static_layouts_address()? + layout_id as usize * COSTUME_LAYOUT_ROW_STRIDE + 0x16;
    read_u16_absolute(address).map(u32::from)
}

fn preview_model_update_static_flag(layout_preview_value: u32) -> Option<u8> {
    if layout_preview_value >= COSTUME_CATEGORY_COUNT as u32 {
        return None;
    }
    let address = static_layouts_address()? + 0xc43c + layout_preview_value as usize * 0x34;
    read_u8_field(address, 0)
}

fn preview_model_update_runtime_mode() -> Option<u8> {
    let main_module = win::main_module() as usize;
    if main_module == 0 {
        return None;
    }
    let database = read_usize_absolute(main_module + costume_table::RUNTIME_DATABASE_RVA)?;
    let root = read_usize_absolute(database + COSTUME_STATIC_ROOT_OFFSET)?;
    let state = read_usize_absolute(root + 0x18)?;
    read_u8_field(state, 0x13)
}

fn preview_gate_trace_interesting(layout_preview: u32, mode: u32, result: u64) -> bool {
    law_custom_slot_active() || is_law_menu_value(layout_preview) || mode == 4 || result == 0
}

fn stack_has_preview_model_update_gate_call(frames: &[usize]) -> bool {
    frames.iter().copied().any(|frame| {
        game_frame_rva(frame).is_some_and(|rva| {
            (COSTUME_PREVIEW_MODEL_UPDATE_RVA..COSTUME_PREVIEW_MODEL_UPDATE_RVA + 0x180)
                .contains(&rva)
        })
    })
}

fn log_costume_preview_global_dlc_gate(result: u64) {
    let frames = capture_stack_trace();
    log_slot5_result1957_consumer_boundary("preview-global-dlc-gate", "after", &frames);
    let from_preview_model_update = stack_has_preview_model_update_gate_call(&frames);
    if !law_custom_slot_active() && !from_preview_model_update {
        return;
    }
    let index = COSTUME_PREVIEW_GATE_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_GATE_TRACE_LOGS {
        return;
    }
    log::write_line(format!(
        "Costume preview gate kind=global-dlc result={result} custom_active={} from_model_update={from_preview_model_update} frames={}",
        law_custom_slot_active(),
        format_stack_frames(&frames, 8)
    ));
}

fn log_costume_preview_layout_mode_gate(layout_preview: u32, mode: u32, result: u64) {
    let interesting = preview_gate_trace_interesting(layout_preview, mode, result);
    let frames = capture_stack_trace();
    log_slot5_result1957_consumer_boundary("preview-layout-mode-gate", "after", &frames);
    let from_preview_model_update = stack_has_preview_model_update_gate_call(&frames);
    if !interesting && !from_preview_model_update {
        return;
    }
    let index = COSTUME_PREVIEW_GATE_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_GATE_TRACE_LOGS {
        return;
    }
    log::write_line(format!(
        "Costume preview gate kind=layout-mode layout_preview={layout_preview} mode={mode} result={result} custom_active={} from_model_update={from_preview_model_update} frames={}",
        law_custom_slot_active(),
        format_stack_frames(&frames, 8)
    ));
}

fn costume_layout_availability_check_reimplemented(
    layout_id: i32,
) -> CostumeLayoutAvailabilityTrace {
    if layout_id < 0 || layout_id >= COSTUME_LAYOUT_ROW_COUNT as i32 {
        return CostumeLayoutAvailabilityTrace {
            layout_id,
            row_flag: None,
            target_index: None,
            preview_value: None,
            matched_index: None,
            matched_flag: None,
            result: 0,
            reason: "layout-out-of-range",
        };
    }

    let Some(base) = static_layouts_address() else {
        return CostumeLayoutAvailabilityTrace {
            layout_id,
            row_flag: None,
            target_index: None,
            preview_value: None,
            matched_index: None,
            matched_flag: None,
            result: 0,
            reason: "static-layouts-missing",
        };
    };
    let Some(row) = (layout_id as usize)
        .checked_mul(COSTUME_LAYOUT_ROW_STRIDE)
        .and_then(|offset| base.checked_add(offset))
    else {
        return CostumeLayoutAvailabilityTrace {
            layout_id,
            row_flag: None,
            target_index: None,
            preview_value: None,
            matched_index: None,
            matched_flag: None,
            result: 0,
            reason: "row-address-overflow",
        };
    };

    let row_flag = read_u8_field(row, COSTUME_LAYOUT_AVAILABILITY_FLAG_OFFSET);
    let target_index = read_u16_field(row, COSTUME_LAYOUT_AVAILABILITY_TARGET_OFFSET);
    let preview_value = read_u16_field(row, COSTUME_LAYOUT_AVAILABILITY_PREVIEW_OFFSET);
    if !row_flag.is_some_and(|flag| (flag & 2) != 0) {
        return CostumeLayoutAvailabilityTrace {
            layout_id,
            row_flag,
            target_index,
            preview_value,
            matched_index: None,
            matched_flag: None,
            result: 0,
            reason: "row-flag-bit2-zero",
        };
    }
    let Some(target_index) = target_index else {
        return CostumeLayoutAvailabilityTrace {
            layout_id,
            row_flag,
            target_index: None,
            preview_value,
            matched_index: None,
            matched_flag: None,
            result: 0,
            reason: "target-index-read-failed",
        };
    };

    for index in 0..COSTUME_LAYOUT_AVAILABILITY_TABLE_COUNT {
        let Some(entry) = index
            .checked_mul(COSTUME_LAYOUT_AVAILABILITY_TABLE_STRIDE)
            .and_then(|offset| COSTUME_LAYOUT_AVAILABILITY_TABLE_OFFSET.checked_add(offset))
            .and_then(|offset| base.checked_add(offset))
        else {
            return CostumeLayoutAvailabilityTrace {
                layout_id,
                row_flag,
                target_index: Some(target_index),
                preview_value,
                matched_index: None,
                matched_flag: None,
                result: 0,
                reason: "table-address-overflow",
            };
        };
        let Some(flag) = read_u8_field(entry, 0) else {
            return CostumeLayoutAvailabilityTrace {
                layout_id,
                row_flag,
                target_index: Some(target_index),
                preview_value,
                matched_index: None,
                matched_flag: None,
                result: 0,
                reason: "table-read-failed",
            };
        };
        if (flag & 2) == 0 && index as u16 == target_index {
            return CostumeLayoutAvailabilityTrace {
                layout_id,
                row_flag,
                target_index: Some(target_index),
                preview_value,
                matched_index: Some(index as u16),
                matched_flag: Some(flag),
                result: 1,
                reason: "matched-open-index",
            };
        }
    }

    CostumeLayoutAvailabilityTrace {
        layout_id,
        row_flag,
        target_index: Some(target_index),
        preview_value,
        matched_index: None,
        matched_flag: None,
        result: 0,
        reason: "target-index-blocked-or-missing",
    }
}

fn log_costume_layout_availability_check(trace: CostumeLayoutAvailabilityTrace) {
    let layout = u32::try_from(trace.layout_id).ok();
    let interesting = law_custom_slot_active()
        || layout.is_some_and(is_law_menu_value)
        || layout.is_some_and(is_law_scene_bank_layout_id)
        || trace
            .preview_value
            .is_some_and(|value| is_law_menu_value(u32::from(value)))
        || trace.result == 0;
    let logged = COSTUME_LAYOUT_AVAILABILITY_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= COSTUME_MENU_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = COSTUME_LAYOUT_AVAILABILITY_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_LAYOUT_AVAILABILITY_TRACE_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume layout availability-check layout={} row_flag={} target_index={} preview={} matched_index={} matched_flag={} result={} reason={} custom_active={} frames={}",
        trace.layout_id,
        format_optional_u8(trace.row_flag),
        format_optional_u16_decimal(trace.target_index),
        format_optional_u16_decimal(trace.preview_value),
        format_optional_u16_decimal(trace.matched_index),
        format_optional_u8(trace.matched_flag),
        trace.result,
        trace.reason,
        law_custom_slot_active(),
        format_stack_frames(&frames, 8)
    ));
}

fn preview_resource_attach_trace_candidate(mapped_resource_id: u32) -> bool {
    law_custom_slot_active()
        || mapped_resource_id == LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID
        || mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
}

fn preview_resource_resolve_trace_candidate(mapped_resource_id: u32) -> bool {
    preview_resource_attach_trace_candidate(mapped_resource_id) || mapped_resource_id == 1957
}

fn tracked_preview_resource_id(mapped_resource_id: u32) -> bool {
    mapped_resource_id == LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID
        || mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
}

fn format_preview_resource_attach_queue(queue: usize) -> String {
    let count = read_u32_field(queue, 0x00);
    let vector = read_usize_field(queue, 0x08).filter(|address| *address != 0);
    let begin = vector.and_then(|address| read_usize_field(address, 0x08));
    let end = vector.and_then(|address| read_usize_field(address, 0x10));
    let len = begin
        .zip(end)
        .and_then(|(begin, end)| end.checked_sub(begin))
        .map(|bytes| bytes / size_of::<u32>());
    let values = begin
        .zip(len)
        .filter(|(_, len)| *len <= 64)
        .map(|(begin, len)| read_u32_pointer_values(begin as *const u32, len.min(16)))
        .unwrap_or_default();
    format!(
        "count={} vector={} begin={} end={} len={} values={}",
        format_optional_u32(count),
        format_optional_address(vector),
        format_optional_address(begin),
        format_optional_address(end),
        len.map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        format_u32_list(&values)
    )
}

fn preview_resource_attach_queue_len_and_contains(queue: usize, resource_id: u32) -> (usize, bool) {
    let Some(vector) = read_usize_field(queue, 0x08).filter(|address| *address != 0) else {
        return (usize::MAX, false);
    };
    let Some(begin) = read_usize_field(vector, 0x08) else {
        return (usize::MAX, false);
    };
    let Some(end) = read_usize_field(vector, 0x10) else {
        return (usize::MAX, false);
    };
    let Some(len) = end.checked_sub(begin).map(|bytes| bytes / size_of::<u32>()) else {
        return (usize::MAX, false);
    };
    let values = if len <= 64 {
        read_u32_pointer_values(begin as *const u32, len)
    } else {
        Vec::new()
    };
    (len, values.contains(&resource_id))
}

fn preview_resource_attach_queue_end(queue: usize) -> Option<usize> {
    let vector = read_usize_field(queue, 0x08).filter(|address| *address != 0)?;
    read_usize_field(vector, 0x10).filter(|address| *address != 0)
}

#[derive(Debug, Clone)]
struct PreviewResourceQueueEntryProbe {
    index: usize,
    slot_addr: usize,
    id: u32,
}

#[derive(Debug, Clone)]
struct PreviewResourceQueueProbe {
    len: usize,
    entries: Vec<PreviewResourceQueueEntryProbe>,
}

fn preview_resource_attach_queue_probe(
    queue: usize,
    resource_id: u32,
) -> PreviewResourceQueueProbe {
    let Some(vector) = read_usize_field(queue, 0x08).filter(|address| *address != 0) else {
        return PreviewResourceQueueProbe {
            len: usize::MAX,
            entries: Vec::new(),
        };
    };
    let Some(begin) = read_usize_field(vector, 0x08) else {
        return PreviewResourceQueueProbe {
            len: usize::MAX,
            entries: Vec::new(),
        };
    };
    let Some(end) = read_usize_field(vector, 0x10) else {
        return PreviewResourceQueueProbe {
            len: usize::MAX,
            entries: Vec::new(),
        };
    };
    let Some(len) = end.checked_sub(begin).map(|bytes| bytes / size_of::<u32>()) else {
        return PreviewResourceQueueProbe {
            len: usize::MAX,
            entries: Vec::new(),
        };
    };
    if len > 64 {
        return PreviewResourceQueueProbe {
            len,
            entries: Vec::new(),
        };
    }

    let mut entries = Vec::new();
    for index in 0..len {
        let Some(slot_addr) = begin.checked_add(index.saturating_mul(size_of::<u32>())) else {
            break;
        };
        if read_u32_field(slot_addr, 0) == Some(resource_id) {
            entries.push(PreviewResourceQueueEntryProbe {
                index,
                slot_addr,
                id: resource_id,
            });
        }
    }

    PreviewResourceQueueProbe { len, entries }
}

fn write_u32_field_checked(base: usize, offset: usize, value: u32) -> bool {
    let Some(address) = base.checked_add(offset) else {
        return false;
    };
    let Some(old_protect) = (unsafe { win::protect_memory(address, size_of::<u32>(), 0x04) })
    else {
        return false;
    };

    unsafe {
        std::ptr::write_unaligned(address as *mut u32, value);
    }

    let _ = unsafe { win::protect_memory(address, size_of::<u32>(), old_protect) };
    read_u32_field(base, offset) == Some(value)
}

fn write_usize_field_checked(base: usize, offset: usize, value: usize) -> bool {
    let Some(address) = base.checked_add(offset) else {
        return false;
    };
    let Some(old_protect) = (unsafe { win::protect_memory(address, size_of::<usize>(), 0x04) })
    else {
        return false;
    };

    unsafe {
        std::ptr::write_unaligned(address as *mut usize, value);
    }

    let _ = unsafe { win::protect_memory(address, size_of::<usize>(), old_protect) };
    read_usize_field(base, offset) == Some(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreviewResourceResolveMatch {
    table_base: usize,
    entry_index: usize,
    entry_ptr: usize,
    state: Option<u32>,
    flags: Option<u32>,
    extra: Option<u32>,
    marker: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TrackedPreviewResourceStates {
    table: usize,
    resource_911: Option<PreviewResourceResolveMatch>,
    resource_643: Option<PreviewResourceResolveMatch>,
    resource_1957: Option<PreviewResourceResolveMatch>,
}

const PREVIEW_RESOURCE_SLOT_COUNT: usize = 0x521;
const PREVIEW_RESOURCE_SLOT_STRIDE: usize = 0x1d0;
const PREVIEW_RESOURCE_SLOT_BASE_OFFSET: usize = 0x10;
const PREVIEW_RESOURCE_SLOT_ID_OFFSET: usize = 0x48;
const PREVIEW_RESOURCE_SLOT_STATE_OFFSET: usize = 0x1c0;
const PREVIEW_RESOURCE_SLOT_FLAGS_OFFSET: usize = 0x1c4;
const PREVIEW_RESOURCE_SLOT_EXTRA_OFFSET: usize = 0x1c8;
const PREVIEW_RESOURCE_SLOT_MARKER_OFFSET: usize = 0x1cc;
const PREVIEW_RESOURCE_LOOKUP_SCAN_TRACKED_IDS: [u32; 4] = [
    LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID,
    LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
    LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID,
    LAW_EXTRA_SLOT_PREVIEW_TABLE_ONI_FALLBACK_SOURCE_ID,
];

fn preview_resource_resolve_match(
    table: usize,
    mapped_resource_id: u32,
) -> Option<PreviewResourceResolveMatch> {
    if table == 0 {
        return None;
    }

    for index in 0..PREVIEW_RESOURCE_SLOT_COUNT {
        let entry = table
            .checked_add(PREVIEW_RESOURCE_SLOT_BASE_OFFSET)?
            .checked_add(index.saturating_mul(PREVIEW_RESOURCE_SLOT_STRIDE))?;
        if read_u32_field(entry, PREVIEW_RESOURCE_SLOT_ID_OFFSET) == Some(mapped_resource_id) {
            return Some(PreviewResourceResolveMatch {
                table_base: table,
                entry_index: index,
                entry_ptr: entry,
                state: read_u32_field(entry, PREVIEW_RESOURCE_SLOT_STATE_OFFSET),
                flags: read_u32_field(entry, PREVIEW_RESOURCE_SLOT_FLAGS_OFFSET),
                extra: read_u32_field(entry, PREVIEW_RESOURCE_SLOT_EXTRA_OFFSET),
                marker: read_u32_field(entry, PREVIEW_RESOURCE_SLOT_MARKER_OFFSET),
            });
        }
    }

    None
}

fn preview_resource_active_duplicate(
    table: usize,
    mapped_resource_id: u32,
    selected_entry_ptr: Option<usize>,
) -> String {
    if table == 0 {
        return "none".to_string();
    }

    let mut matches = Vec::new();
    for index in 0..PREVIEW_RESOURCE_SLOT_COUNT {
        let Some(entry) = table
            .checked_add(PREVIEW_RESOURCE_SLOT_BASE_OFFSET)
            .and_then(|entry| {
                entry.checked_add(index.saturating_mul(PREVIEW_RESOURCE_SLOT_STRIDE))
            })
        else {
            break;
        };
        if Some(mapped_resource_id) != read_u32_field(entry, PREVIEW_RESOURCE_SLOT_ID_OFFSET) {
            continue;
        }
        if selected_entry_ptr == Some(entry) {
            continue;
        }
        if read_u32_field(entry, PREVIEW_RESOURCE_SLOT_STATE_OFFSET) == Some(1)
            && read_u32_field(entry, PREVIEW_RESOURCE_SLOT_MARKER_OFFSET) == Some(0)
        {
            matches.push(format!("index:{index} entry=0x{entry:x}"));
            if matches.len() >= 4 {
                break;
            }
        }
    }

    if matches.is_empty() {
        "none".to_string()
    } else {
        matches.join(",")
    }
}

fn preview_resource_lookup_scan_context(
    table: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> String {
    let timeline_context = preview_resource_trace_context(
        last_law_ready_timeline_trace_label().map(|(_, trace)| trace),
    );
    format!(
        "{} lookup=[table=0x{table:x} arg2=0x{arg2:x} arg3=0x{arg3:x} arg4=0x{arg4:x} direct_ids=[{}] scan_slots=[{}] tracked_entries=[{}]]",
        timeline_context,
        format_preview_resource_lookup_scan_direct_ids(table, arg2, arg3, arg4),
        format_preview_resource_lookup_scan_internal_slots(table),
        format_preview_resource_lookup_scan_tracked_entries(table)
    )
}

fn format_preview_resource_lookup_scan_direct_ids(
    table: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> String {
    let values = [
        ("arg2", u32::try_from(arg2).ok()),
        ("arg3", u32::try_from(arg3).ok()),
        ("arg4", u32::try_from(arg4).ok()),
        (
            "slot0_id",
            table
                .checked_add(PREVIEW_RESOURCE_SLOT_BASE_OFFSET)
                .and_then(|entry| read_u32_field(entry, PREVIEW_RESOURCE_SLOT_ID_OFFSET)),
        ),
        (
            "slot1_id",
            table
                .checked_add(PREVIEW_RESOURCE_SLOT_BASE_OFFSET)
                .and_then(|entry| entry.checked_add(PREVIEW_RESOURCE_SLOT_STRIDE))
                .and_then(|entry| read_u32_field(entry, PREVIEW_RESOURCE_SLOT_ID_OFFSET)),
        ),
    ];
    values
        .iter()
        .map(|(label, value)| {
            let classification = (*value)
                .map(preview_resource_lookup_scan_id_label)
                .unwrap_or("none");
            format!("{label}={}:{}", format_optional_u32(*value), classification)
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_preview_resource_lookup_scan_internal_slots(table: usize) -> String {
    const SCAN_SLOT_BASE_OFFSET: usize = 0xb8;
    const SCAN_SLOT_STRIDE: usize = 0xa0;
    if table == 0 {
        return "table=0".to_string();
    }
    (0..4usize)
        .map(|index| {
            let entry = table
                .checked_add(SCAN_SLOT_BASE_OFFSET)
                .and_then(|entry| entry.checked_add(index.saturating_mul(SCAN_SLOT_STRIDE)));
            let value = entry.and_then(|entry| read_u32_field(entry, 0));
            let classification = value
                .map(preview_resource_lookup_scan_id_label)
                .unwrap_or("none");
            format!(
                "slot{index}@+0x{:x}={}:{}",
                SCAN_SLOT_BASE_OFFSET + index.saturating_mul(SCAN_SLOT_STRIDE),
                format_optional_u32(value),
                classification
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn preview_resource_lookup_scan_id_label(value: u32) -> &'static str {
    match value {
        LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID => "base643",
        LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID => "oni911",
        LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID => "model292",
        LAW_EXTRA_SLOT_PREVIEW_TABLE_ONI_FALLBACK_SOURCE_ID => "oni308",
        _ => "other",
    }
}

fn format_preview_resource_lookup_scan_tracked_entries(table: usize) -> String {
    if table == 0 {
        return "table=0".to_string();
    }

    let mut entries = Vec::new();
    for resource_id in PREVIEW_RESOURCE_LOOKUP_SCAN_TRACKED_IDS {
        if let Some(entry) = preview_resource_resolve_match(table, resource_id) {
            entries.push(format!(
                "{}:{}",
                resource_id,
                format_preview_resource_match_state(Some(entry))
            ));
        } else {
            entries.push(format!("{resource_id}:none"));
        }
    }
    entries.join(";")
}

fn read_tracked_preview_resource_states() -> TrackedPreviewResourceStates {
    let table = LAST_PREVIEW_RESOURCE_TABLE.load(Ordering::Relaxed);
    TrackedPreviewResourceStates {
        table,
        resource_911: preview_resource_resolve_match(
            table,
            LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
        ),
        resource_643: preview_resource_resolve_match(
            table,
            LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID,
        ),
        resource_1957: preview_resource_resolve_match(table, 1957),
    }
}

fn preview_diff_label_code(label: Option<&str>) -> usize {
    match label {
        Some("slot3-oni") => 1,
        Some("slot5-custom") => 2,
        _ => 0,
    }
}

fn format_preview_diff_label_code(code: usize) -> &'static str {
    match code {
        1 => "slot3-oni",
        2 => "slot5-custom",
        _ => "other",
    }
}

fn preview_resource_flow_label() -> (&'static str, Option<CostumeObjectUpdateTrace>) {
    if let Some((label, trace)) = last_law_ready_timeline_trace_label() {
        return (label, Some(trace));
    }
    ("other", None)
}

fn preview_resource_trace_context(trace: Option<CostumeObjectUpdateTrace>) -> String {
    if let Some(trace) = trace {
        return format!(
            "selected_variant={} selected_slot={} active_layout={} selected_model_resource={} mapped294={}",
            format_optional_u16_decimal(trace.selected_variant),
            format_optional_u32(trace.selected_slot),
            format_optional_u32(trace.selected_layout.or(trace.cached_layout)),
            format_optional_u16_decimal(trace.selected_model_resource),
            format_optional_u32(trace.selected_preview_mapped_resource)
        );
    }

    format_last_costume_object_update_context(
        LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed),
    )
}

fn format_preview_resource_caller_rva(frames: &[usize]) -> String {
    frames
        .iter()
        .copied()
        .filter_map(game_frame_rva)
        .find(|rva| {
            matches!(
                *rva,
                0x1491c5f
                    | 0x1490b95
                    | 0x14928f3
                    | 0x14916bb
                    | 0x149169b
                    | 0x149025e
                    | 0x1491c7f
                    | 0x1491d2b
                    | 0x1490acd
                    | 0x1490bfe
            )
        })
        .or_else(|| frames.iter().copied().find_map(game_frame_rva))
        .map(|rva| format!("game+0x{rva:x}"))
        .unwrap_or_else(|| "none".to_string())
}

fn known_preview_resource_callers(frames: &[usize]) -> Vec<usize> {
    let mut callers = Vec::new();
    for rva in frames.iter().copied().filter_map(game_frame_rva) {
        if matches!(
            rva,
            0x1491c5f
                | 0x1490b95
                | 0x14928f3
                | 0x14916bb
                | 0x149169b
                | 0x149025e
                | 0x1491c7f
                | 0x1491d2b
                | 0x1490acd
                | 0x1490bfe
        ) && !callers.contains(&rva)
        {
            callers.push(rva);
        }
    }
    callers
}

fn format_known_preview_resource_callers(frames: &[usize]) -> String {
    let callers = known_preview_resource_callers(frames);
    if callers.is_empty() {
        return "none".to_string();
    }
    callers
        .iter()
        .map(|rva| format!("game+0x{rva:x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn preview_resource_callsite_label(caller_rva: &str) -> &'static str {
    match caller_rva {
        "game+0x1491c5f" => "oni-normal",
        "game+0x1490b95" => "slot5-suspect",
        "game+0x14928f3" => "base-fallback-643",
        "game+0x149025e" => "scene-present-first-resolver",
        "game+0x1491c7f" => "scene-dispatch-present",
        "game+0x1491d2b" => "scene-dispatch-late-resolver",
        "game+0x1490acd" => "scene-apply-fallback-source",
        "game+0x1490bfe" => "scene-apply-present-after",
        _ => "unknown",
    }
}

fn format_preview_resource_match_state(entry: Option<PreviewResourceResolveMatch>) -> String {
    if let Some(entry) = entry {
        format!(
            "table_base=0x{:x} entry_index={} entry_ptr=0x{:x} state={} flags={} extra={} marker={}",
            entry.table_base,
            entry.entry_index,
            entry.entry_ptr,
            format_optional_u32(entry.state),
            format_optional_hex_u32(entry.flags),
            format_optional_hex_u32(entry.extra),
            format_optional_hex_u32(entry.marker)
        )
    } else {
        "table_base=none entry_index=none entry_ptr=none state=none flags=none extra=none marker=none"
            .to_string()
    }
}

fn format_preview_resource_entry_payload(entry: Option<PreviewResourceResolveMatch>) -> String {
    let Some(entry) = entry else {
        return "entry=none".to_string();
    };
    format!(
        "entry=0x{:x} q00={} q08={} q10={} q18={} q20={} q28={} q30={} q38={} q40={} id={} state={} flags={} extra={} marker={}",
        entry.entry_ptr,
        format_optional_address(read_usize_field(entry.entry_ptr, 0x00)),
        format_optional_address(read_usize_field(entry.entry_ptr, 0x08)),
        format_optional_address(read_usize_field(entry.entry_ptr, 0x10)),
        format_optional_address(read_usize_field(entry.entry_ptr, 0x18)),
        format_optional_address(read_usize_field(entry.entry_ptr, 0x20)),
        format_optional_address(read_usize_field(entry.entry_ptr, 0x28)),
        format_optional_address(read_usize_field(entry.entry_ptr, 0x30)),
        format_optional_address(read_usize_field(entry.entry_ptr, 0x38)),
        format_optional_address(read_usize_field(entry.entry_ptr, 0x40)),
        format_optional_u32(read_u32_field(entry.entry_ptr, PREVIEW_RESOURCE_SLOT_ID_OFFSET)),
        format_optional_u32(entry.state),
        format_optional_hex_u32(entry.flags),
        format_optional_hex_u32(entry.extra),
        format_optional_hex_u32(entry.marker)
    )
}

fn format_tracked_preview_resource_payloads(states: TrackedPreviewResourceStates) -> String {
    format!(
        "payload911=[{}] payload643=[{}] payload1957=[{}]",
        format_preview_resource_entry_payload(states.resource_911),
        format_preview_resource_entry_payload(states.resource_643),
        format_preview_resource_entry_payload(states.resource_1957)
    )
}

fn format_preview_resource_queue_state(queue: Option<(usize, bool)>) -> String {
    if let Some((len, has_id)) = queue {
        format!(
            "attach_len={} attach_has_id={}",
            if len == usize::MAX {
                "none".to_string()
            } else {
                len.to_string()
            },
            has_id
        )
    } else {
        "attach_len=none attach_has_id=false".to_string()
    }
}

fn preview_resource_queue_status(
    before: Option<&PreviewResourceQueueProbe>,
    after: Option<&PreviewResourceQueueProbe>,
    after_entry: Option<PreviewResourceResolveMatch>,
) -> &'static str {
    let before_has = before.is_some_and(|probe| !probe.entries.is_empty());
    let after_has = after.is_some_and(|probe| !probe.entries.is_empty());
    if !before_has && after_has {
        return "queue_added";
    }
    if !before_has && !after_has {
        return "queue_missing";
    }

    if after_entry.is_some_and(|entry| entry.state == Some(1) && entry.marker == Some(0)) {
        "queue_already_present_active"
    } else if after_entry.is_some_and(|entry| entry.state == Some(0) && entry.marker == Some(1)) {
        "queue_already_present_stale"
    } else {
        "queue_already_present_unknown"
    }
}

fn format_preview_resource_queue_probe(probe: Option<&PreviewResourceQueueProbe>) -> String {
    let Some(probe) = probe else {
        return "attach_len=none entries=none".to_string();
    };
    let len = if probe.len == usize::MAX {
        "none".to_string()
    } else {
        probe.len.to_string()
    };
    if probe.entries.is_empty() {
        return format!("attach_len={len} entries=none");
    }
    let entries = probe
        .entries
        .iter()
        .map(|entry| {
            format!(
                "index={} slot=0x{:x} id={}",
                entry.index, entry.slot_addr, entry.id
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    format!("attach_len={len} entries=[{entries}]")
}

fn format_preview_resource_queue_first_index(probe: Option<&PreviewResourceQueueProbe>) -> String {
    probe
        .and_then(|probe| probe.entries.first())
        .map(|entry| entry.index.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn stale_preview_911_reactivation_skip_reason(
    mapped_resource_id: u32,
    caller_rva: &str,
    trace: Option<CostumeObjectUpdateTrace>,
    queue_status: &str,
    after: Option<PreviewResourceResolveMatch>,
) -> Option<&'static str> {
    if !LAW_SLOT5_REACTIVATE_STALE_PREVIEW_911_ENABLED {
        return Some("flag_disabled");
    }
    if mapped_resource_id != LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID {
        return Some("resource_not_911");
    }
    if !caller_rva.contains("0x1490b95") {
        return Some("caller_not_slot5_attach");
    }
    let Some(trace) = trace else {
        return Some("missing_trace");
    };
    if trace.selected_variant != current_law_extra_slot_probe_variant_id() {
        return Some("variant_mismatch");
    }
    if trace.selected_slot != Some(4) {
        return Some("slot_mismatch");
    }
    if trace.selected_model_resource != Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID) {
        return Some("model_mismatch");
    }
    if trace.selected_preview_mapped_resource != Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    {
        return Some("mapped_resource_mismatch");
    }
    if queue_status != "queue_already_present_stale" {
        return Some("queue_not_stale");
    }
    let Some(entry) = after else {
        return Some("missing_entry");
    };
    if read_u32_field(entry.entry_ptr, PREVIEW_RESOURCE_SLOT_ID_OFFSET)
        != Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    {
        return Some("entry_id_mismatch");
    }
    if entry.state != Some(0) || entry.marker != Some(1) {
        return Some("entry_not_stale");
    }
    None
}

fn maybe_reactivate_stale_preview_911(
    mapped_resource_id: u32,
    caller_rva: &str,
    trace: Option<CostumeObjectUpdateTrace>,
    before_queue_probe: Option<&PreviewResourceQueueProbe>,
    after_queue_probe: Option<&PreviewResourceQueueProbe>,
    after_911: Option<PreviewResourceResolveMatch>,
    frames: &[usize],
) -> Option<PreviewResourceResolveMatch> {
    let queue_status =
        preview_resource_queue_status(before_queue_probe, after_queue_probe, after_911);
    poll_slot5_pre_controller_loss_window(caller_rva, "resource911-reactivate", trace, frames);
    poll_slot5_controller_state(caller_rva, "resource911-reactivate", trace, frames);
    let skip_reason = stale_preview_911_reactivation_skip_reason(
        mapped_resource_id,
        caller_rva,
        trace,
        queue_status,
        after_911,
    );

    if skip_reason.is_some() {
        if mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
            && (caller_rva.contains("0x1490b95")
                || trace.is_some_and(|trace| {
                    trace.selected_variant == current_law_extra_slot_probe_variant_id()
                }))
            && COSTUME_PREVIEW_RESOURCE_REACTIVATE_LOGS.fetch_add(1, Ordering::Relaxed)
                < MAX_COSTUME_PREVIEW_RESOURCE_REACTIVATE_LOGS
        {
            log::write_line(format!(
                "resource911-stale-queued-reactivate-skip reason={} caller={} callsite={} queue_status={} before_queue=[{}] after_queue=[{}] entry=[{}] context=[{}] frames={}",
                skip_reason.unwrap_or("unknown"),
                caller_rva,
                preview_resource_callsite_label(caller_rva),
                queue_status,
                format_preview_resource_queue_probe(before_queue_probe),
                format_preview_resource_queue_probe(after_queue_probe),
                format_preview_resource_match_state(after_911),
                preview_resource_trace_context(trace),
                format_stack_frames(frames, 7)
            ));
        }
        return after_911;
    }

    let before = after_911?;
    let state_written =
        write_u32_field_checked(before.entry_ptr, PREVIEW_RESOURCE_SLOT_STATE_OFFSET, 1);
    let marker_written =
        write_u32_field_checked(before.entry_ptr, PREVIEW_RESOURCE_SLOT_MARKER_OFFSET, 0);
    let after = preview_resource_resolve_match(
        before.table_base,
        LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
    );

    if COSTUME_PREVIEW_RESOURCE_REACTIVATE_LOGS.fetch_add(1, Ordering::Relaxed)
        < MAX_COSTUME_PREVIEW_RESOURCE_REACTIVATE_LOGS
    {
        log::write_line(format!(
            "resource911-stale-queued-reactivate caller={} callsite={} state_written={} marker_written={} before=[{}] after=[{}] before_queue=[{}] after_queue=[{}] context=[{}] frames={}",
            caller_rva,
            preview_resource_callsite_label(caller_rva),
            state_written,
            marker_written,
            format_preview_resource_match_state(Some(before)),
            format_preview_resource_match_state(after),
            format_preview_resource_queue_probe(before_queue_probe),
            format_preview_resource_queue_probe(after_queue_probe),
            preview_resource_trace_context(trace),
            format_stack_frames(frames, 7)
        ));
    }

    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    maybe_run_slot5_refresh_after_911_ready_with_checkpoint(
        "resource911-stale-queued-reactivate",
        state,
        trace,
    );

    after
}

fn format_tracked_preview_resource_states(states: Option<TrackedPreviewResourceStates>) -> String {
    if let Some(states) = states {
        format!(
            "table=0x{:x} resource911=[{}] resource643=[{}] resource1957=[{}]",
            states.table,
            format_preview_resource_match_state(states.resource_911),
            format_preview_resource_match_state(states.resource_643),
            format_preview_resource_match_state(states.resource_1957)
        )
    } else {
        "table=none resource911=[table_base=none entry_index=none entry_ptr=none state=none flags=none extra=none marker=none] resource643=[table_base=none entry_index=none entry_ptr=none state=none flags=none extra=none marker=none] resource1957=[table_base=none entry_index=none entry_ptr=none state=none flags=none extra=none marker=none]".to_string()
    }
}

fn format_preview_widget_resource_update_state(widget: usize) -> String {
    let mapped294 = read_u32_field(widget, 0x294);
    let queue = read_usize_field(widget, 0x288).filter(|address| *address != 0);
    let child58 = read_usize_field(widget, 0x58).filter(|address| *address != 0);
    let child_b4 = child58.and_then(|address| read_u32_field(address, 0xb4));
    let queue_text = queue
        .map(format_preview_resource_attach_queue)
        .unwrap_or_else(|| "none".to_string());
    format!(
        "mapped294={} queue={} queue_state=[{}] child58={} child_b4={}",
        format_optional_u32(mapped294),
        format_optional_address(queue),
        queue_text,
        format_optional_address(child58),
        format_optional_u32(child_b4)
    )
}

fn preview_widget_resource_update_interesting(
    before_state: &str,
    after_state: &str,
    before_resources: TrackedPreviewResourceStates,
    after_resources: TrackedPreviewResourceStates,
    before_context: &str,
    after_context: &str,
) -> bool {
    before_resources != after_resources
        || before_state.contains("mapped294=911")
        || after_state.contains("mapped294=911")
        || before_state.contains("mapped294=643")
        || after_state.contains("mapped294=643")
        || before_context.contains("selected_variant=699")
        || after_context.contains("selected_variant=699")
        || before_context.contains("selected_slot=4")
        || after_context.contains("selected_slot=4")
}

fn log_preview_widget_resource_update(
    widget: usize,
    before_state: String,
    after_state: String,
    before_resources: TrackedPreviewResourceStates,
    after_resources: TrackedPreviewResourceStates,
    before_context: String,
    after_context: String,
) {
    if !preview_widget_resource_update_interesting(
        &before_state,
        &after_state,
        before_resources,
        after_resources,
        &before_context,
        &after_context,
    ) {
        return;
    }
    let index = COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_WIDGET_RESOURCE_UPDATE_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "preview-widget-resource-update boundary=FUN_14148b800 widget=0x{widget:x} before=[{}] after=[{}] resource_before=[{}] resource_after=[{}] context_before=[{}] context_after=[{}] frames={}",
        before_state,
        after_state,
        format_tracked_preview_resource_states(Some(before_resources)),
        format_tracked_preview_resource_states(Some(after_resources)),
        before_context,
        after_context,
        format_stack_frames(&frames, 8)
    ));
}

fn log_preview_page_reset_callsite(
    widget: usize,
    before_state: String,
    after_state: String,
    before_resources: TrackedPreviewResourceStates,
    after_resources: TrackedPreviewResourceStates,
    before_context: String,
    after_context: String,
) {
    if !preview_widget_resource_update_interesting(
        &before_state,
        &after_state,
        before_resources,
        after_resources,
        &before_context,
        &after_context,
    ) {
        return;
    }
    let index = COSTUME_PREVIEW_PAGE_RESET_CALLSITE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_PAGE_RESET_CALLSITE_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "preview-page-reset-callsite callsite=game+0x{:x} target=FUN_141488570 widget=0x{widget:x} before=[{}] after=[{}] resource_before=[{}] resource_after=[{}] context_before=[{}] context_after=[{}] frames={}",
        COSTUME_PREVIEW_PAGE_RESET_CALLSITE_RVA,
        before_state,
        after_state,
        format_tracked_preview_resource_states(Some(before_resources)),
        format_tracked_preview_resource_states(Some(after_resources)),
        before_context,
        after_context,
        format_stack_frames(&frames, 8)
    ));
}

fn log_preview_page_reset_tailjump(
    widget: usize,
    before_resources: TrackedPreviewResourceStates,
    after_resources: TrackedPreviewResourceStates,
    before_context: String,
    after_context: String,
) {
    if before_resources == after_resources
        && !before_context.contains("selected_variant=699")
        && !after_context.contains("selected_variant=699")
        && !before_context.contains("mapped294=911")
        && !after_context.contains("mapped294=911")
    {
        return;
    }
    let index = COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "preview-page-reset-tailjump callsite=game+0x{:x} target=FUN_141488570 widget=0x{widget:x} resource_before=[{}] resource_after=[{}] context_before=[{}] context_after=[{}] frames={}",
        COSTUME_PREVIEW_PAGE_RESET_TAILJUMP_RVA,
        format_tracked_preview_resource_states(Some(before_resources)),
        format_tracked_preview_resource_states(Some(after_resources)),
        before_context,
        after_context,
        format_stack_frames(&frames, 8)
    ));
}

fn log_preview_resource_candidate_update(
    widget: usize,
    result: usize,
    before_resources: TrackedPreviewResourceStates,
    after_resources: TrackedPreviewResourceStates,
    before_context: String,
    after_context: String,
) {
    if before_resources == after_resources
        && !before_context.contains("selected_variant=699")
        && !after_context.contains("selected_variant=699")
        && !before_context.contains("mapped294=911")
        && !after_context.contains("mapped294=911")
    {
        return;
    }
    let index = COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_CANDIDATE_UPDATE_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "preview-resource-candidate-update boundary=FUN_141488610 widget=0x{widget:x} result=0x{result:x} resource_before=[{}] resource_after=[{}] context_before=[{}] context_after=[{}] frames={}",
        format_tracked_preview_resource_states(Some(before_resources)),
        format_tracked_preview_resource_states(Some(after_resources)),
        before_context,
        after_context,
        format_stack_frames(&frames, 8)
    ));
}

fn log_costume_preview_resource_lookup_scan(
    table: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    result: usize,
    before_resources: TrackedPreviewResourceStates,
    after_resources: TrackedPreviewResourceStates,
    before_context: String,
    after_context: String,
) {
    if before_resources == after_resources
        && !before_context.contains("selected_variant=699")
        && !after_context.contains("selected_variant=699")
    {
        return;
    }
    if before_resources == after_resources
        && !before_context.contains("selected_variant=699")
        && !after_context.contains("selected_variant=699")
        && !before_context.contains("mapped294=911")
        && !after_context.contains("mapped294=911")
        && !before_context.contains("resource911")
        && !after_context.contains("resource911")
    {
        return;
    }
    let index = COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_LOOKUP_SCAN_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "preview-resource-lookup-scan boundary=FUN_141582c30_contains_1582c6d table=0x{table:x} arg2=0x{arg2:x} arg3=0x{arg3:x} arg4=0x{arg4:x} result=0x{result:x} resource_before=[{}] resource_after=[{}] payload_before=[{}] payload_after=[{}] context_before=[{}] context_after=[{}] frames={}",
        format_tracked_preview_resource_states(Some(before_resources)),
        format_tracked_preview_resource_states(Some(after_resources)),
        format_tracked_preview_resource_payloads(before_resources),
        format_tracked_preview_resource_payloads(after_resources),
        before_context,
        after_context,
        format_stack_frames(&frames, 8)
    ));
}

fn scene_trace_selected_model_resource(trace: Option<CostumeSceneTrace>) -> Option<u16> {
    trace
        .and_then(|trace| trace.selected_variant)
        .and_then(read_variant_model_resource)
}

fn scene_trace_selected_preview_mapping(trace: Option<CostumeSceneTrace>) -> Option<u16> {
    trace
        .and_then(|trace| trace.selected_variant)
        .and_then(read_variant_preview_mapping)
}

fn scene_trace_mapped294(trace: Option<CostumeSceneTrace>) -> Option<u32> {
    scene_trace_selected_preview_mapping(trace).map(preview_model_mapped_id_from_metadata_value)
}

fn format_scene_boundary_flags(state: usize) -> String {
    if state == 0 {
        return "446=none 447=none 449=none 44c=none 44f=none 450=none 451=none".to_string();
    }
    format!(
        "446={} 447={} 449={} 44c={} 44f={} 450={} 451={}",
        format_optional_u8(read_u8_field(state, 0x446)),
        format_optional_u8(read_u8_field(state, 0x447)),
        format_optional_u8(read_u8_field(state, 0x449)),
        format_optional_u8(read_u8_field(state, 0x44c)),
        format_optional_u8(read_u8_field(state, 0x44f)),
        format_optional_u8(read_u8_field(state, 0x450)),
        format_optional_u8(read_u8_field(state, 0x451))
    )
}

fn format_scene_boundary_context(state: usize, trace: Option<CostumeSceneTrace>) -> String {
    format!(
        "state=0x{state:x} selected_variant={} selected_slot={} selected_layout={} selected_model_resource={} mapped294={} flags=[{}] trace=[{}]",
        format_optional_u16_decimal(trace.and_then(|trace| trace.selected_variant)),
        format_optional_u32(trace.and_then(|trace| trace.selected_slot)),
        format_optional_u32(trace.and_then(|trace| trace.layout)),
        format_optional_u16_decimal(scene_trace_selected_model_resource(trace)),
        format_optional_u32(scene_trace_mapped294(trace)),
        format_scene_boundary_flags(state),
        trace
            .map(format_costume_scene_trace)
            .unwrap_or_else(|| "none".to_string())
    )
}

fn preview_resource_911_hidden_reset(
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
) -> bool {
    before.is_some_and(|entry| entry.state == Some(1) && entry.marker == Some(0))
        && after.is_some_and(|entry| entry.state == Some(0) && entry.marker == Some(1))
}

fn preview_resource_911_reactivation(
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
) -> bool {
    before.is_some_and(|entry| entry.state == Some(0) && entry.marker == Some(1))
        && after.is_some_and(|entry| entry.state == Some(1) && entry.marker == Some(0))
}

fn format_resource_911_activation_attempt(
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
) -> &'static str {
    if preview_resource_911_reactivation(before, after) {
        "reactivated"
    } else if after.is_some_and(|entry| entry.state == Some(1) && entry.marker == Some(0)) {
        "active"
    } else if after.is_some_and(|entry| entry.state == Some(0) && entry.marker == Some(1)) {
        "stale_marked"
    } else if before != after {
        "changed_other"
    } else {
        "unchanged"
    }
}

fn preview_resource_911_path_mode(
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
) -> &'static str {
    match (before, after) {
        (None, Some(entry)) if entry.state == Some(1) && entry.marker == Some(0) => {
            "create_active_from_none"
        }
        (Some(before), Some(after))
            if before.state == Some(0)
                && before.marker == Some(1)
                && after.state == Some(0)
                && after.marker == Some(1) =>
        {
            "resolve_stale_existing"
        }
        (Some(before), Some(after))
            if before.state == Some(0)
                && before.marker == Some(1)
                && after.state == Some(1)
                && after.marker == Some(0) =>
        {
            "reactivate_stale_existing"
        }
        (Some(before), Some(after))
            if before.state == Some(1)
                && before.marker == Some(0)
                && after.state == Some(1)
                && after.marker == Some(0) =>
        {
            "resolve_active_existing"
        }
        (None, None) => "missing",
        (None, Some(_)) => "create_other_from_none",
        (Some(_), None) => "lost_existing",
        (Some(_), Some(_)) => "resolve_other_existing",
    }
}

fn preview_resource_911_timeline_source(
    caller_rva: &str,
    label: &str,
    trace: Option<CostumeObjectUpdateTrace>,
    context: &str,
) -> &'static str {
    if caller_rva.contains("0x1490b95")
        || label == "slot5-custom"
        || trace.is_some_and(|trace| {
            trace.selected_variant == current_law_extra_slot_probe_variant_id()
        })
        || context.contains("selected_variant=699")
        || context.contains("selected_slot=4")
    {
        "slot5-path"
    } else if caller_rva.contains("0x1491c5f")
        || trace.is_some_and(|trace| {
            trace.selected_variant == Some(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID)
        })
        || context.contains("selected_variant=586")
        || context.contains("selected_slot=3")
    {
        "oni-path"
    } else if caller_rva.contains("game+") || !context.is_empty() {
        "global"
    } else {
        "unknown"
    }
}

fn log_resource_911_reactivation(
    phase: &str,
    caller_rva: &str,
    label: &str,
    preview_arg: u32,
    trace: Option<CostumeObjectUpdateTrace>,
    context: &str,
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
    frames: &[usize],
) {
    if !preview_resource_911_reactivation(before, after) {
        return;
    }
    if RESOURCE_911_REACTIVATION_LOGS.fetch_add(1, Ordering::Relaxed)
        >= RESOURCE_911_REACTIVATION_LOG_CAP
    {
        return;
    }
    let source = preview_resource_911_timeline_source(caller_rva, label, trace, context);
    log::write_line(format!(
        "resource911-reactivation phase={phase} source={source} label={label} caller_rva={caller_rva} callsite={} preview_arg={preview_arg} before=[{}] after=[{}] context=[{}] known_callers={} frames={}",
        preview_resource_callsite_label(caller_rva),
        format_preview_resource_match_state(before),
        format_preview_resource_match_state(after),
        context,
        format_known_preview_resource_callers(frames),
        format_stack_frames(frames, 7)
    ));
}

struct PreviewResource911PathArgs<'a> {
    op: &'a str,
    caller_rva: &'a str,
    label: &'a str,
    mapped_resource_id: u32,
    render_context: u32,
    widget_context: i32,
    param5: Option<i32>,
    result: usize,
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
    before_queue: Option<(usize, bool)>,
    after_queue: Option<(usize, bool)>,
    before_queue_probe: Option<&'a PreviewResourceQueueProbe>,
    after_queue_probe: Option<&'a PreviewResourceQueueProbe>,
    trace: Option<CostumeObjectUpdateTrace>,
    frames: &'a [usize],
}

fn log_preview_resource_911_path(args: PreviewResource911PathArgs<'_>) {
    if args.mapped_resource_id != LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID {
        return;
    }
    let line_label = if args.caller_rva.contains("0x1491c5f") {
        "oni-911-path"
    } else if args.caller_rva.contains("0x1490b95") || args.label == "slot5-custom" {
        "slot5-911-path"
    } else {
        return;
    };
    if COSTUME_PREVIEW_RESOURCE_PATH_LOGS.fetch_add(1, Ordering::Relaxed)
        >= MAX_COSTUME_PREVIEW_RESOURCE_PATH_LOGS
    {
        return;
    }

    let table = args
        .after
        .or(args.before)
        .map(|entry| entry.table_base)
        .unwrap_or_else(|| LAST_PREVIEW_RESOURCE_TABLE.load(Ordering::Relaxed));
    let selected_entry_ptr = args.after.or(args.before).map(|entry| entry.entry_ptr);
    let active_duplicate =
        preview_resource_active_duplicate(table, args.mapped_resource_id, selected_entry_ptr);
    let mode = preview_resource_911_path_mode(args.before, args.after);
    let activation = format_resource_911_activation_attempt(args.before, args.after);
    let queue_status =
        preview_resource_queue_status(args.before_queue_probe, args.after_queue_probe, args.after);
    let context = preview_resource_trace_context(args.trace);
    log::write_line(format!(
        "{line_label} op={} caller={} callsite={} mode={mode} activation911={activation} queue_status={queue_status} table=0x{table:x} mapped_resource={} render_context={} widget_context={} param5={} result=0x{:x} before=[{}] after=[{}] before_payload=[{}] after_payload=[{}] before_queue=[{}] after_queue=[{}] before_queue_detail=[{}] after_queue_detail=[{}] active_duplicate_911={} context=[{}] known_callers={} frames={}",
        args.op,
        args.caller_rva,
        preview_resource_callsite_label(args.caller_rva),
        args.mapped_resource_id,
        args.render_context,
        args.widget_context,
        args.param5
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        args.result,
        format_preview_resource_match_state(args.before),
        format_preview_resource_match_state(args.after),
        format_preview_resource_entry_payload(args.before),
        format_preview_resource_entry_payload(args.after),
        format_preview_resource_queue_state(args.before_queue),
        format_preview_resource_queue_state(args.after_queue),
        format_preview_resource_queue_probe(args.before_queue_probe),
        format_preview_resource_queue_probe(args.after_queue_probe),
        active_duplicate,
        context,
        format_known_preview_resource_callers(args.frames),
        format_stack_frames(args.frames, 7)
    ));
}

fn log_preview_resource_911_queue_state(args: &PreviewResource911PathArgs<'_>) {
    if args.op != "attach"
        || args.mapped_resource_id != LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
    {
        return;
    }
    let line_label = if args.caller_rva.contains("0x1491c5f") {
        "oni"
    } else if args.caller_rva.contains("0x1490b95") || args.label == "slot5-custom" {
        "slot5"
    } else {
        return;
    };
    if COSTUME_PREVIEW_RESOURCE_QUEUE_LOGS.fetch_add(1, Ordering::Relaxed)
        >= MAX_COSTUME_PREVIEW_RESOURCE_QUEUE_LOGS
    {
        return;
    }

    let mode = preview_resource_911_path_mode(args.before, args.after);
    let queue_status =
        preview_resource_queue_status(args.before_queue_probe, args.after_queue_probe, args.after);
    let after_has = args
        .after_queue_probe
        .is_some_and(|probe| !probe.entries.is_empty());
    let context = preview_resource_trace_context(args.trace);
    log::write_line(format!(
        "resource911-queue-state label={line_label} caller={} callsite={} mode={mode} queue_status={queue_status} entry_state={} entry_marker={} attach_len={} queue_index={} queue_has911={} before_queue=[{}] after_queue=[{}] context=[{}] frames={}",
        args.caller_rva,
        preview_resource_callsite_label(args.caller_rva),
        args.after
            .and_then(|entry| entry.state)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        args.after
            .and_then(|entry| entry.marker)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        args.after_queue_probe
            .map(|probe| {
                if probe.len == usize::MAX {
                    "none".to_string()
                } else {
                    probe.len.to_string()
                }
            })
            .unwrap_or_else(|| "none".to_string()),
        format_preview_resource_queue_first_index(args.after_queue_probe),
        after_has,
        format_preview_resource_queue_probe(args.before_queue_probe),
        format_preview_resource_queue_probe(args.after_queue_probe),
        context,
        format_stack_frames(args.frames, 7)
    ));

    if queue_status == "queue_already_present_stale" {
        log::write_line(format!(
            "resource911-stale-queued caller={} callsite={} selected_variant={} selected_slot={} model_resource={} mapped294={} queue_index={} entry_state={} entry_marker={} context=[{}] frames={}",
            args.caller_rva,
            preview_resource_callsite_label(args.caller_rva),
            args.trace
                .and_then(|trace| trace.selected_variant)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            args.trace
                .and_then(|trace| trace.selected_slot)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            args.trace
                .and_then(|trace| trace.selected_model_resource)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            args.trace
                .and_then(|trace| trace.selected_preview_mapped_resource)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            format_preview_resource_queue_first_index(args.after_queue_probe),
            args.after
                .and_then(|entry| entry.state)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            args.after
                .and_then(|entry| entry.marker)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            context,
            format_stack_frames(args.frames, 7)
        ));
    }
}

fn log_resource_911_cleanup(
    boundary: &str,
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
    context: &str,
    entries: &str,
) {
    if !preview_resource_911_hidden_reset(before, after) {
        return;
    }
    if RESOURCE_911_CLEANUP_LOGS.fetch_add(1, Ordering::Relaxed) >= RESOURCE_911_CLEANUP_LOG_CAP {
        return;
    }
    log::write_line(format!(
        "911-cleanup boundary={boundary} before=[{}] after=[{}] entries=[{}] context=[{}]",
        format_preview_resource_match_state(before),
        format_preview_resource_match_state(after),
        entries,
        context
    ));
}

fn preview_resource_911_watch_current() -> Option<PreviewResourceResolveMatch> {
    let table = LAST_PREVIEW_RESOURCE_TABLE.load(Ordering::Relaxed);
    preview_resource_resolve_match(table, LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
}

fn preview_resource_911_watch_previous() -> Option<PreviewResourceResolveMatch> {
    if !RESOURCE_911_WATCH_ARMED.load(Ordering::Relaxed) {
        return None;
    }
    let entry_ptr = RESOURCE_911_WATCH_ENTRY_PTR.load(Ordering::Relaxed);
    if entry_ptr == 0 {
        return None;
    }
    Some(PreviewResourceResolveMatch {
        table_base: RESOURCE_911_WATCH_TABLE_BASE.load(Ordering::Relaxed),
        entry_index: RESOURCE_911_WATCH_ENTRY_INDEX.load(Ordering::Relaxed),
        entry_ptr,
        state: preview_resource_watch_optional_u32(
            RESOURCE_911_WATCH_STATE.load(Ordering::Relaxed),
        ),
        flags: preview_resource_watch_optional_u32(
            RESOURCE_911_WATCH_FLAGS.load(Ordering::Relaxed),
        ),
        extra: preview_resource_watch_optional_u32(
            RESOURCE_911_WATCH_EXTRA.load(Ordering::Relaxed),
        ),
        marker: preview_resource_watch_optional_u32(
            RESOURCE_911_WATCH_MARKER.load(Ordering::Relaxed),
        ),
    })
}

fn preview_resource_watch_optional_u32(value: usize) -> Option<u32> {
    if value == u32::MAX as usize {
        None
    } else {
        Some(value as u32)
    }
}

fn store_preview_resource_911_watch_snapshot(entry: PreviewResourceResolveMatch) {
    RESOURCE_911_WATCH_TABLE_BASE.store(entry.table_base, Ordering::Relaxed);
    RESOURCE_911_WATCH_ENTRY_INDEX.store(entry.entry_index, Ordering::Relaxed);
    RESOURCE_911_WATCH_ENTRY_PTR.store(entry.entry_ptr, Ordering::Relaxed);
    RESOURCE_911_WATCH_STATE.store(entry.state.unwrap_or(u32::MAX) as usize, Ordering::Relaxed);
    RESOURCE_911_WATCH_FLAGS.store(entry.flags.unwrap_or(u32::MAX) as usize, Ordering::Relaxed);
    RESOURCE_911_WATCH_EXTRA.store(entry.extra.unwrap_or(u32::MAX) as usize, Ordering::Relaxed);
    RESOURCE_911_WATCH_MARKER.store(entry.marker.unwrap_or(u32::MAX) as usize, Ordering::Relaxed);
}

fn resource_911_watch_arm_candidate(entry: PreviewResourceResolveMatch) -> bool {
    entry.state == Some(1) && entry.marker == Some(0)
}

fn install_resource_911_page_guard_watch() -> bool {
    if !RESOURCE_911_PAGE_GUARD_WATCH_ENABLED {
        log::write_line("resource911 page guard watch disabled");
        return false;
    }
    if RESOURCE_911_PAGE_GUARD_HANDLER_INSTALLED.swap(true, Ordering::AcqRel) {
        return false;
    }
    let Some(handle) =
        (unsafe { win::add_vectored_exception_handler(true, resource_911_page_guard_handler) })
    else {
        RESOURCE_911_PAGE_GUARD_HANDLER_INSTALLED.store(false, Ordering::Release);
        log::write_line("resource911 page guard watch failed error=veh_install_failed");
        return false;
    };
    log::write_line(format!(
        "resource911 page guard watch installed veh=0x{handle:x}"
    ));
    true
}

unsafe extern "system" fn resource_911_page_guard_handler(
    exception_info: *mut win::ExceptionPointers,
) -> std::ffi::c_long {
    if exception_info.is_null() {
        return win::EXCEPTION_CONTINUE_SEARCH;
    }
    let record = (*exception_info).exception_record;
    if record.is_null() {
        return win::EXCEPTION_CONTINUE_SEARCH;
    }
    if (*record).exception_code == win::EXCEPTION_SINGLE_STEP
        && RESOURCE_911_PAGE_GUARD_SINGLE_STEP_PENDING.swap(false, Ordering::AcqRel)
    {
        if RESOURCE_911_PAGE_GUARD_SINGLE_STEP_EVENTS.load(Ordering::Acquire)
            < RESOURCE_911_PAGE_GUARD_SINGLE_STEP_CAP
        {
            let page_base = RESOURCE_911_PAGE_GUARD_PAGE_BASE.load(Ordering::Relaxed);
            let page_size = RESOURCE_911_PAGE_GUARD_PAGE_SIZE.load(Ordering::Relaxed);
            if page_base != 0 && page_size != 0 {
                if let Some(region) = win::memory_region(page_base) {
                    let new_protect = region.protect | win::PAGE_GUARD;
                    if unsafe { win::protect_memory(page_base, page_size, new_protect) }.is_some() {
                        RESOURCE_911_PAGE_GUARD_ACTIVE.store(true, Ordering::Release);
                    }
                }
            }
        }
        return win::EXCEPTION_CONTINUE_EXECUTION;
    }
    if (*record).exception_code != win::EXCEPTION_GUARD_PAGE
        || !RESOURCE_911_PAGE_GUARD_ACTIVE.load(Ordering::Acquire)
    {
        return win::EXCEPTION_CONTINUE_SEARCH;
    }

    let page_base = RESOURCE_911_PAGE_GUARD_PAGE_BASE.load(Ordering::Relaxed);
    let page_size = RESOURCE_911_PAGE_GUARD_PAGE_SIZE.load(Ordering::Relaxed);
    let fault_addr = if (*record).number_parameters > 1 {
        (*record).exception_information[1]
    } else {
        0
    };
    let access_type = if (*record).number_parameters > 0 {
        (*record).exception_information[0]
    } else {
        usize::MAX
    };
    if page_base == 0
        || page_size == 0
        || fault_addr < page_base
        || fault_addr >= page_base.saturating_add(page_size)
    {
        return win::EXCEPTION_CONTINUE_SEARCH;
    }

    let events = RESOURCE_911_PAGE_GUARD_EVENTS.fetch_add(1, Ordering::AcqRel);
    if events >= RESOURCE_911_PAGE_GUARD_EVENT_CAP {
        RESOURCE_911_PAGE_GUARD_ACTIVE.store(false, Ordering::Release);
        return win::EXCEPTION_CONTINUE_EXECUTION;
    }

    let entry_ptr = RESOURCE_911_WATCH_ENTRY_PTR.load(Ordering::Relaxed);
    let relevant = fault_addr >= entry_ptr
        && fault_addr < entry_ptr.saturating_add(PREVIEW_RESOURCE_SLOT_STRIDE);
    let write_offset = if relevant {
        fault_addr.saturating_sub(entry_ptr)
    } else {
        usize::MAX
    };

    let rip = (*record).exception_address as usize;
    let hit_seq = RESOURCE_911_PAGE_GUARD_HIT_SEQ.fetch_add(1, Ordering::AcqRel) + 1;
    let hit_state = RESOURCE_911_WATCH_STATE.load(Ordering::Relaxed);
    let hit_marker = RESOURCE_911_WATCH_MARKER.load(Ordering::Relaxed);
    match access_type {
        0 => {
            RESOURCE_911_PAGE_GUARD_READ_HITS.fetch_add(1, Ordering::AcqRel);
        }
        1 => {
            RESOURCE_911_PAGE_GUARD_WRITE_HITS.fetch_add(1, Ordering::AcqRel);
        }
        _ => {}
    }
    let registers = unsafe { win::read_x64_register_snapshot((*exception_info).context_record) };
    let hit_id = read_u32_field(entry_ptr, PREVIEW_RESOURCE_SLOT_ID_OFFSET)
        .map(|value| value as usize)
        .unwrap_or(u32::MAX as usize);
    let hit_q40 = read_usize_field(entry_ptr, 0x40).unwrap_or(0);

    RESOURCE_911_PAGE_GUARD_HIT_RIP.store(rip, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HIT_FAULT.store(fault_addr, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HIT_OFFSET.store(write_offset, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HIT_ACCESS.store(access_type, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HIT_TABLE_BASE.store(
        RESOURCE_911_WATCH_TABLE_BASE.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
    RESOURCE_911_PAGE_GUARD_HIT_ENTRY_INDEX.store(
        RESOURCE_911_WATCH_ENTRY_INDEX.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
    RESOURCE_911_PAGE_GUARD_HIT_ENTRY_PTR.store(entry_ptr, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HIT_STATE.store(hit_state, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HIT_FLAGS.store(
        RESOURCE_911_WATCH_FLAGS.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
    RESOURCE_911_PAGE_GUARD_HIT_EXTRA.store(
        RESOURCE_911_WATCH_EXTRA.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
    RESOURCE_911_PAGE_GUARD_HIT_MARKER.store(hit_marker, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_RELEVANT_HIT.store(relevant, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_PENDING.store(true, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_ACTIVE.store(false, Ordering::Release);
    let rearmed = RESOURCE_911_PAGE_GUARD_SINGLE_STEP_ENABLED
        && RESOURCE_911_PAGE_GUARD_SINGLE_STEP_EVENTS.fetch_add(1, Ordering::AcqRel)
            < RESOURCE_911_PAGE_GUARD_SINGLE_STEP_CAP
        && unsafe { win::set_x64_context_trap_flag((*exception_info).context_record) };
    RESOURCE_911_PAGE_GUARD_HIT_REARMED.store(rearmed, Ordering::Relaxed);
    store_resource_911_page_guard_hit_history(
        hit_seq,
        rip,
        fault_addr,
        write_offset,
        access_type,
        rearmed,
        hit_state,
        hit_marker,
        hit_id,
        hit_q40,
        registers,
    );
    if rearmed {
        RESOURCE_911_PAGE_GUARD_SINGLE_STEP_PENDING.store(true, Ordering::Release);
    }

    win::EXCEPTION_CONTINUE_EXECUTION
}

fn store_resource_911_page_guard_hit_history(
    seq: usize,
    rip: usize,
    fault_addr: usize,
    offset: usize,
    access_type: usize,
    rearmed: bool,
    state: usize,
    marker: usize,
    id: usize,
    q40: usize,
    registers: Option<win::X64RegisterSnapshot>,
) {
    let slot = seq % RESOURCE_911_PAGE_GUARD_HISTORY_SEQ.len();
    RESOURCE_911_PAGE_GUARD_HISTORY_RIP[slot].store(rip, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_FAULT[slot].store(fault_addr, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_OFFSET[slot].store(offset, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_ACCESS[slot].store(access_type, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_REARMED[slot].store(rearmed, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_STATE[slot].store(state, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_MARKER[slot].store(marker, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_ID[slot].store(id, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_Q40[slot].store(q40, Ordering::Relaxed);
    let registers = registers.unwrap_or(win::X64RegisterSnapshot {
        rax: 0,
        rbx: 0,
        rcx: 0,
        rdx: 0,
        rsi: 0,
        rdi: 0,
        r8: 0,
        r9: 0,
        r10: 0,
        r11: 0,
    });
    RESOURCE_911_PAGE_GUARD_HISTORY_RAX[slot].store(registers.rax, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_RBX[slot].store(registers.rbx, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_RCX[slot].store(registers.rcx, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_RDX[slot].store(registers.rdx, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_RSI[slot].store(registers.rsi, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_RDI[slot].store(registers.rdi, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_R8[slot].store(registers.r8, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_R9[slot].store(registers.r9, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_R10[slot].store(registers.r10, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_R11[slot].store(registers.r11, Ordering::Relaxed);
    RESOURCE_911_PAGE_GUARD_HISTORY_SEQ[slot].store(seq, Ordering::Release);
}

fn clear_resource_911_page_guard_hit_history() {
    RESOURCE_911_PAGE_GUARD_HIT_SEQ.store(0, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_HIT_REARMED.store(false, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_READ_HITS.store(0, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_WRITE_HITS.store(0, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_READ_LOGS.store(0, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_WRITE_LOGS.store(0, Ordering::Release);
    for index in 0..RESOURCE_911_PAGE_GUARD_HISTORY_SEQ.len() {
        RESOURCE_911_PAGE_GUARD_HISTORY_SEQ[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_RIP[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_FAULT[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_OFFSET[index].store(usize::MAX, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_ACCESS[index].store(usize::MAX, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_REARMED[index].store(false, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_STATE[index].store(u32::MAX as usize, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_MARKER[index].store(u32::MAX as usize, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_ID[index].store(u32::MAX as usize, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_Q40[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_RAX[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_RBX[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_RCX[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_RDX[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_RSI[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_RDI[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_R8[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_R9[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_R10[index].store(0, Ordering::Release);
        RESOURCE_911_PAGE_GUARD_HISTORY_R11[index].store(0, Ordering::Release);
    }
}

fn preview_resource_911_page_guard_hit_snapshot() -> Option<PreviewResourceResolveMatch> {
    let entry_ptr = RESOURCE_911_PAGE_GUARD_HIT_ENTRY_PTR.load(Ordering::Relaxed);
    if entry_ptr == 0 {
        return None;
    }
    Some(PreviewResourceResolveMatch {
        table_base: RESOURCE_911_PAGE_GUARD_HIT_TABLE_BASE.load(Ordering::Relaxed),
        entry_index: RESOURCE_911_PAGE_GUARD_HIT_ENTRY_INDEX.load(Ordering::Relaxed),
        entry_ptr,
        state: preview_resource_watch_optional_u32(
            RESOURCE_911_PAGE_GUARD_HIT_STATE.load(Ordering::Relaxed),
        ),
        flags: preview_resource_watch_optional_u32(
            RESOURCE_911_PAGE_GUARD_HIT_FLAGS.load(Ordering::Relaxed),
        ),
        extra: preview_resource_watch_optional_u32(
            RESOURCE_911_PAGE_GUARD_HIT_EXTRA.load(Ordering::Relaxed),
        ),
        marker: preview_resource_watch_optional_u32(
            RESOURCE_911_PAGE_GUARD_HIT_MARKER.load(Ordering::Relaxed),
        ),
    })
}

fn format_resource_911_page_guard_offset(offset: usize) -> String {
    if offset == usize::MAX {
        "outside".to_string()
    } else {
        format!("0x{offset:x}")
    }
}

fn format_resource_911_page_guard_delta(fault_addr: usize, entry_ptr: usize) -> String {
    if fault_addr >= entry_ptr {
        format!("0x{:x}", fault_addr - entry_ptr)
    } else {
        format!("-0x{:x}", entry_ptr - fault_addr)
    }
}

fn format_resource_911_page_guard_access(access_type: usize) -> &'static str {
    match access_type {
        0 => "read",
        1 => "write",
        8 => "execute",
        usize::MAX => "unknown",
        _ => "other",
    }
}

fn format_resource_911_page_guard_hit_chain(entry_ptr: usize) -> String {
    let mut hits = Vec::new();
    for index in 0..RESOURCE_911_PAGE_GUARD_HISTORY_SEQ.len() {
        let seq = RESOURCE_911_PAGE_GUARD_HISTORY_SEQ[index].load(Ordering::Acquire);
        if seq == 0 {
            continue;
        }
        hits.push((
            seq,
            RESOURCE_911_PAGE_GUARD_HISTORY_RIP[index].load(Ordering::Relaxed),
            RESOURCE_911_PAGE_GUARD_HISTORY_FAULT[index].load(Ordering::Relaxed),
            RESOURCE_911_PAGE_GUARD_HISTORY_OFFSET[index].load(Ordering::Relaxed),
            RESOURCE_911_PAGE_GUARD_HISTORY_ACCESS[index].load(Ordering::Relaxed),
            RESOURCE_911_PAGE_GUARD_HISTORY_REARMED[index].load(Ordering::Relaxed),
            RESOURCE_911_PAGE_GUARD_HISTORY_STATE[index].load(Ordering::Relaxed),
            RESOURCE_911_PAGE_GUARD_HISTORY_MARKER[index].load(Ordering::Relaxed),
        ));
    }
    hits.sort_by_key(|hit| hit.0);
    if hits.is_empty() {
        return "none".to_string();
    }
    hits.into_iter()
        .map(
            |(seq, rip, fault_addr, offset, access_type, rearmed, state, marker)| {
                format!(
                    "#{} rip={} access_type={} fault_addr=0x{:x} entry_delta={} write_offset={} rearm={} state={} marker={}",
                    seq,
                    format_stack_frame(rip),
                    format_resource_911_page_guard_access(access_type),
                    fault_addr,
                    format_resource_911_page_guard_delta(fault_addr, entry_ptr),
                    format_resource_911_page_guard_offset(offset),
                    rearmed,
                    format_optional_u32(preview_resource_watch_optional_u32(state)),
                    format_optional_u32(preview_resource_watch_optional_u32(marker))
                )
            },
        )
        .collect::<Vec<_>>()
        .join(" | ")
}

fn preview_resource_entry_snapshot_at(
    table_base: usize,
    entry_index: usize,
    entry_ptr: usize,
) -> Option<PreviewResourceResolveMatch> {
    if entry_ptr == 0 {
        return None;
    }
    Some(PreviewResourceResolveMatch {
        table_base,
        entry_index,
        entry_ptr,
        state: read_u32_field(entry_ptr, PREVIEW_RESOURCE_SLOT_STATE_OFFSET),
        flags: read_u32_field(entry_ptr, PREVIEW_RESOURCE_SLOT_FLAGS_OFFSET),
        extra: read_u32_field(entry_ptr, PREVIEW_RESOURCE_SLOT_EXTRA_OFFSET),
        marker: read_u32_field(entry_ptr, PREVIEW_RESOURCE_SLOT_MARKER_OFFSET),
    })
}

fn resource_911_page_guard_history_rva(rip: usize) -> Option<usize> {
    let game_base = win::main_module() as usize;
    (game_base != 0 && rip >= game_base).then_some(rip - game_base)
}

fn resource_911_direct_reset_write_kind(write_offset: usize) -> Option<&'static str> {
    match write_offset {
        PREVIEW_RESOURCE_SLOT_STATE_OFFSET => Some("state"),
        PREVIEW_RESOURCE_SLOT_MARKER_OFFSET => Some("marker"),
        _ => None,
    }
}

fn resource_911_direct_reset_expected_rip(kind: &str) -> usize {
    match kind {
        "state" => COSTUME_PREVIEW_RESOURCE_RESET_STATE_WRITE_RVA,
        "marker" => COSTUME_PREVIEW_RESOURCE_RESET_MARKER_WRITE_RVA,
        _ => 0,
    }
}

fn format_resource_911_page_guard_history_registers(index: usize) -> String {
    format!(
        "rax=0x{:x} rbx=0x{:x} rcx=0x{:x} rdx=0x{:x} rsi=0x{:x} rdi=0x{:x} r8=0x{:x} r9=0x{:x} r10=0x{:x} r11=0x{:x}",
        RESOURCE_911_PAGE_GUARD_HISTORY_RAX[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_RBX[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_RCX[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_RDX[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_RSI[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_RDI[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_R8[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_R9[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_R10[index].load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_HISTORY_R11[index].load(Ordering::Relaxed)
    )
}

fn format_resource_911_register_delta(value: usize, entry_ptr: usize) -> String {
    if value == 0 || entry_ptr == 0 {
        "none".to_string()
    } else if value >= entry_ptr {
        format!("0x{:x}", value - entry_ptr)
    } else {
        format!("-0x{:x}", entry_ptr - value)
    }
}

fn format_resource_911_page_guard_history_register_deltas(
    index: usize,
    entry_ptr: usize,
) -> String {
    let registers = [
        (
            "rax",
            RESOURCE_911_PAGE_GUARD_HISTORY_RAX[index].load(Ordering::Relaxed),
        ),
        (
            "rbx",
            RESOURCE_911_PAGE_GUARD_HISTORY_RBX[index].load(Ordering::Relaxed),
        ),
        (
            "rcx",
            RESOURCE_911_PAGE_GUARD_HISTORY_RCX[index].load(Ordering::Relaxed),
        ),
        (
            "rdx",
            RESOURCE_911_PAGE_GUARD_HISTORY_RDX[index].load(Ordering::Relaxed),
        ),
        (
            "rsi",
            RESOURCE_911_PAGE_GUARD_HISTORY_RSI[index].load(Ordering::Relaxed),
        ),
        (
            "rdi",
            RESOURCE_911_PAGE_GUARD_HISTORY_RDI[index].load(Ordering::Relaxed),
        ),
        (
            "r8",
            RESOURCE_911_PAGE_GUARD_HISTORY_R8[index].load(Ordering::Relaxed),
        ),
        (
            "r9",
            RESOURCE_911_PAGE_GUARD_HISTORY_R9[index].load(Ordering::Relaxed),
        ),
        (
            "r10",
            RESOURCE_911_PAGE_GUARD_HISTORY_R10[index].load(Ordering::Relaxed),
        ),
        (
            "r11",
            RESOURCE_911_PAGE_GUARD_HISTORY_R11[index].load(Ordering::Relaxed),
        ),
    ];
    registers
        .into_iter()
        .map(|(name, value)| {
            format!(
                "{name}={}",
                format_resource_911_register_delta(value, entry_ptr)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_resource_911_direct_reset_cursor(
    index: usize,
    entry_ptr: usize,
    fault_addr: usize,
) -> String {
    let rdi = RESOURCE_911_PAGE_GUARD_HISTORY_RDI[index].load(Ordering::Relaxed);
    let rbx = RESOURCE_911_PAGE_GUARD_HISTORY_RBX[index].load(Ordering::Relaxed);
    let q40 = RESOURCE_911_PAGE_GUARD_HISTORY_Q40[index].load(Ordering::Relaxed);
    let cursor_entry_ptr = rdi.checked_sub(0x50);
    let state_expr = rdi.checked_add(0x170) == Some(fault_addr);
    let marker_expr = rdi.checked_add(0x17c) == Some(fault_addr);
    let cursor_reg = if state_expr || marker_expr {
        "rdi"
    } else {
        "none"
    };
    let cursor_index_guess = cursor_entry_ptr
        .and_then(|cursor| {
            cursor.checked_sub(RESOURCE_911_WATCH_TABLE_BASE.load(Ordering::Relaxed))
        })
        .and_then(|delta| delta.checked_sub(PREVIEW_RESOURCE_SLOT_BASE_OFFSET))
        .map(|delta| delta / PREVIEW_RESOURCE_SLOT_STRIDE);
    let cursor_entry_id =
        cursor_entry_ptr.and_then(|cursor| read_u32_field(cursor, PREVIEW_RESOURCE_SLOT_ID_OFFSET));
    format!(
        "cursor_reg={cursor_reg} cursor_delta={} state_expr={} marker_expr={} q40_match={} cursor_entry_ptr={} cursor_index_guess={} cursor_entry_id={}",
        format_resource_911_register_delta(rdi, entry_ptr),
        state_expr,
        marker_expr,
        rbx != 0 && rbx == q40,
        format_optional_address(cursor_entry_ptr),
        cursor_index_guess
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        format_optional_u32(cursor_entry_id)
    )
}

fn format_resource_911_direct_reset_write_before(index: usize) -> String {
    let id = RESOURCE_911_PAGE_GUARD_HISTORY_ID[index].load(Ordering::Relaxed);
    let q40 = RESOURCE_911_PAGE_GUARD_HISTORY_Q40[index].load(Ordering::Relaxed);
    let state = RESOURCE_911_PAGE_GUARD_HISTORY_STATE[index].load(Ordering::Relaxed);
    let marker = RESOURCE_911_PAGE_GUARD_HISTORY_MARKER[index].load(Ordering::Relaxed);
    format!(
        "id={} q40={} state={} marker={}",
        format_optional_u32(preview_resource_watch_optional_u32(id)),
        format_optional_address((q40 != 0).then_some(q40)),
        format_optional_u32(preview_resource_watch_optional_u32(state)),
        format_optional_u32(preview_resource_watch_optional_u32(marker))
    )
}

fn resource_911_writer_block_cursor_index(
    cursor_entry_ptr: usize,
    table_base: usize,
) -> Option<usize> {
    cursor_entry_ptr
        .checked_sub(table_base)?
        .checked_sub(PREVIEW_RESOURCE_SLOT_BASE_OFFSET)
        .filter(|delta| delta % PREVIEW_RESOURCE_SLOT_STRIDE == 0)
        .map(|delta| delta / PREVIEW_RESOURCE_SLOT_STRIDE)
}

fn resource_911_writer_block_interesting(id: Option<u32>, index: Option<usize>) -> bool {
    matches!(
        id,
        Some(
            LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID
                | LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
                | LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID
                | LAW_EXTRA_SLOT_PREVIEW_TABLE_ONI_FALLBACK_SOURCE_ID
        )
    ) || index.is_some_and(|value| (642..=648).contains(&value))
}

fn log_resource_911_writer_block(rdi: usize, rbx: usize, r12: usize) {
    let Some(cursor_entry_ptr) = rdi.checked_sub(0x50) else {
        return;
    };
    let table_base = RESOURCE_911_WATCH_TABLE_BASE.load(Ordering::Relaxed);
    let watch_index = RESOURCE_911_WATCH_ENTRY_INDEX.load(Ordering::Relaxed);
    let watch_entry_ptr = RESOURCE_911_WATCH_ENTRY_PTR.load(Ordering::Relaxed);
    let cursor_index = resource_911_writer_block_cursor_index(cursor_entry_ptr, table_base);
    let cursor_entry_id = read_u32_field(cursor_entry_ptr, PREVIEW_RESOURCE_SLOT_ID_OFFSET);
    if !resource_911_writer_block_interesting(cursor_entry_id, cursor_index) {
        return;
    }
    if RESOURCE_911_WRITER_BLOCK_LOGS.fetch_add(1, Ordering::Relaxed)
        >= RESOURCE_911_WRITER_BLOCK_LOG_CAP
    {
        return;
    }

    let q40 = read_usize_field(cursor_entry_ptr, 0x40).unwrap_or(0);
    let before_entry = preview_resource_entry_snapshot_at(
        table_base,
        cursor_index.unwrap_or(usize::MAX),
        cursor_entry_ptr,
    );
    let context = preview_resource_trace_context(
        last_law_ready_timeline_trace_label().map(|(_, trace)| trace),
    );
    log::write_line(format!(
        "resource911-writer-block rdi=0x{rdi:x} rbx=0x{rbx:x} r12=0x{r12:x} cursor_entry_ptr=0x{cursor_entry_ptr:x} cursor_entry_id={} cursor_index_guess={} q40={} q40_match={} would_write_state={} would_write_marker=1 watch_table_base=0x{table_base:x} watch_entry_index={} watch_entry_ptr={} before_entry=[{}] context=[{}]",
        format_optional_u32(cursor_entry_id),
        cursor_index
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        format_optional_address((q40 != 0).then_some(q40)),
        q40 != 0 && q40 == rbx,
        (r12 as u32),
        if watch_index == usize::MAX {
            "none".to_string()
        } else {
            watch_index.to_string()
        },
        format_optional_address((watch_entry_ptr != 0).then_some(watch_entry_ptr)),
        format_preview_resource_entry_payload(before_entry),
        context
    ));
}

#[derive(Debug, Clone)]
struct WriterBlockSnapshotTrace {
    seq: usize,
    rdi: usize,
    rbx: usize,
    r12: usize,
    cursor_entry_ptr: Option<usize>,
    cursor_index: Option<usize>,
    cursor_entry_id: Option<u32>,
    q40: usize,
    before_entry: Option<PreviewResourceResolveMatch>,
}

fn log_resource_911_writer_block_sequence(
    boundary: &str,
    before_states: TrackedPreviewResourceStates,
    after_states: TrackedPreviewResourceStates,
    context: &str,
) {
    let start_seq = RESOURCE_911_WRITER_BLOCK_SEQUENCE_START.load(Ordering::Acquire);
    let end_seq = RESOURCE_911_WRITER_BLOCK_SNAPSHOT_SEQ.load(Ordering::Acquire);
    if end_seq <= start_seq {
        return;
    }

    let hidden_reset =
        preview_resource_911_hidden_reset(before_states.resource_911, after_states.resource_911);
    let table_base = RESOURCE_911_WATCH_TABLE_BASE.load(Ordering::Relaxed);
    let watch_index = RESOURCE_911_WATCH_ENTRY_INDEX.load(Ordering::Relaxed);
    let watch_entry_ptr = RESOURCE_911_WATCH_ENTRY_PTR.load(Ordering::Relaxed);
    let truncated = end_seq.saturating_sub(start_seq) > RESOURCE_911_WRITER_BLOCK_RING_SIZE;
    let first_seq = if truncated {
        end_seq.saturating_sub(RESOURCE_911_WRITER_BLOCK_RING_SIZE - 1)
    } else {
        start_seq + 1
    };
    let snapshots = (first_seq..=end_seq)
        .filter_map(read_resource_911_writer_block_snapshot_trace)
        .collect::<Vec<_>>();
    if snapshots.is_empty() {
        return;
    }

    let watch_seen = snapshots
        .iter()
        .any(|snapshot| snapshot.cursor_entry_ptr == Some(watch_entry_ptr));
    let interesting = hidden_reset
        || watch_seen
        || snapshots.iter().any(|snapshot| {
            resource_911_writer_block_interesting(snapshot.cursor_entry_id, snapshot.cursor_index)
                || writer_block_snapshot_near_watch(snapshot.cursor_index, watch_index)
        });
    if !interesting {
        return;
    }
    if watch_seen
        && RESOURCE_911_WRITER_BLOCK_FIRST_WATCH_SEQUENCE_LOGGED.swap(true, Ordering::Relaxed)
    {
        return;
    }
    if !hidden_reset && !watch_seen {
        return;
    }
    if RESOURCE_911_WRITER_BLOCK_LOGS.fetch_add(1, Ordering::Relaxed)
        >= RESOURCE_911_WRITER_BLOCK_LOG_CAP
    {
        return;
    }

    let entries = snapshots
        .iter()
        .filter(|snapshot| {
            hidden_reset
                || snapshot.cursor_entry_ptr == Some(watch_entry_ptr)
                || writer_block_snapshot_near_watch(snapshot.cursor_index, watch_index)
                || resource_911_writer_block_interesting(
                    snapshot.cursor_entry_id,
                    snapshot.cursor_index,
                )
        })
        .map(format_resource_911_writer_block_snapshot_trace)
        .collect::<Vec<_>>()
        .join(" | ");

    log_resource_911_cleanup(
        boundary,
        before_states.resource_911,
        after_states.resource_911,
        context,
        &entries,
    );

    log::write_line(format!(
        "resource911-writer-block-sequence boundary={boundary} seq_range={}..{} captured={} truncated={} hidden_reset={} watch_seen={} table_base=0x{table_base:x} watch_entry_index={} watch_entry_ptr={} before911=[{}] after911=[{}] entries=[{}] context=[{}]",
        start_seq + 1,
        end_seq,
        snapshots.len(),
        truncated,
        hidden_reset,
        watch_seen,
        if watch_index == usize::MAX {
            "none".to_string()
        } else {
            watch_index.to_string()
        },
        format_optional_address((watch_entry_ptr != 0).then_some(watch_entry_ptr)),
        format_preview_resource_entry_payload(before_states.resource_911),
        format_preview_resource_entry_payload(after_states.resource_911),
        entries,
        context
    ));
}

fn read_resource_911_writer_block_snapshot_trace(seq: usize) -> Option<WriterBlockSnapshotTrace> {
    let slot = seq & RESOURCE_911_WRITER_BLOCK_RING_MASK;
    if RESOURCE_911_WRITER_BLOCK_SNAPSHOT_SEQ_RING[slot].load(Ordering::Acquire) != seq {
        return None;
    }

    let rdi = RESOURCE_911_WRITER_BLOCK_SNAPSHOT_RDI_RING[slot].load(Ordering::Relaxed);
    let rbx = RESOURCE_911_WRITER_BLOCK_SNAPSHOT_RBX_RING[slot].load(Ordering::Relaxed);
    let r12 = RESOURCE_911_WRITER_BLOCK_SNAPSHOT_R12_RING[slot].load(Ordering::Relaxed);
    let cursor_entry_ptr = rdi.checked_sub(0x50);
    let table_base = RESOURCE_911_WATCH_TABLE_BASE.load(Ordering::Relaxed);
    let cursor_index = cursor_entry_ptr
        .and_then(|cursor| resource_911_writer_block_cursor_index(cursor, table_base));
    let cursor_entry_id = cursor_index
        .and(cursor_entry_ptr)
        .and_then(|cursor| read_u32_field(cursor, PREVIEW_RESOURCE_SLOT_ID_OFFSET));
    let q40 = cursor_index
        .and(cursor_entry_ptr)
        .and_then(|cursor| read_usize_field(cursor, 0x40))
        .unwrap_or(0);
    let before_entry = cursor_index.and_then(|index| {
        cursor_entry_ptr
            .and_then(|cursor| preview_resource_entry_snapshot_at(table_base, index, cursor))
    });

    Some(WriterBlockSnapshotTrace {
        seq,
        rdi,
        rbx,
        r12,
        cursor_entry_ptr,
        cursor_index,
        cursor_entry_id,
        q40,
        before_entry,
    })
}

fn writer_block_snapshot_near_watch(cursor_index: Option<usize>, watch_index: usize) -> bool {
    if watch_index == usize::MAX {
        return false;
    }
    cursor_index.is_some_and(|index| index.abs_diff(watch_index) <= 8)
}

fn format_resource_911_writer_block_snapshot_trace(snapshot: &WriterBlockSnapshotTrace) -> String {
    format!(
        "seq={} index={} id={} rdi=0x{:x} rbx=0x{:x} r12=0x{:x} cursor_entry_ptr={} q40={} q40_match={} would_write_state={} would_write_marker=1 before=[{}]",
        snapshot.seq,
        snapshot
            .cursor_index
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        format_optional_u32(snapshot.cursor_entry_id),
        snapshot.rdi,
        snapshot.rbx,
        snapshot.r12,
        format_optional_address(snapshot.cursor_entry_ptr),
        format_optional_address((snapshot.q40 != 0).then_some(snapshot.q40)),
        snapshot.q40 != 0 && snapshot.q40 == snapshot.rbx,
        snapshot.r12 as u32,
        format_preview_resource_entry_payload(snapshot.before_entry)
    )
}

fn log_resource_911_writer_code_bytes_once() {
    if RESOURCE_911_WRITER_CODE_DUMP_LOGGED.swap(true, Ordering::AcqRel) {
        return;
    }
    let game_base = win::main_module() as usize;
    if game_base == 0 {
        log_resource_911_page_guard_line(
            "resource911-writer-code-bytes skipped reason=no_game_base".to_string(),
        );
        return;
    }
    let start = game_base.saturating_add(RESOURCE_911_WRITER_CODE_DUMP_START_RVA);
    let mut bytes = [0u8; RESOURCE_911_WRITER_CODE_DUMP_LEN];
    if !read_exact_process_memory(start, &mut bytes) {
        log_resource_911_page_guard_line(format!(
            "resource911-writer-code-bytes skipped reason=read_failed start=game+0x{:x} len=0x{:x}",
            RESOURCE_911_WRITER_CODE_DUMP_START_RVA, RESOURCE_911_WRITER_CODE_DUMP_LEN
        ));
        return;
    }
    log_resource_911_page_guard_line(format!(
        "resource911-writer-code-bytes start=game+0x{:x} len=0x{:x} table_base=0x{:x} entry_index={} entry_ptr=0x{:x} bytes={}",
        RESOURCE_911_WRITER_CODE_DUMP_START_RVA,
        RESOURCE_911_WRITER_CODE_DUMP_LEN,
        RESOURCE_911_WATCH_TABLE_BASE.load(Ordering::Relaxed),
        RESOURCE_911_WATCH_ENTRY_INDEX.load(Ordering::Relaxed),
        RESOURCE_911_WATCH_ENTRY_PTR.load(Ordering::Relaxed),
        format_bytes(&bytes)
    ));
}

fn log_resource_911_direct_reset_writes(boundary: &str, phase: &str, context: &str) -> bool {
    let mut logged_any = false;
    for index in 0..RESOURCE_911_PAGE_GUARD_HISTORY_SEQ.len() {
        let seq = RESOURCE_911_PAGE_GUARD_HISTORY_SEQ[index].load(Ordering::Acquire);
        if seq == 0 {
            continue;
        }
        let rip = RESOURCE_911_PAGE_GUARD_HISTORY_RIP[index].load(Ordering::Relaxed);
        let write_offset = RESOURCE_911_PAGE_GUARD_HISTORY_OFFSET[index].load(Ordering::Relaxed);
        let access_type = RESOURCE_911_PAGE_GUARD_HISTORY_ACCESS[index].load(Ordering::Relaxed);
        let Some(kind) = resource_911_direct_reset_write_kind(write_offset) else {
            continue;
        };
        if access_type != 1 {
            continue;
        }
        if RESOURCE_911_DIRECT_RESET_WRITE_LOGS.fetch_add(1, Ordering::Relaxed)
            >= RESOURCE_911_DIRECT_RESET_WRITE_LOG_CAP
        {
            return logged_any;
        }
        let table_base = RESOURCE_911_WATCH_TABLE_BASE.load(Ordering::Relaxed);
        let entry_index = RESOURCE_911_WATCH_ENTRY_INDEX.load(Ordering::Relaxed);
        let entry_ptr = RESOURCE_911_WATCH_ENTRY_PTR.load(Ordering::Relaxed);
        let fault_addr = RESOURCE_911_PAGE_GUARD_HISTORY_FAULT[index].load(Ordering::Relaxed);
        let after_entry = preview_resource_entry_snapshot_at(table_base, entry_index, entry_ptr);
        log_resource_911_writer_code_bytes_once();
        let rip_rva = resource_911_page_guard_history_rva(rip);
        let expected_rva = resource_911_direct_reset_expected_rip(kind);
        log_resource_911_page_guard_line(format!(
            "resource911-direct-reset-write kind={kind} boundary={boundary} phase={phase} seq={seq} rip={} rip_rva={} expected_rva=game+0x{expected_rva:x} expected_match={} fault_addr=0x{fault_addr:x} write_offset={} table_base=0x{table_base:x} entry_index={entry_index} entry_ptr=0x{entry_ptr:x} regs=[{}] deltas=[{}] cursor=[{}] before_entry=[{}] after_entry=[{}] context=[{}]",
            format_stack_frame(rip),
            rip_rva
                .map(|value| format!("game+0x{value:x}"))
                .unwrap_or_else(|| "none".to_string()),
            rip_rva == Some(expected_rva),
            format_resource_911_page_guard_offset(write_offset),
            format_resource_911_page_guard_history_registers(index),
            format_resource_911_page_guard_history_register_deltas(index, entry_ptr),
            format_resource_911_direct_reset_cursor(index, entry_ptr, fault_addr),
            format_resource_911_direct_reset_write_before(index),
            format_preview_resource_entry_payload(after_entry),
            context
        ));
        logged_any = true;
    }
    logged_any
}

fn resource_911_guard_missed_write_reason() -> &'static str {
    let write_hits = RESOURCE_911_PAGE_GUARD_WRITE_HITS.load(Ordering::Relaxed);
    let single_step_events = RESOURCE_911_PAGE_GUARD_SINGLE_STEP_EVENTS.load(Ordering::Relaxed);
    let events = RESOURCE_911_PAGE_GUARD_EVENTS.load(Ordering::Relaxed);
    let read_hits = RESOURCE_911_PAGE_GUARD_READ_HITS.load(Ordering::Relaxed);
    let active = RESOURCE_911_PAGE_GUARD_ACTIVE.load(Ordering::Acquire);
    let pending = RESOURCE_911_PAGE_GUARD_PENDING.load(Ordering::Acquire);
    if write_hits != 0 {
        "write_not_target"
    } else if single_step_events >= RESOURCE_911_PAGE_GUARD_SINGLE_STEP_CAP {
        "rearm_cap"
    } else if events >= RESOURCE_911_PAGE_GUARD_EVENT_CAP {
        "event_cap"
    } else if read_hits >= RESOURCE_911_PAGE_GUARD_READ_LOG_CAP {
        "read_saturated"
    } else if !active && !pending {
        "guard_disarmed"
    } else {
        "unknown"
    }
}

fn log_resource_911_guard_missed_write(
    boundary: &str,
    phase: &str,
    context: &str,
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
) {
    if RESOURCE_911_GUARD_MISSED_WRITE_LOGS.fetch_add(1, Ordering::Relaxed) != 0 {
        return;
    }
    let entry_ptr = RESOURCE_911_WATCH_ENTRY_PTR.load(Ordering::Relaxed);
    log_resource_911_page_guard_line(format!(
        "resource911-guard-missed-write boundary={boundary} phase={phase} reason={} events={} read_hits={} write_hits={} single_step_events={} active={} pending={} before=[{}] after=[{}] chain=[{}] context=[{}]",
        resource_911_guard_missed_write_reason(),
        RESOURCE_911_PAGE_GUARD_EVENTS.load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_READ_HITS.load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_WRITE_HITS.load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_SINGLE_STEP_EVENTS.load(Ordering::Relaxed),
        RESOURCE_911_PAGE_GUARD_ACTIVE.load(Ordering::Acquire),
        RESOURCE_911_PAGE_GUARD_PENDING.load(Ordering::Acquire),
        format_preview_resource_match_state(before),
        format_preview_resource_match_state(after),
        format_resource_911_page_guard_hit_chain(entry_ptr),
        context
    ));
}

fn log_resource_911_page_guard_line(message: String) {
    let index = RESOURCE_911_PAGE_GUARD_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_RESOURCE_911_PAGE_GUARD_LOGS {
        log::write_line(message);
    }
}

fn disarm_resource_911_page_guard(scope: &str, context: &str) {
    let was_active = RESOURCE_911_PAGE_GUARD_ACTIVE.swap(false, Ordering::AcqRel);
    RESOURCE_911_PAGE_GUARD_SINGLE_STEP_PENDING.store(false, Ordering::Release);
    if !was_active {
        return;
    }
    let page_base = RESOURCE_911_PAGE_GUARD_PAGE_BASE.load(Ordering::Relaxed);
    let page_size = RESOURCE_911_PAGE_GUARD_PAGE_SIZE.load(Ordering::Relaxed);
    let old_protect = RESOURCE_911_PAGE_GUARD_OLD_PROTECT.load(Ordering::Relaxed) as u32;
    if page_base != 0 && page_size != 0 && old_protect != 0 {
        let _ = unsafe { win::protect_memory(page_base, page_size, old_protect) };
    }
    log_resource_911_page_guard_line(format!(
        "resource911-page-guard-disarmed scope={scope} page_base=0x{page_base:x} page_size=0x{page_size:x} old_protect=0x{old_protect:x} context=[{}]",
        context
    ));
}

fn try_arm_resource_911_page_guard(entry: PreviewResourceResolveMatch, context: &str, scope: &str) {
    if !RESOURCE_911_PAGE_GUARD_WATCH_ENABLED
        || !RESOURCE_911_PAGE_GUARD_HANDLER_INSTALLED.load(Ordering::Acquire)
        || RESOURCE_911_PAGE_GUARD_ACTIVE.load(Ordering::Acquire)
        || RESOURCE_911_PAGE_GUARD_PENDING.load(Ordering::Acquire)
    {
        return;
    }
    RESOURCE_911_PAGE_GUARD_EVENTS.store(0, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_SINGLE_STEP_EVENTS.store(0, Ordering::Release);
    clear_resource_911_page_guard_hit_history();

    let Some(region) = win::memory_region(entry.entry_ptr) else {
        log_resource_911_page_guard_line(format!(
            "resource911-page-guard-arm-skip scope={scope} reason=no_region entry_ptr=0x{:x} context=[{}]",
            entry.entry_ptr, context
        ));
        return;
    };
    let page_base = entry.entry_ptr & !0xfff;
    let region_end = region.base.saturating_add(region.size);
    if page_base < region.base || page_base.saturating_add(0x1000) > region_end {
        log_resource_911_page_guard_line(format!(
            "resource911-page-guard-arm-skip scope={scope} reason=page_outside_region table_base=0x{:x} entry_index={} entry_ptr=0x{:x} region_base=0x{:x} region_size=0x{:x} context=[{}]",
            entry.table_base, entry.entry_index, entry.entry_ptr, region.base, region.size, context
        ));
        return;
    }

    let new_protect = region.protect | win::PAGE_GUARD;
    let Some(old_protect) = (unsafe { win::protect_memory(page_base, 0x1000, new_protect) }) else {
        log_resource_911_page_guard_line(format!(
            "resource911-page-guard-arm-skip scope={scope} reason=protect_failed table_base=0x{:x} entry_index={} entry_ptr=0x{:x} page_base=0x{page_base:x} protect=0x{:x} context=[{}]",
            entry.table_base, entry.entry_index, entry.entry_ptr, region.protect, context
        ));
        return;
    };

    RESOURCE_911_PAGE_GUARD_PAGE_BASE.store(page_base, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_PAGE_SIZE.store(0x1000, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_OLD_PROTECT.store(old_protect as usize, Ordering::Release);
    RESOURCE_911_PAGE_GUARD_ACTIVE.store(true, Ordering::Release);
    log_resource_911_page_guard_line(format!(
        "resource911-page-guard-armed-scope scope={scope} table_base=0x{:x} entry_index={} entry_ptr=0x{:x} page_base=0x{page_base:x} page_size=0x1000 state={} flags={} extra={} marker={} protect=0x{:x} old_protect=0x{:x} single_step_enabled={} single_step_cap={} context=[{}]",
        entry.table_base,
        entry.entry_index,
        entry.entry_ptr,
        format_optional_u32(entry.state),
        format_optional_hex_u32(entry.flags),
        format_optional_hex_u32(entry.extra),
        format_optional_hex_u32(entry.marker),
        new_protect,
        old_protect,
        RESOURCE_911_PAGE_GUARD_SINGLE_STEP_ENABLED,
        RESOURCE_911_PAGE_GUARD_SINGLE_STEP_CAP,
        context
    ));
}

fn flush_resource_911_page_guard_pending(boundary: &str, phase: &str, context: &str) {
    if !RESOURCE_911_PAGE_GUARD_WATCH_ENABLED
        || !RESOURCE_911_PAGE_GUARD_PENDING.swap(false, Ordering::AcqRel)
    {
        return;
    }

    let rip = RESOURCE_911_PAGE_GUARD_HIT_RIP.load(Ordering::Relaxed);
    let fault_addr = RESOURCE_911_PAGE_GUARD_HIT_FAULT.load(Ordering::Relaxed);
    let write_offset = RESOURCE_911_PAGE_GUARD_HIT_OFFSET.load(Ordering::Relaxed);
    let access_type = RESOURCE_911_PAGE_GUARD_HIT_ACCESS.load(Ordering::Relaxed);
    let rearmed = RESOURCE_911_PAGE_GUARD_HIT_REARMED.load(Ordering::Relaxed);
    let hit_seq = RESOURCE_911_PAGE_GUARD_HIT_SEQ.load(Ordering::Relaxed);
    let relevant = RESOURCE_911_PAGE_GUARD_RELEVANT_HIT.load(Ordering::Acquire);
    let old_snapshot = preview_resource_911_page_guard_hit_snapshot();
    let new_snapshot = preview_resource_911_watch_current();
    let rip_label = format_stack_frame(rip);
    let page_base = RESOURCE_911_PAGE_GUARD_PAGE_BASE.load(Ordering::Relaxed);
    let page_offset = fault_addr.saturating_sub(page_base);
    let entry_ptr = RESOURCE_911_PAGE_GUARD_HIT_ENTRY_PTR.load(Ordering::Relaxed);
    let entry_delta = format_resource_911_page_guard_delta(fault_addr, entry_ptr);
    let chain = format_resource_911_page_guard_hit_chain(entry_ptr);
    let hit_kind = match (access_type, relevant) {
        (0, _) => "resource911-page-guard-read-hit",
        (1, true) => "resource911-page-guard-write-hit",
        (1, false) => "resource911-page-guard-page-write-hit",
        (_, true) => "resource911-page-guard-hit",
        _ => "resource911-page-guard-page-hit",
    };
    let should_log_hit = match access_type {
        0 => {
            RESOURCE_911_PAGE_GUARD_READ_LOGS.fetch_add(1, Ordering::AcqRel)
                < RESOURCE_911_PAGE_GUARD_READ_LOG_CAP
        }
        1 => {
            RESOURCE_911_PAGE_GUARD_WRITE_LOGS.fetch_add(1, Ordering::AcqRel)
                < RESOURCE_911_PAGE_GUARD_WRITE_LOG_CAP
        }
        _ => true,
    };
    if relevant {
        if should_log_hit {
            log_resource_911_page_guard_line(format!(
                "{hit_kind} boundary={boundary} phase={phase} seq={hit_seq} rip={} access_type={} fault_addr=0x{fault_addr:x} page_offset=0x{page_offset:x} entry_delta={} write_offset={} rearm={} old=[{}] new=[{}] context=[{}]",
                rip_label,
                format_resource_911_page_guard_access(access_type),
                entry_delta,
                format_resource_911_page_guard_offset(write_offset),
                rearmed,
                format_preview_resource_match_state(old_snapshot),
                format_preview_resource_match_state(new_snapshot),
                context
            ));
        }
        if preview_resource_911_hidden_reset(old_snapshot, new_snapshot) {
            log_resource_911_page_guard_line(format!(
                "resource911-page-guard-exact-hit boundary={boundary} phase={phase} rip={} access_type={} fault_addr=0x{fault_addr:x} page_offset=0x{page_offset:x} entry_delta={} write_offset={} old=[{}] new=[{}] context=[{}]",
                rip_label,
                format_resource_911_page_guard_access(access_type),
                entry_delta,
                format_resource_911_page_guard_offset(write_offset),
                format_preview_resource_match_state(old_snapshot),
                format_preview_resource_match_state(new_snapshot),
                context
            ));
            log_resource_911_page_guard_line(format!(
                "resource911-page-guard-reset-confirmed boundary={boundary} phase={phase} seq={hit_seq} rip={} access_type={} fault_addr=0x{fault_addr:x} page_offset=0x{page_offset:x} entry_delta={} write_offset={} rearm={} old=[{}] new=[{}] context=[{}]",
                rip_label,
                format_resource_911_page_guard_access(access_type),
                entry_delta,
                format_resource_911_page_guard_offset(write_offset),
                rearmed,
                format_preview_resource_match_state(old_snapshot),
                format_preview_resource_match_state(new_snapshot),
                context
            ));
            log_resource_911_page_guard_line(format!(
                "resource911-reset-hit-chain boundary={boundary} phase={phase} chain=[{}]",
                chain
            ));
            if !log_resource_911_direct_reset_writes(boundary, phase, context) {
                log_resource_911_guard_missed_write(
                    boundary,
                    phase,
                    context,
                    old_snapshot,
                    new_snapshot,
                );
            }
        }
        return;
    }

    if should_log_hit {
        log_resource_911_page_guard_line(format!(
            "{hit_kind} boundary={boundary} phase={phase} seq={hit_seq} rip={} access_type={} fault_addr=0x{fault_addr:x} page_offset=0x{page_offset:x} entry_delta={} rearm={} old=[{}] new=[{}] context=[{}]",
            rip_label,
            format_resource_911_page_guard_access(access_type),
            entry_delta,
            rearmed,
            format_preview_resource_match_state(old_snapshot),
            format_preview_resource_match_state(new_snapshot),
            context
        ));
    }
    if preview_resource_911_hidden_reset(old_snapshot, new_snapshot) {
        log_resource_911_page_guard_line(format!(
            "resource911-page-hit-correlated-reset boundary={boundary} phase={phase} seq={hit_seq} rip={} access_type={} fault_addr=0x{fault_addr:x} page_offset=0x{page_offset:x} entry_delta={} rearm={} old=[{}] new=[{}] context=[{}]",
            rip_label,
            format_resource_911_page_guard_access(access_type),
            entry_delta,
            rearmed,
            format_preview_resource_match_state(old_snapshot),
            format_preview_resource_match_state(new_snapshot),
            context
        ));
        log_resource_911_page_guard_line(format!(
            "resource911-reset-hit-chain boundary={boundary} phase={phase} chain=[{}]",
            chain
        ));
        if !log_resource_911_direct_reset_writes(boundary, phase, context) {
            log_resource_911_guard_missed_write(
                boundary,
                phase,
                context,
                old_snapshot,
                new_snapshot,
            );
        }
        return;
    }
}

fn log_resource_911_watch(
    kind: &str,
    boundary: &str,
    phase: &str,
    current: PreviewResourceResolveMatch,
    previous: Option<PreviewResourceResolveMatch>,
    context: &str,
) {
    let index = COSTUME_PREVIEW_RESOURCE_WATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_WATCH_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    let previous = previous
        .map(|entry| format_preview_resource_match_state(Some(entry)))
        .unwrap_or_else(|| "none".to_string());
    log::write_line(format!(
        "{kind} boundary={boundary} phase={phase} table_base=0x{:x} entry_index={} entry_ptr=0x{:x} state={} flags={} extra={} marker={} before=[{}] after=[{}] context=[{}] page_guard_enabled={} known_callers={} frames={}",
        current.table_base,
        current.entry_index,
        current.entry_ptr,
        format_optional_u32(current.state),
        format_optional_hex_u32(current.flags),
        format_optional_hex_u32(current.extra),
        format_optional_hex_u32(current.marker),
        previous,
        format_preview_resource_match_state(Some(current)),
        context,
        RESOURCE_911_PAGE_GUARD_WATCH_ENABLED,
        format_known_preview_resource_callers(&frames),
        format_stack_frames(&frames, 8)
    ));
}

fn log_resource_911_watch_boundary(boundary: &str, phase: &str, context: &str) {
    if !RESOURCE_911_WATCH_ARMED.load(Ordering::Relaxed)
        && !context.contains("selected_layout=26")
        && !context.contains("active_layout=26")
    {
        return;
    }
    let Some(current) = preview_resource_911_watch_current() else {
        return;
    };
    let index = COSTUME_PREVIEW_RESOURCE_WATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_WATCH_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "resource911-watch-boundary boundary={boundary} phase={phase} table_base=0x{:x} entry_index={} entry_ptr=0x{:x} state={} flags={} extra={} marker={} current=[{}] context=[{}] page_guard_enabled={} known_callers={} frames={}",
        current.table_base,
        current.entry_index,
        current.entry_ptr,
        format_optional_u32(current.state),
        format_optional_hex_u32(current.flags),
        format_optional_hex_u32(current.extra),
        format_optional_hex_u32(current.marker),
        format_preview_resource_match_state(Some(current)),
        context,
        RESOURCE_911_PAGE_GUARD_WATCH_ENABLED,
        format_known_preview_resource_callers(&frames),
        format_stack_frames(&frames, 8)
    ));
}

fn log_resource_911_subboundary_reset(
    boundary: &str,
    phase: &str,
    previous: PreviewResourceResolveMatch,
    current: PreviewResourceResolveMatch,
    context: &str,
) -> Option<PreviewResourceResolveMatch> {
    if !boundary.contains("FUN_141582c30")
        || !preview_resource_911_hidden_reset(Some(previous), Some(current))
    {
        return None;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "resource911-subboundary boundary={boundary} phase={phase} before=[{}] after=[{}] context=[{}] known_callers={} frames={}",
        format_preview_resource_match_state(Some(previous)),
        format_preview_resource_match_state(Some(current)),
        context,
        format_known_preview_resource_callers(&frames),
        format_stack_frames(&frames, 8)
    ));
    if slot5_post_widget_cleanup_context_match(context)
        && LAST_SLOT5_PREVIEW_WIDGET_SEQ.load(Ordering::Relaxed) != 0
        && LAW_SLOT5_POST_WIDGET_TIMELINE_LOGS.fetch_add(1, Ordering::Relaxed)
            < MAX_LAW_SLOT5_POST_WIDGET_TIMELINE_LOGS
    {
        let widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
        if widget != 0 {
            log_preview_child_final(
                "slot5",
                "post-cleanup",
                widget,
                LAST_SLOT5_PREVIEW_VARIANT.load(Ordering::Relaxed) as u32,
                context,
                &frames,
            );
        }
        log::write_line(format!(
            "911-cleanup-post-widget boundary={boundary} phase={phase} before=[{}] after=[{}] post_widget=[{}] context=[{}] frames={}",
            format_preview_resource_match_state(Some(previous)),
            format_preview_resource_match_state(Some(current)),
            format_slot5_post_preview_widget_probe(),
            context,
            format_stack_frames(&frames, 7)
        ));
    }
    log_resource_911_page_guard_line(format!(
        "resource911-reset-hit-chain boundary={boundary} phase={phase} chain=[{}]",
        format_resource_911_page_guard_hit_chain(current.entry_ptr)
    ));
    if !log_resource_911_direct_reset_writes(boundary, phase, context) {
        log_resource_911_guard_missed_write(
            boundary,
            phase,
            context,
            Some(previous),
            Some(current),
        );
    }
    let reactivated = maybe_reactivate_911_after_post_widget_cleanup(
        boundary, phase, previous, current, context, &frames,
    );
    if reactivated.is_some() {
        let widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
        if widget != 0 {
            let preview_variant = LAST_SLOT5_PREVIEW_VARIANT.load(Ordering::Relaxed) as u32;
            log_preview_child_final(
                "slot5",
                "post-late-reactivate",
                widget,
                preview_variant,
                context,
                &frames,
            );
            log_slot5_preview_ready_for_render_phase(
                "post-late-reactivate",
                widget,
                preview_variant,
                read_preview_model_widget_trace(widget),
                context,
                &frames,
            );
            maybe_refresh_slot5_preview_after_late_ready(
                boundary,
                phase,
                widget,
                preview_variant,
                context,
                &frames,
            );
        }
    }
    reactivated
}

fn poll_resource_911_watch(boundary: &str, phase: &str, context: String) {
    flush_resource_911_page_guard_pending(boundary, phase, &context);
    let Some(current) = preview_resource_911_watch_current() else {
        return;
    };
    let previous = preview_resource_911_watch_previous();
    if previous.is_none() && resource_911_watch_arm_candidate(current) {
        RESOURCE_911_WATCH_ARMED.store(true, Ordering::Relaxed);
        store_preview_resource_911_watch_snapshot(current);
        log_resource_911_watch(
            "resource911-watch-armed",
            boundary,
            phase,
            current,
            None,
            &context,
        );
        return;
    }

    let Some(previous) = previous else {
        return;
    };
    if previous == current {
        return;
    }
    store_preview_resource_911_watch_snapshot(current);
    let post_reset_current =
        log_resource_911_subboundary_reset(boundary, phase, previous, current, &context);
    if post_reset_current.is_some() {
        return;
    }
    let frames = capture_stack_trace();
    log_resource_911_reactivation(
        "watch",
        &format!("{boundary}:{phase}"),
        "watch",
        LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
        None,
        &context,
        Some(previous),
        Some(current),
        &frames,
    );
    log_resource_911_watch(
        "resource911-watch-change",
        boundary,
        phase,
        current,
        Some(previous),
        &context,
    );
}

fn preview_resource_reset_boundary_context(owner: usize) -> String {
    let child58 = read_usize_field(owner, 0x58).filter(|address| *address != 0);
    let slot_b0 = read_usize_field(owner, 0xb0).filter(|address| *address != 0);
    let slot_120 = read_usize_field(owner, 0x120).filter(|address| *address != 0);
    let slot_190 = read_usize_field(owner, 0x190).filter(|address| *address != 0);
    let timeline = last_law_ready_timeline_trace_label().map(|(_, trace)| trace);
    format!(
        "owner=0x{owner:x} child58={} slot_b0={} slot_120={} slot_190={} last=[{}]",
        format_optional_address(child58),
        format_optional_address(slot_b0),
        format_optional_address(slot_120),
        format_optional_address(slot_190),
        preview_resource_trace_context(timeline)
    )
}

fn log_preview_resource_boundary(
    boundary: &str,
    state: usize,
    before_trace: Option<CostumeSceneTrace>,
    after_trace: Option<CostumeSceneTrace>,
    before_states: TrackedPreviewResourceStates,
    after_states: TrackedPreviewResourceStates,
    frames: &[usize],
) {
    let resource_changed = before_states.resource_911 != after_states.resource_911
        || before_states.resource_643 != after_states.resource_643
        || before_states.table != after_states.table;
    let hidden_reset =
        preview_resource_911_hidden_reset(before_states.resource_911, after_states.resource_911);
    let interesting_trace = before_trace.is_some_and(is_interesting_costume_scene_trace)
        || after_trace.is_some_and(is_interesting_costume_scene_trace)
        || before_trace
            .and_then(|trace| trace.selected_slot)
            .is_some_and(|slot| slot >= 3)
        || after_trace
            .and_then(|trace| trace.selected_slot)
            .is_some_and(|slot| slot >= 3)
        || scene_trace_mapped294(before_trace)
            == Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
        || scene_trace_mapped294(after_trace)
            == Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID);
    if !resource_changed && !interesting_trace {
        return;
    }

    let index = COSTUME_PREVIEW_RESOURCE_BOUNDARY_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_BOUNDARY_TRACE_LOGS {
        return;
    }

    let kind = if hidden_reset {
        "resource911-hidden-change"
    } else {
        "preview-resource-boundary"
    };
    log_resource_911_reactivation(
        "boundary",
        boundary,
        "boundary",
        LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
        None,
        &format_scene_boundary_context(state, after_trace),
        before_states.resource_911,
        after_states.resource_911,
        frames,
    );
    log::write_line(format!(
        "{kind} boundary={boundary} changed={resource_changed} caller_rva={} callsite={} before_resource=[{}] after_resource=[{}] before_context=[{}] after_context=[{}] known_callers={} frames={}",
        format_preview_resource_caller_rva(frames),
        preview_resource_callsite_label(&format_preview_resource_caller_rva(frames)),
        format_tracked_preview_resource_states(Some(before_states)),
        format_tracked_preview_resource_states(Some(after_states)),
        format_scene_boundary_context(state, before_trace),
        format_scene_boundary_context(state, after_trace),
        format_known_preview_resource_callers(frames),
        format_stack_frames(frames, 8)
    ));
}

fn preview_model_resource_state_candidate(
    preview_variant: u32,
    slot5_context: Option<(&'static str, CostumeObjectUpdateTrace)>,
) -> bool {
    preview_variant == u32::from(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID)
        || slot5_context.is_some_and(|(label, trace)| {
            label == "slot5-custom" && trace.selected_variant == Some(preview_variant as u16)
        })
}

fn log_preview_model_resource_state(
    preview_variant: u32,
    visible: i32,
    effective_visible: i32,
    layout_id: i32,
    fallback: u32,
    before: Option<TrackedPreviewResourceStates>,
    after: Option<TrackedPreviewResourceStates>,
    slot5_context: Option<(&'static str, CostumeObjectUpdateTrace)>,
    frames: &[usize],
) {
    if !preview_model_resource_state_candidate(preview_variant, slot5_context) {
        return;
    }
    let index = COSTUME_PREVIEW_MODEL_RESOURCE_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_MODEL_RESOURCE_STATE_LOGS {
        return;
    }
    let (label, context) = slot5_context
        .map(|(label, trace)| (label, format_costume_object_update_trace(trace)))
        .unwrap_or_else(|| {
            (
                "other",
                format_last_costume_object_update_context(
                    LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed),
                ),
            )
        });
    let caller_rva = format_preview_resource_caller_rva(frames);
    log::write_line(format!(
        "preview-model-resource-state label={label} preview_variant={preview_variant} visible={visible} effective_visible={effective_visible} layout={layout_id} fallback={fallback} caller_rva={caller_rva} callsite={} before=[{}] after=[{}] context=[{context}] frames={}",
        preview_resource_callsite_label(&caller_rva),
        format_tracked_preview_resource_states(before),
        format_tracked_preview_resource_states(after),
        format_stack_frames(frames, 7)
    ));
}

fn observed_preview_resource_key(entry: PreviewResourceResolveMatch) -> usize {
    let state = entry.state.unwrap_or(u32::MAX) as usize;
    let marker = entry.marker.unwrap_or(u32::MAX) as usize;
    let flags = entry.flags.unwrap_or(u32::MAX) as usize;
    (state & 0xffff) | ((marker & 0xffff) << 16) | ((flags & 0xffff) << 32)
}

fn log_preview_resource_911_change(
    phase: &str,
    caller_rva: &str,
    label: &str,
    preview_arg: u32,
    trace: Option<CostumeObjectUpdateTrace>,
    before: Option<PreviewResourceResolveMatch>,
    after: Option<PreviewResourceResolveMatch>,
    frames: &[usize],
) {
    let changed = before != after;
    if !changed {
        return;
    }
    let index = COSTUME_PREVIEW_RESOURCE_CHANGE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_CHANGE_LOGS {
        return;
    }
    log::write_line(format!(
        "resource911-entry-change phase={phase} label={label} caller_rva={caller_rva} callsite={} preview_arg={preview_arg} before=[{}] after=[{}] context=[{}] known_callers={} frames={}",
        preview_resource_callsite_label(caller_rva),
        format_preview_resource_match_state(before),
        format_preview_resource_match_state(after),
        preview_resource_trace_context(trace),
        format_known_preview_resource_callers(frames),
        format_stack_frames(frames, 7)
    ));
}

fn remember_preview_resource_911_observation(
    label: &str,
    caller_rva: &str,
    preview_arg: u32,
    trace: Option<CostumeObjectUpdateTrace>,
    after: Option<PreviewResourceResolveMatch>,
    frames: &[usize],
) {
    let Some(after) = after else {
        return;
    };
    let key = observed_preview_resource_key(after);
    let previous_seen = LAST_PREVIEW_RESOURCE_911_OBSERVED.swap(1, Ordering::Relaxed);
    let previous_state = LAST_PREVIEW_RESOURCE_911_OBSERVED_STATE.load(Ordering::Relaxed);
    let previous_flags = LAST_PREVIEW_RESOURCE_911_OBSERVED_FLAGS.load(Ordering::Relaxed);
    let previous_marker = LAST_PREVIEW_RESOURCE_911_OBSERVED_MARKER.load(Ordering::Relaxed);
    let previous_key = previous_state | (previous_marker << 16) | (previous_flags << 32);
    let previous_entry = (previous_seen != 0).then_some(PreviewResourceResolveMatch {
        table_base: after.table_base,
        entry_index: after.entry_index,
        entry_ptr: after.entry_ptr,
        state: preview_resource_watch_optional_u32(previous_state),
        flags: preview_resource_watch_optional_u32(previous_flags),
        extra: None,
        marker: preview_resource_watch_optional_u32(previous_marker),
    });
    LAST_PREVIEW_RESOURCE_911_OBSERVED_STATE
        .store(after.state.unwrap_or(u32::MAX) as usize, Ordering::Relaxed);
    LAST_PREVIEW_RESOURCE_911_OBSERVED_FLAGS
        .store(after.flags.unwrap_or(u32::MAX) as usize, Ordering::Relaxed);
    LAST_PREVIEW_RESOURCE_911_OBSERVED_MARKER
        .store(after.marker.unwrap_or(u32::MAX) as usize, Ordering::Relaxed);
    log_resource_911_reactivation(
        "observed",
        caller_rva,
        label,
        preview_arg,
        trace,
        &preview_resource_trace_context(trace),
        previous_entry,
        Some(after),
        frames,
    );
    if previous_seen == 0 || previous_key == key {
        return;
    }
    let index = COSTUME_PREVIEW_RESOURCE_CHANGE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_CHANGE_LOGS {
        return;
    }
    log::write_line(format!(
        "resource911-entry-change phase=observed label={label} caller_rva={caller_rva} callsite={} preview_arg={preview_arg} previous_key=0x{previous_key:x} after=[{}] context=[{}] known_callers={} frames={}",
        preview_resource_callsite_label(caller_rva),
        format_preview_resource_match_state(Some(after)),
        preview_resource_trace_context(trace),
        format_known_preview_resource_callers(frames),
        format_stack_frames(frames, 7)
    ));
}

fn record_preview_resource_911_resolve(table: usize, result: usize) {
    LAST_PREVIEW_RESOURCE_TABLE.store(table, Ordering::Relaxed);
    let label = last_law_ready_timeline_trace_label().map(|(label, _)| label);
    let matched =
        preview_resource_resolve_match(table, LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID);
    LAST_PREVIEW_RESOURCE_911_RESOLVE_LABEL
        .store(preview_diff_label_code(label), Ordering::Relaxed);
    LAST_PREVIEW_RESOURCE_911_RESOLVE_RESULT.store(result, Ordering::Relaxed);
    LAST_PREVIEW_RESOURCE_911_RESOLVE_STATE.store(
        matched.and_then(|entry| entry.state).unwrap_or(u32::MAX) as usize,
        Ordering::Relaxed,
    );
    LAST_PREVIEW_RESOURCE_911_RESOLVE_FLAGS.store(
        matched.and_then(|entry| entry.flags).unwrap_or(u32::MAX) as usize,
        Ordering::Relaxed,
    );
    LAST_PREVIEW_RESOURCE_911_RESOLVE_MARKER.store(
        matched.and_then(|entry| entry.marker).unwrap_or(u32::MAX) as usize,
        Ordering::Relaxed,
    );
    LAST_PREVIEW_RESOURCE_911_RESOLVE_SEQ.fetch_add(1, Ordering::Relaxed);
}

fn record_preview_resource_911_attach(queue: usize, result: usize) {
    let label = last_law_ready_timeline_trace_label().map(|(label, _)| label);
    let (len, has_911) = preview_resource_attach_queue_len_and_contains(
        queue,
        LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
    );
    LAST_PREVIEW_RESOURCE_911_ATTACH_LABEL.store(preview_diff_label_code(label), Ordering::Relaxed);
    LAST_PREVIEW_RESOURCE_911_ATTACH_RESULT.store(result, Ordering::Relaxed);
    LAST_PREVIEW_RESOURCE_911_ATTACH_LEN.store(len, Ordering::Relaxed);
    LAST_PREVIEW_RESOURCE_911_ATTACH_HAS_911.store(has_911 as usize, Ordering::Relaxed);
    LAST_PREVIEW_RESOURCE_911_ATTACH_SEQ.fetch_add(1, Ordering::Relaxed);
}

fn format_optional_preview_resource_field(value: usize) -> String {
    if value == u32::MAX as usize {
        "none".to_string()
    } else {
        value.to_string()
    }
}

fn format_optional_preview_resource_hex_field(value: usize) -> String {
    if value == u32::MAX as usize {
        "none".to_string()
    } else {
        format!("0x{:08x}", value as u32)
    }
}

fn format_preview_resource_911_state() -> String {
    let resolve_seq = LAST_PREVIEW_RESOURCE_911_RESOLVE_SEQ.load(Ordering::Relaxed);
    let attach_seq = LAST_PREVIEW_RESOURCE_911_ATTACH_SEQ.load(Ordering::Relaxed);
    let attach_len = LAST_PREVIEW_RESOURCE_911_ATTACH_LEN.load(Ordering::Relaxed);
    format!(
        "resolve_seq={} resolve_label={} resolve_result=0x{:x} state={} flags={} marker={} attach_seq={} attach_label={} attach_result=0x{:x} attach_len={} attach_has911={}",
        resolve_seq,
        format_preview_diff_label_code(LAST_PREVIEW_RESOURCE_911_RESOLVE_LABEL.load(Ordering::Relaxed)),
        LAST_PREVIEW_RESOURCE_911_RESOLVE_RESULT.load(Ordering::Relaxed),
        format_optional_preview_resource_field(LAST_PREVIEW_RESOURCE_911_RESOLVE_STATE.load(Ordering::Relaxed)),
        format_optional_preview_resource_hex_field(LAST_PREVIEW_RESOURCE_911_RESOLVE_FLAGS.load(Ordering::Relaxed)),
        format_optional_preview_resource_hex_field(LAST_PREVIEW_RESOURCE_911_RESOLVE_MARKER.load(Ordering::Relaxed)),
        attach_seq,
        format_preview_diff_label_code(LAST_PREVIEW_RESOURCE_911_ATTACH_LABEL.load(Ordering::Relaxed)),
        LAST_PREVIEW_RESOURCE_911_ATTACH_RESULT.load(Ordering::Relaxed),
        if attach_len == usize::MAX {
            "none".to_string()
        } else {
            attach_len.to_string()
        },
        LAST_PREVIEW_RESOURCE_911_ATTACH_HAS_911.load(Ordering::Relaxed) != 0
    )
}

fn format_preview_resource_911_live_state() -> String {
    format_preview_resource_match_state(read_tracked_preview_resource_states().resource_911)
}

fn format_preview_resource_1957_live_state() -> String {
    format_preview_resource_match_state(read_tracked_preview_resource_states().resource_1957)
}

fn slot5_custom_preview_variant() -> Option<u32> {
    current_law_extra_slot_probe_variant_id().map(u32::from)
}

fn is_slot5_preview_widget_final(preview_variant: u32, trace: PreviewModelWidgetTrace) -> bool {
    slot5_custom_preview_variant() == Some(preview_variant)
        && trace.mapped294 == Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
}

fn remember_slot5_preview_widget_state(
    widget: usize,
    preview_variant: u32,
    visible: i32,
    layout_id: i32,
    fallback: u32,
    trace: PreviewModelWidgetTrace,
    context: &str,
    frames: &[usize],
) {
    if !is_slot5_preview_widget_final(preview_variant, trace) {
        return;
    }

    let model_queue = read_usize_field(widget, 0x1e8).unwrap_or(0);
    let widget_queue = read_usize_field(widget, 0x288).unwrap_or(0);
    LAST_SLOT5_PREVIEW_WIDGET.store(widget, Ordering::Relaxed);
    LAST_SLOT5_PREVIEW_MODEL_QUEUE.store(model_queue, Ordering::Relaxed);
    LAST_SLOT5_PREVIEW_WIDGET_QUEUE.store(widget_queue, Ordering::Relaxed);
    LAST_SLOT5_PREVIEW_VARIANT.store(preview_variant as usize, Ordering::Relaxed);
    LAST_SLOT5_PREVIEW_VISIBLE.store(visible as usize, Ordering::Relaxed);
    LAST_SLOT5_PREVIEW_LAYOUT.store(layout_id as usize, Ordering::Relaxed);
    LAST_SLOT5_PREVIEW_FALLBACK.store(fallback as usize, Ordering::Relaxed);
    let seq = LAST_SLOT5_PREVIEW_WIDGET_SEQ.fetch_add(1, Ordering::Relaxed) + 1;

    if LAW_SLOT5_POST_WIDGET_TIMELINE_LOGS.fetch_add(1, Ordering::Relaxed)
        >= MAX_LAW_SLOT5_POST_WIDGET_TIMELINE_LOGS
    {
        return;
    }

    log_slot5_ui_label_state(
        "slot5-preview-widget-final",
        "remember",
        widget,
        LAST_SLOT5_COMPANION_PREVIEW_WIDGET.load(Ordering::Relaxed),
        context,
        frames,
    );

    log::write_line(format!(
        "slot5-post-preview-update-remember seq={seq} widget=0x{widget:x} preview_variant={preview_variant} model_queue={} widget_queue={} resource911_live=[{}] widget_live=[{}] res_probe=[{}] context=[{context}] frames={}",
        format_optional_address((model_queue != 0).then_some(model_queue)),
        format_optional_address((widget_queue != 0).then_some(widget_queue)),
        format_preview_resource_911_live_state(),
        format_preview_model_widget_trace(trace),
        format_preview_model_resource_probe(widget),
        format_stack_frames(frames, 6)
    ));
}

fn remember_oni_preview_widget_state(widget: usize, preview_variant: u32) {
    if preview_variant != u32::from(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID) {
        return;
    }
    let trace = read_preview_model_widget_trace(widget);
    if trace.mapped294 != Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID) {
        return;
    }

    LAST_ONI_PREVIEW_WIDGET.store(widget, Ordering::Relaxed);
    LAST_ONI_PREVIEW_MODEL_QUEUE.store(
        read_usize_field(widget, 0x1e8).unwrap_or(0),
        Ordering::Relaxed,
    );
    LAST_ONI_PREVIEW_WIDGET_QUEUE.store(
        read_usize_field(widget, 0x288).unwrap_or(0),
        Ordering::Relaxed,
    );
    LAST_ONI_PREVIEW_VARIANT.store(preview_variant as usize, Ordering::Relaxed);
    LAST_ONI_PREVIEW_WIDGET_SEQ.fetch_add(1, Ordering::Relaxed);
}

fn format_tracked_id_hits_in_range(base: usize, max_offset: usize) -> String {
    if base == 0 {
        return "none".to_string();
    }

    let mut hits = Vec::new();
    for offset in (0..=max_offset).step_by(size_of::<u32>()) {
        let Some(value) = read_u32_field(base, offset) else {
            continue;
        };
        if let Some(label) = preview_candidate_tracked_id_label(value) {
            hits.push(format!("u32+0x{offset:x}={value}:{label}"));
        }
        if hits.len() >= 24 {
            break;
        }
    }
    for offset in (0..=max_offset).step_by(size_of::<u16>()) {
        let Some(value) = read_u16_field(base, offset).map(u32::from) else {
            continue;
        };
        if let Some(label) = preview_candidate_tracked_id_label(value) {
            hits.push(format!("u16+0x{offset:x}={value}:{label}"));
        }
        if hits.len() >= 40 {
            break;
        }
    }

    if hits.is_empty() {
        "none".to_string()
    } else {
        hits.join("|")
    }
}

fn format_widget_res1d0_values(widget: usize) -> String {
    [
        (0x1d0usize, "res1d0_00"),
        (0x1d4, "res1d0_04"),
        (0x1d8, "res1d0_08"),
        (0x1dc, "res1d0_0c"),
        (0x1e0, "res1d0_10"),
        (0x1e4, "res1d0_14"),
    ]
    .into_iter()
    .map(|(offset, name)| {
        let value = read_u32_field(widget, offset);
        let label = value
            .and_then(preview_candidate_tracked_id_label)
            .unwrap_or("untracked");
        format!("{name}={}:{label}", format_optional_u32(value))
    })
    .collect::<Vec<_>>()
    .join(" ")
}

fn format_main_widget_binding_probe(widget: usize) -> String {
    if widget == 0 {
        return "main_widget=none".to_string();
    }

    let model_queue = read_usize_field(widget, 0x1e8).unwrap_or(0);
    let widget_queue = read_usize_field(widget, 0x288).unwrap_or(0);
    format!(
        "main=[{}] res1d0=[{}] model_queue=[{}] widget_queue=[{}] ids=[{}] pointer_refs=[{}] child8=[{}] children678=[{}]",
        format_preview_model_widget_trace(read_preview_model_widget_trace(widget)),
        format_widget_res1d0_values(widget),
        format_preview_resource_queue_summary(model_queue),
        format_preview_resource_queue_summary(widget_queue),
        format_tracked_id_hits_in_range(widget, 0x320),
        format_preview_render_result_pointer_refs(widget),
        format_preview_child8_snapshot(read_preview_child8_snapshot(widget)),
        format_preview_children_678(widget)
    )
}

fn log_main_widget_final_state(
    label: &str,
    widget: usize,
    expected_model: u32,
    preview_variant: u32,
    context: &str,
    frames: &[usize],
) {
    let index = MAIN_WIDGET_FINAL_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_MAIN_WIDGET_FINAL_LOGS {
        log::write_line(format!(
            "{label}-main-widget-final expected_model={expected_model} preview_variant={preview_variant} widget=0x{widget:x} {} resource911_live=[{}] resource1957_live=[{}] context=[{context}] frames={}",
            format_main_widget_binding_probe(widget),
            format_preview_resource_911_live_state(),
            format_preview_resource_1957_live_state(),
            format_stack_frames(frames, 8)
        ));
    }
    log_preview_model_binding_state(
        label,
        widget,
        expected_model,
        "widget-final",
        context,
        frames,
    );
}

fn log_preview_model_binding_state(
    label: &str,
    widget: usize,
    expected_model: u32,
    boundary: &str,
    context: &str,
    frames: &[usize],
) {
    let index = PREVIEW_MODEL_BINDING_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_PREVIEW_MODEL_BINDING_STATE_LOGS {
        return;
    }

    let expected_label = preview_candidate_tracked_id_label(expected_model).unwrap_or("model");
    let preview_resource = LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID;
    let has_expected = preview_render_result_has_tracked_value(widget, expected_model);
    let has_preview = preview_render_result_has_tracked_value(widget, preview_resource);
    let has_companion = preview_render_result_has_tracked_value(widget, 1957);
    log::write_line(format!(
        "preview-model-binding-state label={label} boundary={boundary} expected_model={expected_model}:{expected_label} has_expected_model={has_expected} preview_resource={preview_resource} has_preview_resource={has_preview} companion_resource=1957 has_companion_resource={has_companion} widget=0x{widget:x} {} resource911_live=[{}] resource1957_live=[{}] context=[{context}] frames={}",
        format_main_widget_binding_probe(widget),
        format_preview_resource_911_live_state(),
        format_preview_resource_1957_live_state(),
        format_stack_frames(frames, 8)
    ));
}

fn log_slot5_preview_ready_for_render(
    widget: usize,
    preview_variant: u32,
    trace: PreviewModelWidgetTrace,
    context: &str,
    frames: &[usize],
) {
    log_slot5_preview_ready_for_render_phase(
        "widget-final",
        widget,
        preview_variant,
        trace,
        context,
        frames,
    );
}

fn log_slot5_preview_ready_for_render_phase(
    phase: &str,
    widget: usize,
    preview_variant: u32,
    trace: PreviewModelWidgetTrace,
    context: &str,
    frames: &[usize],
) {
    let context_matches = if phase == "widget-final" {
        context.contains("selected_slot=4")
            && context.contains("selected_model_resource=292")
            && context.contains("object_pending=0x00")
    } else {
        slot5_post_widget_cleanup_context_match(context)
    };
    if slot5_custom_preview_variant() != Some(preview_variant)
        || trace.mapped294 != Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
        || !context_matches
    {
        return;
    }

    let Some(resource_911) = read_tracked_preview_resource_states().resource_911 else {
        return;
    };
    if resource_911.state != Some(1) || resource_911.marker != Some(0) {
        return;
    }
    let child8 = read_preview_child8_snapshot(widget);
    if child8.object.is_none() {
        return;
    }
    if !child8.is_slot5_good() {
        return;
    }

    maybe_run_slot5_widget_ready_temp_controller_refresh(widget, phase, context, frames);

    let index = SLOT5_PREVIEW_READY_FOR_RENDER_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_PREVIEW_READY_FOR_RENDER_LOGS {
        return;
    }

    log::write_line(format!(
        "slot5-preview-ready-for-render phase={phase} widget=0x{widget:x} preview_variant={preview_variant} model=292 mapped294=911 resource1957_live=[{}] resource911_live=[{}] child8=[{}] widget_live=[{}] res_probe=[{}] context=[{context}] frames={}",
        format_preview_resource_1957_live_state(),
        format_preview_resource_match_state(Some(resource_911)),
        format_preview_child8_snapshot(child8),
        format_preview_model_widget_trace(trace),
        format_preview_model_resource_probe(widget),
        format_stack_frames(frames, 8)
    ));
}

fn format_preview_resource_queue_summary(queue: usize) -> String {
    if queue == 0 {
        return "queue=none attach_len=none queue_has911=false state=[none]".to_string();
    }
    let (len, has_911) = preview_resource_attach_queue_len_and_contains(
        queue,
        LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
    );
    let len = if len == usize::MAX {
        "none".to_string()
    } else {
        len.to_string()
    };
    format!(
        "queue=0x{queue:x} attach_len={len} queue_has911={has_911} state=[{}]",
        format_preview_resource_attach_queue(queue)
    )
}

fn format_slot5_post_preview_widget_probe() -> String {
    let seq = LAST_SLOT5_PREVIEW_WIDGET_SEQ.load(Ordering::Relaxed);
    let widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
    let model_queue = LAST_SLOT5_PREVIEW_MODEL_QUEUE.load(Ordering::Relaxed);
    let widget_queue = LAST_SLOT5_PREVIEW_WIDGET_QUEUE.load(Ordering::Relaxed);
    let preview_variant = LAST_SLOT5_PREVIEW_VARIANT.load(Ordering::Relaxed);

    if widget == 0 {
        return format!(
            "post_widget_seq={seq} post_widget=none resource911_live=[{}]",
            format_preview_resource_911_live_state()
        );
    }

    let trace = read_preview_model_widget_trace(widget);
    format!(
        "post_widget_seq={seq} preview_variant={preview_variant} post_widget=0x{widget:x} model_queue=[{}] widget_queue=[{}] resource911_live=[{}] widget_live=[{}] child37=[{}] res_probe=[{}]",
        format_preview_resource_queue_summary(model_queue),
        format_preview_resource_queue_summary(widget_queue),
        format_preview_resource_911_live_state(),
        format_preview_model_widget_trace(trace),
        format_preview_widget_child37(trace),
        format_preview_model_resource_probe(widget)
    )
}

fn log_preview_child_final(
    label: &str,
    phase: &str,
    widget: usize,
    preview_variant: u32,
    context: &str,
    frames: &[usize],
) {
    let index = PREVIEW_CHILD_FINAL_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_PREVIEW_CHILD_FINAL_LOGS {
        return;
    }
    let trace = read_preview_model_widget_trace(widget);
    let child8_label = match label {
        "oni" => Some("oni"),
        "slot5" => Some("slot5"),
        _ => None,
    };
    log_preview_child8_state(
        child8_label,
        "preview-widget-final",
        phase,
        widget,
        context,
        frames,
    );
    if label == "slot5" {
        log::write_line(format!(
            "slot5-child8-final phase={phase} widget=0x{widget:x} child8=[{}] resource911_live=[{}] context=[{context}] frames={}",
            format_preview_child8_snapshot(read_preview_child8_snapshot(widget)),
            format_preview_resource_911_live_state(),
            format_stack_frames(frames, 8)
        ));
    }
    log::write_line(format!(
        "preview-child-final label={label} phase={phase} widget=0x{widget:x} preview_variant={preview_variant} mapped294={} resource911_live=[{}] children678=[{}] child37=[{}] context=[{context}] frames={}",
        format_optional_u32(trace.mapped294),
        format_preview_resource_911_live_state(),
        format_preview_children_678(widget),
        format_preview_widget_child37(trace),
        format_stack_frames(frames, 7)
    ));
}

fn preview_child_render_candidate_label(context: &str, widget: usize) -> Option<&'static str> {
    if context.contains("selected_variant=699")
        && context.contains("selected_slot=4")
        && context.contains("selected_model_resource=292")
        && context.contains("mapped294=911")
    {
        return Some("slot5");
    }
    if context.contains("selected_variant=586") {
        return Some("oni");
    }
    if context.contains("mapped294=911") {
        return Some("oni-or-law");
    }
    if widget != 0 && widget == LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed) {
        return Some("slot5-widget");
    }
    None
}

fn log_preview_child_render_candidate(
    boundary: &str,
    phase: &str,
    widget: usize,
    result: Option<usize>,
    context: &str,
    frames: &[usize],
) {
    let Some(label) = preview_child_render_candidate_label(context, widget) else {
        return;
    };
    if !matches!(label, "slot5" | "slot5-widget") {
        return;
    }
    let index = PREVIEW_CHILD_RENDER_CANDIDATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_PREVIEW_CHILD_RENDER_CANDIDATE_LOGS {
        return;
    }
    let trace = read_preview_model_widget_trace(widget);
    log::write_line(format!(
        "preview-child-render-candidate label={label} boundary={boundary} phase={phase} widget=0x{widget:x} result={} mapped294={} resource1957_live=[{}] resource911_live=[{}] children678=[{}] child37=[{}] context=[{}] frames={}",
        format_optional_address(result.filter(|address| *address != 0)),
        format_optional_u32(trace.mapped294),
        format_preview_resource_1957_live_state(),
        format_preview_resource_911_live_state(),
        format_preview_children_678(widget),
        format_preview_widget_child37(trace),
        context,
        format_stack_frames(frames, 7)
    ));
}

fn format_preview_render_result_qwords(result: usize) -> String {
    if result == 0 {
        return "none".to_string();
    }

    let qwords = (0..=0x80)
        .step_by(size_of::<usize>())
        .filter_map(|offset| {
            let value = read_usize_field(result, offset)?;
            (value != 0).then(|| format!("+0x{offset:x}=0x{value:x}"))
        })
        .take(14)
        .collect::<Vec<_>>();

    if qwords.is_empty() {
        "none".to_string()
    } else {
        qwords.join("|")
    }
}

fn format_preview_render_result_id_hits(result: usize) -> String {
    if result == 0 {
        return "none".to_string();
    }

    let mut hits = Vec::new();
    for offset in (0..=0x180).step_by(size_of::<u32>()) {
        let Some(value) = read_u32_field(result, offset) else {
            continue;
        };
        if let Some(label) = preview_candidate_tracked_id_label(value) {
            hits.push(format!("u32+0x{offset:x}={value}:{label}"));
        }
        if hits.len() >= 20 {
            break;
        }
    }
    for offset in (0..=0x180).step_by(size_of::<u16>()) {
        let Some(value) = read_u16_field(result, offset).map(u32::from) else {
            continue;
        };
        if let Some(label) = preview_candidate_tracked_id_label(value) {
            hits.push(format!("u16+0x{offset:x}={value}:{label}"));
        }
        if hits.len() >= 32 {
            break;
        }
    }

    if hits.is_empty() {
        "none".to_string()
    } else {
        hits.join("|")
    }
}

fn format_preview_render_result_pointer_refs(result: usize) -> String {
    if result == 0 {
        return "none".to_string();
    }

    let mut refs = Vec::new();
    for offset in (0..=0x100).step_by(size_of::<usize>()) {
        let Some(pointer) = read_usize_field(result, offset).filter(|address| *address != 0) else {
            continue;
        };
        let model30 = read_u32_field(pointer, 0x30);
        let color34 = read_u16_field(pointer, 0x34);
        let field38 = read_u32_field(pointer, 0x38);
        let field3c = read_u32_field(pointer, 0x3c);
        let ptr50 = read_usize_field(pointer, 0x50).filter(|address| *address != 0);
        let ptr58 = read_usize_field(pointer, 0x58).filter(|address| *address != 0);
        let interesting = model30.is_some_and(|value| matches!(value, 26 | 292 | 308))
            || ptr50.is_some()
            || ptr58.is_some();
        if !interesting {
            continue;
        }
        refs.push(format!(
            "+0x{offset:x}->0x{pointer:x}:model30={} color34={} field38={} field3c={} ptr50={} ptr58={}",
            format_optional_u32(model30),
            format_optional_u16_decimal(color34),
            format_optional_u32(field38),
            format_optional_u32(field3c),
            format_optional_address(ptr50),
            format_optional_address(ptr58)
        ));
        if refs.len() >= 10 {
            break;
        }
    }

    if refs.is_empty() {
        "none".to_string()
    } else {
        refs.join("|")
    }
}

fn preview_render_result_has_tracked_value(result: usize, target: u32) -> bool {
    if result == 0 {
        return false;
    }

    for offset in (0..=0x180).step_by(size_of::<u32>()) {
        if read_u32_field(result, offset) == Some(target) {
            return true;
        }
    }
    for offset in (0..=0x180).step_by(size_of::<u16>()) {
        if read_u16_field(result, offset).map(u32::from) == Some(target) {
            return true;
        }
    }
    for offset in (0..=0x100).step_by(size_of::<usize>()) {
        let Some(pointer) = read_usize_field(result, offset).filter(|address| *address != 0) else {
            continue;
        };
        for field_offset in [0x30usize, 0x34, 0x38, 0x3c, 0x40, 0x44] {
            if read_u32_field(pointer, field_offset) == Some(target) {
                return true;
            }
            if read_u16_field(pointer, field_offset).map(u32::from) == Some(target) {
                return true;
            }
        }
    }

    false
}

fn format_preview_render_asset_candidates(result: usize) -> String {
    if result == 0 {
        return "model_candidate=none texture_candidate=none material_candidate=none instance_candidate=none references=none".to_string();
    }

    let model_candidate = [292u32, 308, 26]
        .into_iter()
        .find(|id| preview_render_result_has_tracked_value(result, *id))
        .map(|id| preview_candidate_tracked_id_label(id).unwrap_or("model"))
        .unwrap_or("none");
    let resource_candidate = [911u32, 643, 1957, 4713]
        .into_iter()
        .find(|id| preview_render_result_has_tracked_value(result, *id))
        .map(|id| preview_candidate_tracked_id_label(id).unwrap_or("resource"))
        .unwrap_or("none");
    let engine_candidate = read_usize_field(result, 0x48)
        .filter(|address| preview_result_pointer_kind(Some(*address)) == "pointer");
    let sentinel50 = read_usize_field(result, 0x50).filter(|address| *address != 0);

    let mut texture_candidates = Vec::new();
    let mut material_candidates = Vec::new();
    for offset in (0..=0x100).step_by(size_of::<usize>()) {
        let Some(pointer) = read_usize_field(result, offset).filter(|address| *address != 0) else {
            continue;
        };
        let ptr50 = read_usize_field(pointer, 0x50).filter(|address| *address != 0);
        let ptr58 = read_usize_field(pointer, 0x58).filter(|address| *address != 0);
        let color34 = read_u16_field(pointer, 0x34);
        if let Some(ptr50) = ptr50 {
            texture_candidates.push(format!(
                "result+0x{offset:x}->0x{pointer:x}.ptr50=0x{ptr50:x}"
            ));
        }
        if let Some(ptr58) = ptr58 {
            texture_candidates.push(format!(
                "result+0x{offset:x}->0x{pointer:x}.ptr58=0x{ptr58:x}"
            ));
        }
        if color34.is_some() || ptr50.is_some() || ptr58.is_some() {
            material_candidates.push(format!(
                "result+0x{offset:x}->0x{pointer:x}:color34={} field38={} field3c={}",
                format_optional_u16_decimal(color34),
                format_optional_u32(read_u32_field(pointer, 0x38)),
                format_optional_u32(read_u32_field(pointer, 0x3c))
            ));
        }
        if texture_candidates.len() >= 4 && material_candidates.len() >= 4 {
            break;
        }
    }

    format!(
        "model_candidate={model_candidate} resource_candidate={resource_candidate} texture_candidate={} material_candidate={} instance_candidate={} engine48_candidate={} sentinel50={} references=[{}]",
        if texture_candidates.is_empty() {
            "none".to_string()
        } else {
            texture_candidates.join("|")
        },
        if material_candidates.is_empty() {
            "none".to_string()
        } else {
            material_candidates.join("|")
        },
        format_optional_address(engine_candidate),
        format_optional_address(engine_candidate),
        format_optional_address(sentinel50),
        format_preview_render_result_id_hits(result)
    )
}

fn preview_result_pointer_kind(value: Option<usize>) -> &'static str {
    match value {
        None | Some(0) => "none",
        Some(address) if address < 0x10_000 => "sentinel_or_flags",
        Some(address) if address >= 0x0000_8000_0000_0000 => "sentinel_or_flags",
        Some(_) => "pointer",
    }
}

fn preview_result_is_readable_pointer(result: usize) -> bool {
    preview_result_pointer_kind((result != 0).then_some(result)) == "pointer"
}

fn post_companion_result_diff_reason(result: usize) -> &'static str {
    if result == 0 {
        return "result_missing";
    }
    if !slot5_result1957_has_render_instance(result) {
        let instance = read_usize_field(result, 0x50);
        if preview_result_pointer_kind(instance) == "sentinel_or_flags" {
            return "instance_invalid";
        }
        if !preview_render_result_has_tracked_value(result, 292)
            && !preview_render_result_has_tracked_value(result, 308)
            && !preview_render_result_has_tracked_value(
                result,
                LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
            )
        {
            return "model_missing";
        }
        return "material_missing";
    }
    "same_as_oni"
}

fn format_post_companion_instance_pointers(result: usize) -> String {
    if !preview_result_is_readable_pointer(result) {
        return "none".to_string();
    }

    let mut parts = Vec::new();
    for offset in [0x50usize, 0xa8, 0xb0, 0xd8, 0xe0, 0xe8] {
        let Some(pointer) = read_usize_field(result, offset).filter(|address| *address != 0) else {
            parts.push(format!("+0x{offset:x}=none"));
            continue;
        };
        let pointer_kind = preview_result_pointer_kind(Some(pointer));
        if pointer_kind != "pointer" {
            parts.push(format!("+0x{offset:x}=0x{pointer:x}:{pointer_kind}"));
            continue;
        }
        parts.push(format!(
            "+0x{offset:x}=0x{pointer:x}:{pointer_kind}:qwords=[{}]:ids=[{}]:assets=[{}]:ptr_refs=[{}]",
            format_preview_render_result_qwords(pointer),
            format_preview_render_result_id_hits(pointer),
            format_preview_render_asset_candidates(pointer),
            format_preview_render_result_pointer_refs(pointer)
        ));
    }

    parts.join(" || ")
}

fn log_post_companion_instance_state(
    label: &str,
    boundary: &str,
    phase: &str,
    widget: usize,
    result: usize,
    result_kind: &str,
    context: &str,
    frames: &[usize],
) {
    if result_kind != "companion_queue_object" || !preview_result_is_readable_pointer(result) {
        return;
    }
    let index = POST_COMPANION_INSTANCE_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_POST_COMPANION_INSTANCE_STATE_LOGS {
        return;
    }
    let engine_candidate = read_usize_field(result, 0x48)
        .filter(|address| preview_result_pointer_kind(Some(*address)) == "pointer");
    let sentinel50 = read_usize_field(result, 0x50).filter(|address| *address != 0);
    log::write_line(format!(
        "post-companion-instance-state label={label} boundary={boundary} phase={phase} result=0x{result:x} result_kind={result_kind} widget={} engine48_candidate={} sentinel50={} sentinel50_kind={} deep=[{}] resource1957_live=[{}] resource911_live=[{}] main_widget=[{}] companion=[{}] context=[{}] frames={}",
        format_optional_address((widget != 0).then_some(widget)),
        format_optional_address(engine_candidate),
        format_optional_address(sentinel50),
        preview_result_pointer_kind(sentinel50),
        format_post_companion_instance_pointers(result),
        format_preview_resource_1957_live_state(),
        format_preview_resource_911_live_state(),
        format_slot5_post_preview_widget_probe(),
        if widget != 0 {
            format_companion_preview_widget_probe(widget)
        } else {
            "none".to_string()
        },
        context,
        format_stack_frames(frames, 8)
    ));
}

fn log_post_companion_result_state(
    label: &str,
    boundary: &str,
    phase: &str,
    widget: usize,
    result: usize,
    result_kind: &str,
    context: &str,
    frames: &[usize],
) {
    if result != 0 && !preview_result_is_readable_pointer(result) {
        return;
    }
    let index = POST_COMPANION_RESULT_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_POST_COMPANION_RESULT_STATE_LOGS {
        return;
    }

    let sentinel50 = read_usize_field(result, 0x50).filter(|address| *address != 0);
    let engine_candidate = read_usize_field(result, 0x48)
        .filter(|address| preview_result_pointer_kind(Some(*address)) == "pointer");
    log::write_line(format!(
        "post-companion-result-state label={label} boundary={boundary} phase={phase} result_kind={result_kind} widget={} result={} engine48_candidate={} sentinel50_kind={} qwords=[{}] id_hits=[{}] pointer_refs=[{}] {} companion=[{}] main_widget=[{}] resource1957_live=[{}] resource911_live=[{}] context=[{}] frames={}",
        format_optional_address((widget != 0).then_some(widget)),
        format_optional_address((result != 0).then_some(result)),
        format_optional_address(engine_candidate),
        preview_result_pointer_kind(sentinel50),
        format_preview_render_result_qwords(result),
        format_preview_render_result_id_hits(result),
        format_preview_render_result_pointer_refs(result),
        format_preview_render_asset_candidates(result),
        if widget != 0 {
            format_companion_preview_widget_probe(widget)
        } else {
            "none".to_string()
        },
        format_slot5_post_preview_widget_probe(),
        format_preview_resource_1957_live_state(),
        format_preview_resource_911_live_state(),
        context,
        format_stack_frames(frames, 8)
    ));
}

fn log_companion_render_result(
    label: &str,
    source: &str,
    boundary: &str,
    widget: usize,
    result: usize,
    context: &str,
    frames: &[usize],
) {
    if source != "companion" || label != "slot5" {
        return;
    }

    let index = COMPANION_RENDER_RESULT_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_COMPANION_RENDER_RESULT_LOGS {
        log::write_line(format!(
            "companion-render-result label={label} source={source} boundary={boundary} widget=0x{widget:x} result={} companion=[{}] resource1957_live=[{}] resource911_live=[{}] context=[{}] frames={}",
            format_optional_address((result != 0).then_some(result)),
            format_companion_preview_widget_probe(widget),
            format_preview_resource_1957_live_state(),
            format_preview_resource_911_live_state(),
            context,
            format_stack_frames(frames, 8)
        ));
    }

    let asset_index = COMPANION_RENDER_ASSET_LOGS.fetch_add(1, Ordering::Relaxed);
    if asset_index < MAX_COMPANION_RENDER_ASSET_LOGS {
        log::write_line(format!(
            "companion-render-asset label={label} source={source} boundary={boundary} widget=0x{widget:x} result={} {} pointer_refs=[{}] context=[{}] frames={}",
            format_optional_address((result != 0).then_some(result)),
            format_preview_render_asset_candidates(result),
            format_preview_render_result_pointer_refs(result),
            context,
            format_stack_frames(frames, 8)
        ));
    }

    let oni_baseline_also_missing_model = result != 0
        && !preview_render_result_has_tracked_value(result, 292)
        && !preview_render_result_has_tracked_value(result, 308);
    let resource_1957_stale = read_tracked_preview_resource_states()
        .resource_1957
        .is_some_and(|entry| entry.state == Some(0) && entry.marker == Some(1));
    if label == "slot5" && result != 0 && resource_1957_stale {
        let mismatch_index = COMPANION_RENDER_MISMATCH_LOGS.fetch_add(1, Ordering::Relaxed);
        if mismatch_index < MAX_COMPANION_RENDER_MISMATCH_LOGS {
            log::write_line(format!(
                "slot5-companion-render-state note=resource1957_stale_not_mismatch source={source} boundary={boundary} widget=0x{widget:x} result=0x{result:x} baseline_same_as_oni_model={oni_baseline_also_missing_model} resource1957_live=[{}] resource911_live=[{}] {} context=[{}] frames={}",
                format_preview_resource_1957_live_state(),
                format_preview_resource_911_live_state(),
                format_preview_render_asset_candidates(result),
                context,
                format_stack_frames(frames, 8)
            ));
        }
    }
}

fn log_companion_resource_flow(
    widget: usize,
    layout_id: u32,
    force_refresh: i32,
    scene_available: i32,
    result: usize,
    context: &str,
    frames: &[usize],
) {
    if widget == 0 || layout_id != u32::from(LAW_MASTER_LAYOUT_ID) {
        return;
    }
    let label = preview_child_render_candidate_label(context, widget).unwrap_or("other");
    if !matches!(label, "oni" | "slot5" | "oni-or-law")
        && !preview_companion_widget_shape_match(widget)
    {
        return;
    }

    let index = COMPANION_RESOURCE_FLOW_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COMPANION_RESOURCE_FLOW_LOGS {
        return;
    }

    let mapped_resource = read_u32_field(widget, 0x2d8);
    let queue = widget.checked_add(0x2c0);
    let tracked = read_tracked_preview_resource_states();
    let resource_1957 = preview_resource_resolve_match(tracked.table, 1957);
    log::write_line(format!(
        "companion-resource-flow label={label} widget=0x{widget:x} layout={layout_id} force_refresh={force_refresh} scene_available={scene_available} result=0x{result:x} mapped_resource={} queue={} queue_state=[{}] resource1957=[{}] resource911=[{}] main_widget=[{}] companion=[{}] context=[{}] frames={}",
        format_optional_u32(mapped_resource),
        format_optional_address(queue),
        queue.map(format_preview_resource_attach_queue).unwrap_or_else(|| "none".to_string()),
        format_preview_resource_match_state(resource_1957),
        format_preview_resource_match_state(tracked.resource_911),
        format_slot5_post_preview_widget_probe(),
        format_companion_preview_widget_probe(widget),
        context,
        format_stack_frames(frames, 8)
    ));

    if label == "slot5"
        && mapped_resource == Some(1957)
        && slot5_post_widget_cleanup_context_match(context)
    {
        log::write_line(format!(
            "slot5-companion-bound-to-1957 main_model=292 main_mapped=911 companion_mapped=1957 widget=0x{widget:x} queue_state=[{}] resource1957=[{}] resource911=[{}] context=[{}] frames={}",
            queue.map(format_preview_resource_attach_queue).unwrap_or_else(|| "none".to_string()),
            format_preview_resource_match_state(resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            context,
            format_stack_frames(frames, 8)
        ));
    }
}

fn log_preview_render_result(
    boundary: &str,
    widget: usize,
    result: usize,
    context: &str,
    frames: &[usize],
) {
    let Some(label) = preview_child_render_candidate_label(context, widget) else {
        return;
    };
    if !matches!(label, "slot5" | "slot5-widget") {
        return;
    }
    if result == 0 && label != "slot5" {
        return;
    }
    let source = preview_render_result_source(boundary, widget);
    log_companion_render_result(label, source, boundary, widget, result, context, frames);

    let index = PREVIEW_RENDER_RESULT_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_PREVIEW_RENDER_RESULT_LOGS {
        return;
    }

    log::write_line(format!(
        "preview-render-result label={label} preview-render-result-source={source} boundary={boundary} widget=0x{widget:x} result={} qwords=[{}] id_hits=[{}] pointer_refs=[{}] widget_probe=[{}] resource1957_live=[{}] resource911_live=[{}] children678=[{}] context=[{}] frames={}",
        format_optional_address((result != 0).then_some(result)),
        format_preview_render_result_qwords(result),
        format_preview_render_result_id_hits(result),
        format_preview_render_result_pointer_refs(result),
        format_preview_model_widget_trace(read_preview_model_widget_trace(widget)),
        format_preview_resource_1957_live_state(),
        format_preview_resource_911_live_state(),
        format_preview_children_678(widget),
        context,
        format_stack_frames(frames, 8)
    ));
}

fn preview_resource_match_is_active(entry: Option<PreviewResourceResolveMatch>) -> bool {
    entry.is_some_and(|entry| entry.state == Some(1) && entry.marker == Some(0))
}

fn preview_final_result_kind(
    result: usize,
    companion_widget: usize,
    resource_1957: Option<PreviewResourceResolveMatch>,
) -> &'static str {
    if result == 0 {
        return "none";
    }
    if resource_1957.is_some_and(|entry| result == entry.entry_ptr) {
        return "runtime_entry_1957";
    }
    if companion_widget != 0 {
        let queue = companion_widget.checked_add(0x2c0);
        if queue
            .and_then(preview_resource_attach_queue_end)
            .is_some_and(|end| end == result)
        {
            return "companion_queue_object";
        }
    }
    "other"
}

fn log_preview_final_result_kind(
    label: &str,
    boundary: &str,
    widget: usize,
    result: usize,
    context: &str,
    frames: &[usize],
) -> &'static str {
    let tracked = read_tracked_preview_resource_states();
    let mapped2d8 = read_u32_field(widget, 0x2d8);
    let queue = widget.checked_add(0x2c0);
    let queue_end = queue.and_then(preview_resource_attach_queue_end);
    let result_kind = preview_final_result_kind(result, widget, tracked.resource_1957);
    let counter = FINAL_RESULT_KIND_LOGS.fetch_add(1, Ordering::Relaxed);
    if counter < MAX_FINAL_RESULT_KIND_LOGS {
        log::write_line(format!(
            "{label}-final-result-kind boundary={boundary} result_kind={result_kind} widget=0x{widget:x} result={} mapped2d8={} queue_end={} queue_state=[{}] resource1957_live=[{}] resource911_live=[{}] context=[{}] frames={}",
            format_optional_address((result != 0).then_some(result)),
            format_optional_u32(mapped2d8),
            format_optional_address(queue_end),
            queue.map(format_preview_resource_attach_queue).unwrap_or_else(|| "none".to_string()),
            format_preview_resource_match_state(tracked.resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            context,
            format_stack_frames(frames, 8)
        ));
    }
    result_kind
}

fn log_oni_final_render_result_baseline(
    boundary: &str,
    widget: usize,
    result: usize,
    context: &str,
    frames: &[usize],
) {
    if !context.contains("selected_variant=586") || result == 0 {
        return;
    }
    let index = ONI_FINAL_RENDER_RESULT_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_ONI_FINAL_RENDER_RESULT_LOGS {
        return;
    }
    LAST_ONI_FINAL_RENDER_RESULT.store(result, Ordering::Relaxed);
    LAST_ONI_FINAL_RENDER_RESULT_WIDGET.store(widget, Ordering::Relaxed);
    let result_kind =
        log_preview_final_result_kind("oni", boundary, widget, result, context, frames);
    log_post_companion_result_state(
        "oni",
        boundary,
        "final-result",
        widget,
        result,
        result_kind,
        context,
        frames,
    );
    log_post_companion_instance_state(
        "oni",
        boundary,
        "final-result",
        widget,
        result,
        result_kind,
        context,
        frames,
    );
    log::write_line(format!(
        "oni-final-render-result-after-active-resources boundary={boundary} result_kind={result_kind} widget=0x{widget:x} result={} qwords=[{}] id_hits=[{}] pointer_refs=[{}] {} companion=[{}] main_widget=[{}] resource1957_live=[{}] resource911_live=[{}] context=[{}] frames={}",
        format_optional_address((result != 0).then_some(result)),
        format_preview_render_result_qwords(result),
        format_preview_render_result_id_hits(result),
        format_preview_render_result_pointer_refs(result),
        format_preview_render_asset_candidates(result),
        format_companion_preview_widget_probe(widget),
        format_slot5_post_preview_widget_probe(),
        format_preview_resource_1957_live_state(),
        format_preview_resource_911_live_state(),
        context,
        format_stack_frames(frames, 8)
    ));
}

fn log_slot5_final_render_result_after_active_resources(
    boundary: &str,
    widget: usize,
    result: usize,
    context: &str,
    frames: &[usize],
) {
    if !slot5_post_widget_cleanup_context_match(context) {
        return;
    }

    let tracked = read_tracked_preview_resource_states();
    let resource_911_active = preview_resource_match_is_active(tracked.resource_911);
    if !resource_911_active {
        return;
    }

    let main_widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
    let main_child8 = (main_widget != 0).then(|| read_preview_child8_snapshot(main_widget));
    let child8_good = main_child8.is_some_and(PreviewChild8Snapshot::is_slot5_good);
    if !child8_good {
        return;
    }
    if result != 0 && !preview_result_is_readable_pointer(result) {
        return;
    }

    let result_kind =
        log_preview_final_result_kind("slot5", boundary, widget, result, context, frames);
    if result != 0 && result_kind == "companion_queue_object" {
        LAST_SLOT5_FINAL_RESULT1957.store(result, Ordering::Relaxed);
        LAST_SLOT5_FINAL_RESULT1957_WIDGET.store(widget, Ordering::Relaxed);
    }

    log_post_companion_result_state(
        "slot5",
        boundary,
        "final-result",
        widget,
        result,
        result_kind,
        context,
        frames,
    );
    log_post_companion_instance_state(
        "slot5",
        boundary,
        "final-result",
        widget,
        result,
        result_kind,
        context,
        frames,
    );
    let result_text = format_optional_address((result != 0).then_some(result));
    let resource1957_status = if preview_resource_match_is_active(tracked.resource_1957) {
        "active"
    } else if tracked
        .resource_1957
        .is_some_and(|entry| entry.state == Some(0) && entry.marker == Some(1))
    {
        "stale_marked"
    } else {
        "other"
    };
    let reason = if result == 0 {
        "no_slot5_result_after_active_resources"
    } else {
        "result_after_active_resources"
    };

    log_loader_object_final_compare("slot5-final-preview-state", boundary, frames);

    let verdict_index = SLOT5_FINAL_PREVIEW_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if verdict_index < MAX_SLOT5_FINAL_PREVIEW_STATE_LOGS {
        log::write_line(format!(
            "slot5-final-preview-state ready=true result={result_text} result_kind={result_kind} resource911=active resource1957={resource1957_status} child8=good preview_visible=false reason={reason} boundary={boundary} widget=0x{widget:x} main_widget={} companion_widget={} context=[{}] frames={}",
            format_optional_address((main_widget != 0).then_some(main_widget)),
            format_optional_address((widget != 0).then_some(widget)),
            context,
            format_stack_frames(frames, 8)
        ));
    }

    let index = SLOT5_FINAL_RENDER_RESULT_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_FINAL_RENDER_RESULT_LOGS {
        return;
    }

    log::write_line(format!(
        "slot5-final-render-result-after-active-resources boundary={boundary} result_kind={result_kind} widget=0x{widget:x} result={result_text} qwords=[{}] id_hits=[{}] pointer_refs=[{}] {} companion=[{}] main_widget=[{}] main_children678=[{}] companion_children678=[{}] resource1957_live=[{}] resource911_live=[{}] context=[{}] frames={}",
        format_preview_render_result_qwords(result),
        format_preview_render_result_id_hits(result),
        format_preview_render_result_pointer_refs(result),
        format_preview_render_asset_candidates(result),
        format_companion_preview_widget_probe(widget),
        format_slot5_post_preview_widget_probe(),
        if main_widget != 0 {
            format_preview_children_678(main_widget)
        } else {
            "none".to_string()
        },
        format_preview_children_678(widget),
        format_preview_resource_match_state(tracked.resource_1957),
        format_preview_resource_match_state(tracked.resource_911),
        context,
        format_stack_frames(frames, 8)
    ));

    if result_kind == "companion_queue_object" {
        log::write_line(format!(
            "slot5-result-kind-fixed result_kind=companion_queue_object boundary={boundary} widget=0x{widget:x} result={result_text} companion=[{}] resource1957_live=[{}] resource911_live=[{}] context=[{}] frames={}",
            format_companion_preview_widget_probe(widget),
            format_preview_resource_match_state(tracked.resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            context,
            format_stack_frames(frames, 8)
        ));
        log_slot5_render_result_diff(boundary, widget, result, context, frames);
    } else if result_kind == "runtime_entry_1957" {
        log::write_line(format!(
            "slot5-result-kind-still-runtime-entry result_kind=runtime_entry_1957 boundary={boundary} widget=0x{widget:x} result={result_text} companion=[{}] resource1957_live=[{}] resource911_live=[{}] context=[{}] frames={}",
            format_companion_preview_widget_probe(widget),
            format_preview_resource_match_state(tracked.resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            context,
            format_stack_frames(frames, 8)
        ));
    }
}

fn log_slot5_render_result_diff(
    boundary: &str,
    widget: usize,
    slot5_result: usize,
    context: &str,
    frames: &[usize],
) {
    let index = SLOT5_RENDER_RESULT_DIFF_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_RENDER_RESULT_DIFF_LOGS {
        return;
    }
    let oni_result = LAST_ONI_FINAL_RENDER_RESULT.load(Ordering::Relaxed);
    let oni_widget = LAST_ONI_FINAL_RENDER_RESULT_WIDGET.load(Ordering::Relaxed);
    let oni_kind = preview_final_result_kind(
        oni_result,
        oni_widget,
        read_tracked_preview_resource_states().resource_1957,
    );
    let slot5_assets = format_preview_render_asset_candidates(slot5_result);
    let oni_assets = format_preview_render_asset_candidates(oni_result);
    let assets_match = slot5_assets == oni_assets;

    log::write_line(format!(
        "slot5-render-result-diff boundary={boundary} slot5_result={} oni_result={} same_result={} assets_match={assets_match} slot5_widget=0x{widget:x} oni_widget={} slot5_kind=companion_queue_object oni_kind={oni_kind} slot5_qwords=[{}] oni_qwords=[{}] slot5_assets=[{slot5_assets}] oni_assets=[{oni_assets}] slot5_companion=[{}] oni_companion=[{}] context=[{}] frames={}",
        format_optional_address((slot5_result != 0).then_some(slot5_result)),
        format_optional_address((oni_result != 0).then_some(oni_result)),
        slot5_result != 0 && slot5_result == oni_result,
        format_optional_address((oni_widget != 0).then_some(oni_widget)),
        format_preview_render_result_qwords(slot5_result),
        format_preview_render_result_qwords(oni_result),
        format_companion_preview_widget_probe(widget),
        if oni_widget != 0 {
            format_companion_preview_widget_probe(oni_widget)
        } else {
            "none".to_string()
        },
        context,
        format_stack_frames(frames, 8)
    ));
    if slot5_result != 0 && slot5_result == oni_result && assets_match {
        log::write_line(format!(
            "shared-companion-result-verified boundary={boundary} same_result=true assets_match=true result=0x{slot5_result:x} result_kind=companion_queue_object slot5_widget=0x{widget:x} oni_widget={} context=[{}] frames={}",
            format_optional_address((oni_widget != 0).then_some(oni_widget)),
            context,
            format_stack_frames(frames, 8)
        ));
    }
}

fn slot5_result1957_boundary_from_frames(frames: &[usize]) -> Option<&'static str> {
    const TARGETS: &[(usize, &str)] = &[
        (0x1589481, "game+0x1589481"),
        (0x16150fa, "game+0x16150fa"),
        (0x133c19f, "game+0x133c19f"),
        (0x1341d35, "game+0x1341d35"),
        (0x15e3b99, "game+0x15e3b99"),
        (0x15fc455, "game+0x15fc455"),
    ];
    frames.iter().find_map(|frame| {
        let rva = game_frame_rva(*frame)?;
        TARGETS
            .iter()
            .find_map(|(target, label)| (rva == *target).then_some(*label))
    })
}

fn slot5_result1957_has_render_instance(result: usize) -> bool {
    if result == 0 {
        return false;
    }
    if preview_render_result_has_tracked_value(result, 292)
        || preview_render_result_has_tracked_value(result, 308)
        || preview_render_result_has_tracked_value(
            result,
            LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
        )
    {
        return true;
    }
    read_usize_field(result, 0x48)
        .is_some_and(|value| preview_result_pointer_kind(Some(value)) == "pointer")
}

fn shared_result_draw_diff_reason(slot5_result: usize, oni_result: usize) -> &'static str {
    if slot5_result == 0 {
        return "result_missing";
    }
    if oni_result != 0 && slot5_result == oni_result {
        if !preview_render_result_has_tracked_value(slot5_result, 292) {
            return "same_result_no_slot_model";
        }
        return "no_diff_found";
    }

    let slot5_instance = read_usize_field(slot5_result, 0x48)
        .filter(|address| preview_result_pointer_kind(Some(*address)) == "pointer");
    let oni_instance = read_usize_field(oni_result, 0x48)
        .filter(|address| preview_result_pointer_kind(Some(*address)) == "pointer");
    if slot5_instance != oni_instance {
        return "instance_state_diff";
    }
    let slot5_material = format_preview_render_asset_candidates(slot5_result);
    let oni_material = format_preview_render_asset_candidates(oni_result);
    if slot5_material != oni_material {
        return "material_state_diff";
    }
    "no_diff_found"
}

fn shared_companion_result_verified(slot5_result: usize) -> bool {
    let oni_result = LAST_ONI_FINAL_RENDER_RESULT.load(Ordering::Relaxed);
    slot5_result != 0
        && oni_result != 0
        && slot5_result == oni_result
        && format_preview_render_asset_candidates(slot5_result)
            == format_preview_render_asset_candidates(oni_result)
}

fn log_slot5_shared_result_draw_diff(
    stack_boundary: &str,
    source_boundary: &str,
    phase: &str,
    widget: usize,
    result: usize,
    result_kind: &str,
    tracked: TrackedPreviewResourceStates,
    frames: &[usize],
) {
    if result_kind != "companion_queue_object" || !preview_result_is_readable_pointer(result) {
        return;
    }
    let index = SLOT5_SHARED_RESULT_DRAW_DIFF_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_SHARED_RESULT_DRAW_DIFF_LOGS {
        return;
    }

    let oni_result = LAST_ONI_FINAL_RENDER_RESULT.load(Ordering::Relaxed);
    let oni_widget = LAST_ONI_FINAL_RENDER_RESULT_WIDGET.load(Ordering::Relaxed);
    let reason = shared_result_draw_diff_reason(result, oni_result);
    let slot5_engine = read_usize_field(result, 0x48)
        .filter(|address| preview_result_pointer_kind(Some(*address)) == "pointer");
    let oni_engine = read_usize_field(oni_result, 0x48)
        .filter(|address| preview_result_pointer_kind(Some(*address)) == "pointer");
    let slot5_sentinel50 = read_usize_field(result, 0x50).filter(|address| *address != 0);
    let oni_sentinel50 = read_usize_field(oni_result, 0x50).filter(|address| *address != 0);
    log::write_line(format!(
        "slot5-shared-result-draw-diff boundary={stack_boundary} source_boundary={source_boundary} phase={phase} reason={reason} slot5_result={} oni_result={} same_result={} result_kind={result_kind} slot5_engine48={} oni_engine48={} slot5_sentinel50={} oni_sentinel50={} slot5_deep=[{}] oni_deep=[{}] resource1957_live=[{}] resource911_live=[{}] main_widget=[{}] slot5_companion=[{}] oni_companion=[{}] frames={}",
        format_optional_address((result != 0).then_some(result)),
        format_optional_address((oni_result != 0).then_some(oni_result)),
        result != 0 && result == oni_result,
        format_optional_address(slot5_engine),
        format_optional_address(oni_engine),
        format_optional_address(slot5_sentinel50),
        format_optional_address(oni_sentinel50),
        format_post_companion_instance_pointers(result),
        format_post_companion_instance_pointers(oni_result),
        format_preview_resource_match_state(tracked.resource_1957),
        format_preview_resource_match_state(tracked.resource_911),
        format_slot5_post_preview_widget_probe(),
        if widget != 0 {
            format_companion_preview_widget_probe(widget)
        } else {
            "none".to_string()
        },
        if oni_widget != 0 {
            format_companion_preview_widget_probe(oni_widget)
        } else {
            "none".to_string()
        },
        format_stack_frames(frames, 8)
    ));
}

fn log_main_widget_consumer_boundary(
    stack_boundary: &str,
    source_boundary: &str,
    phase: &str,
    result: usize,
    result_kind: &str,
    frames: &[usize],
) {
    let main_widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
    if main_widget == 0 {
        return;
    }
    let tracked = read_tracked_preview_resource_states();
    if !preview_resource_match_is_active(tracked.resource_911)
        || !read_preview_child8_snapshot(main_widget).is_slot5_good()
    {
        return;
    }
    let index = MAIN_WIDGET_CONSUMER_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_MAIN_WIDGET_CONSUMER_LOGS {
        return;
    }

    let oni_widget = LAST_ONI_PREVIEW_WIDGET.load(Ordering::Relaxed);
    log::write_line(format!(
        "main-widget-consumer boundary={stack_boundary} source_boundary={source_boundary} phase={phase} result={} result_kind={result_kind} slot5_expected_model=292 oni_expected_model=308 slot5_main=[{}] oni_main=[{}] slot5_binding=[{}] oni_binding=[{}] resource911_live=[{}] resource1957_live=[{}] frames={}",
        format_optional_address((result != 0).then_some(result)),
        format_main_widget_binding_probe(main_widget),
        if oni_widget != 0 {
            format_main_widget_binding_probe(oni_widget)
        } else {
            "main_widget=none".to_string()
        },
        format_tracked_id_hits_in_range(main_widget, 0x320),
        if oni_widget != 0 {
            format_tracked_id_hits_in_range(oni_widget, 0x320)
        } else {
            "none".to_string()
        },
        format_preview_resource_match_state(tracked.resource_911),
        format_preview_resource_match_state(tracked.resource_1957),
        format_stack_frames(frames, 8)
    ));
    log_preview_model_binding_state(
        "slot5",
        main_widget,
        u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
        stack_boundary,
        "main-widget-consumer",
        frames,
    );
}

fn log_slot5_result1957_consumer_boundary(boundary: &str, phase: &str, frames: &[usize]) {
    let Some(stack_boundary) = slot5_result1957_boundary_from_frames(frames) else {
        return;
    };
    let result = LAST_SLOT5_FINAL_RESULT1957.load(Ordering::Relaxed);
    if result == 0 {
        return;
    }
    if !preview_result_is_readable_pointer(result) {
        return;
    }
    let widget = LAST_SLOT5_FINAL_RESULT1957_WIDGET.load(Ordering::Relaxed);
    let tracked = read_tracked_preview_resource_states();
    if !preview_resource_match_is_active(tracked.resource_911) {
        return;
    }
    let main_widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
    let child8_good = main_widget != 0 && read_preview_child8_snapshot(main_widget).is_slot5_good();
    if !child8_good {
        return;
    }
    let result_kind = preview_final_result_kind(result, widget, tracked.resource_1957);

    let index = SLOT5_RESULT1957_CONSUMER_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_RESULT1957_CONSUMER_LOGS {
        log_post_companion_result_state(
            "slot5",
            stack_boundary,
            phase,
            widget,
            result,
            result_kind,
            "slot5-post-companion-boundary",
            frames,
        );
        log_post_companion_instance_state(
            "slot5",
            stack_boundary,
            phase,
            widget,
            result,
            result_kind,
            "slot5-post-companion-boundary",
            frames,
        );
        log::write_line(format!(
            "slot5-result1957-consumer boundary={stack_boundary} source_boundary={boundary} phase={phase} result1957_ptr=0x{result:x} widget={} qwords=[{}] id_hits=[{}] pointer_refs=[{}] {} resource1957_live=[{}] resource911_live=[{}] main_widget=[{}] companion=[{}] frames={}",
            format_optional_address((widget != 0).then_some(widget)),
            format_preview_render_result_qwords(result),
            format_preview_render_result_id_hits(result),
            format_preview_render_result_pointer_refs(result),
            format_preview_render_asset_candidates(result),
            format_preview_resource_match_state(tracked.resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            format_slot5_post_preview_widget_probe(),
            if widget != 0 {
                format_companion_preview_widget_probe(widget)
            } else {
                "none".to_string()
            },
            format_stack_frames(frames, 8)
        ));
    }

    let verdict_index = SLOT5_RESULT1957_VERDICT_LOGS.fetch_add(1, Ordering::Relaxed);
    if verdict_index >= MAX_SLOT5_RESULT1957_VERDICT_LOGS {
        return;
    }
    if result_kind == "companion_queue_object" {
        log_slot5_shared_result_draw_diff(
            stack_boundary,
            boundary,
            phase,
            widget,
            result,
            result_kind,
            tracked,
            frames,
        );
        log_main_widget_consumer_boundary(
            stack_boundary,
            boundary,
            phase,
            result,
            result_kind,
            frames,
        );
        if shared_companion_result_verified(result) {
            log::write_line(format!(
                "shared-companion-result-verified boundary={stack_boundary} source_boundary={boundary} phase={phase} same_result=true assets_match=true result=0x{result:x} result_kind={result_kind} resource1957_live=[{}] resource911_live=[{}] main_widget=[{}] frames={}",
                format_preview_resource_match_state(tracked.resource_1957),
                format_preview_resource_match_state(tracked.resource_911),
                format_slot5_post_preview_widget_probe(),
                format_stack_frames(frames, 8)
            ));
            return;
        }
    }
    if slot5_result1957_has_render_instance(result) {
        log::write_line(format!(
            "slot5-result1957-consumed boundary={stack_boundary} source_boundary={boundary} phase={phase} result1957_ptr=0x{result:x} {} resource1957_live=[{}] resource911_live=[{}] frames={}",
            format_preview_render_asset_candidates(result),
            format_preview_resource_match_state(tracked.resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            format_stack_frames(frames, 8)
        ));
    } else {
        let diff_index = SLOT5_POST_COMPANION_RESULT_DIFF_LOGS.fetch_add(1, Ordering::Relaxed);
        if diff_index < MAX_SLOT5_POST_COMPANION_RESULT_DIFF_LOGS {
            log::write_line(format!(
                "slot5-post-companion-result-diff boundary={stack_boundary} source_boundary={boundary} phase={phase} reason={} result_kind={result_kind} result={} instance_kind={} {} resource1957_live=[{}] resource911_live=[{}] main_widget=[{}] companion=[{}] frames={}",
                post_companion_result_diff_reason(result),
                format_optional_address((result != 0).then_some(result)),
                preview_result_pointer_kind(read_usize_field(result, 0x50).filter(|address| *address != 0)),
                format_preview_render_asset_candidates(result),
                format_preview_resource_match_state(tracked.resource_1957),
                format_preview_resource_match_state(tracked.resource_911),
                format_slot5_post_preview_widget_probe(),
                if widget != 0 {
                    format_companion_preview_widget_probe(widget)
                } else {
                    "none".to_string()
                },
                format_stack_frames(frames, 8)
            ));
        }
        log::write_line(format!(
            "slot5-result1957-not-consumed boundary={stack_boundary} source_boundary={boundary} phase={phase} result1957_ptr=0x{result:x} reason=no_instance_or_model_reference {} resource1957_live=[{}] resource911_live=[{}] frames={}",
            format_preview_render_asset_candidates(result),
            format_preview_resource_match_state(tracked.resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            format_stack_frames(frames, 8)
        ));
    }
}

fn preview_render_result_source(boundary: &str, widget: usize) -> &'static str {
    if widget != 0 && widget == LAST_SLOT5_COMPANION_PREVIEW_WIDGET.load(Ordering::Relaxed) {
        return "companion";
    }
    if preview_companion_widget_shape_match(widget) {
        return "companion";
    }
    if widget != 0 && widget == LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed) {
        return "main-widget";
    }
    if boundary.contains("companion") {
        return "companion";
    }
    "unknown"
}

fn preview_candidate_tracked_id_label(value: u32) -> Option<&'static str> {
    match value {
        292 => Some("model292"),
        294 => Some("preview294"),
        308 => Some("oni308"),
        586 => Some("variant586"),
        643 => Some("base643"),
        699 => Some("variant699"),
        911 => Some("preview911"),
        1957 => Some("companion1957"),
        2013 => Some("companion2013"),
        1982 => Some("companion1982"),
        1943 => Some("companion1943"),
        4713 => Some("companion4713"),
        65_535 => Some("ffff"),
        _ => None,
    }
}

fn format_preview_candidate_result_probe(result: usize) -> String {
    if result == 0 {
        return "result=none".to_string();
    }

    let pointers = (0..=0x80)
        .step_by(size_of::<usize>())
        .filter_map(|offset| {
            let value = read_usize_field(result, offset)?;
            (value != 0).then(|| format!("+0x{offset:x}=0x{value:x}"))
        })
        .take(18)
        .collect::<Vec<_>>()
        .join("|");
    let u32_hits = (0..=0x180)
        .step_by(size_of::<u32>())
        .filter_map(|offset| {
            let value = read_u32_field(result, offset)?;
            preview_candidate_tracked_id_label(value)
                .map(|label| format!("+0x{offset:x}={value}:{label}"))
        })
        .take(24)
        .collect::<Vec<_>>()
        .join("|");
    let u16_hits = (0..=0x180)
        .step_by(size_of::<u16>())
        .filter_map(|offset| {
            let value = u32::from(read_u16_field(result, offset)?);
            preview_candidate_tracked_id_label(value)
                .map(|label| format!("+0x{offset:x}={value}:{label}"))
        })
        .take(24)
        .collect::<Vec<_>>()
        .join("|");

    format!(
        "result=0x{result:x} ptrs=[{}] u32_hits=[{}] u16_hits=[{}]",
        if pointers.is_empty() {
            "none"
        } else {
            &pointers
        },
        if u32_hits.is_empty() {
            "none"
        } else {
            &u32_hits
        },
        if u16_hits.is_empty() {
            "none"
        } else {
            &u16_hits
        }
    )
}

fn log_preview_candidate_result_probe(
    boundary: &str,
    widget: usize,
    result: usize,
    context: &str,
    frames: &[usize],
) {
    if result == 0 {
        return;
    }
    let Some(label) = preview_child_render_candidate_label(context, widget) else {
        return;
    };
    if !matches!(label, "slot5" | "slot5-widget") {
        return;
    }
    let index = PREVIEW_CANDIDATE_RESULT_PROBE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_PREVIEW_CANDIDATE_RESULT_PROBE_LOGS {
        return;
    }
    log::write_line(format!(
        "preview-candidate-result-probe label={label} boundary={boundary} widget=0x{widget:x} {} widget_probe=[mapped294={} children678=[{}] child37=[{}]] resource911_live=[{}] context=[{}] frames={}",
        format_preview_candidate_result_probe(result),
        format_optional_u32(read_preview_model_widget_trace(widget).mapped294),
        format_preview_children_678(widget),
        format_preview_widget_child37(read_preview_model_widget_trace(widget)),
        format_preview_resource_911_live_state(),
        context,
        format_stack_frames(frames, 7)
    ));
}

fn preview_render_attach_label(trace: CostumeObjectUpdateTrace) -> Option<&'static str> {
    if trace.selected_variant == Some(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID)
        && trace.selected_model_resource == Some(308)
    {
        return Some("oni-render-attach");
    }

    if trace.selected_variant == current_law_extra_slot_probe_variant_id()
        && trace.selected_slot == Some(4)
        && trace.selected_model_resource == Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        && trace.selected_preview_mapped_resource
            == Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    {
        return Some("slot5-render-attach");
    }

    None
}

fn format_preview_render_attach_probe(trace: CostumeObjectUpdateTrace) -> String {
    let selected_variant = trace
        .selected_variant
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    let selected_slot = trace
        .selected_slot
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    let model_resource = trace
        .selected_model_resource
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    let mapped294 = trace
        .selected_preview_mapped_resource
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    format!(
        "selected_variant={selected_variant} selected_slot={selected_slot} model_resource={model_resource} mapped294={mapped294} resource911_live=[{}] {}",
        format_preview_resource_911_live_state(),
        format_slot5_post_preview_widget_probe()
    )
}

fn slot5_post_widget_cleanup_context_match(context: &str) -> bool {
    let variant = slot5_custom_preview_variant().unwrap_or(699);
    context.contains(&format!("selected_variant={variant}"))
        && context.contains("selected_slot=4")
        && (context.contains("selected_layout=26") || context.contains("active_layout=26"))
        && context.contains("selected_model_resource=292")
        && (context.contains("mapped294=911")
            || context.contains("selected_preview_mapped_resource=911")
            || context.contains("selected_preview_mapped=911"))
}

fn maybe_reactivate_911_before_render_candidate(
    boundary: &str,
    phase: &str,
    widget: usize,
    context: &str,
    frames: &[usize],
) -> Option<PreviewResourceResolveMatch> {
    if !LAW_SLOT5_REACTIVATE_911_BEFORE_RENDER_CANDIDATE_ENABLED {
        return None;
    }
    if !slot5_post_widget_cleanup_context_match(context) {
        return None;
    }

    let before = read_tracked_preview_resource_states().resource_911?;
    if before.state != Some(0) || before.marker != Some(1) {
        return None;
    }
    if read_u32_field(before.entry_ptr, PREVIEW_RESOURCE_SLOT_ID_OFFSET)
        != Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    {
        return None;
    }

    let state_written =
        write_u32_field_checked(before.entry_ptr, PREVIEW_RESOURCE_SLOT_STATE_OFFSET, 1);
    let marker_written =
        write_u32_field_checked(before.entry_ptr, PREVIEW_RESOURCE_SLOT_MARKER_OFFSET, 0);
    let after = preview_resource_resolve_match(
        before.table_base,
        LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
    );
    if let Some(after) = after {
        store_preview_resource_911_watch_snapshot(after);
    }

    let log_index = LAW_SLOT5_PRE_RENDER_REACTIVATE_911_LOGS.fetch_add(1, Ordering::Relaxed);
    if log_index < MAX_LAW_SLOT5_PRE_RENDER_REACTIVATE_911_LOGS {
        log::write_line(format!(
            "law-slot5-pre-render-reactivate-911 boundary={boundary} phase={phase} widget=0x{widget:x} state_written={state_written} marker_written={marker_written} before=[{}] after=[{}] context=[{context}] children678=[{}] child37=[{}] frames={}",
            format_preview_resource_match_state(Some(before)),
            format_preview_resource_match_state(after),
            format_preview_children_678(widget),
            format_preview_widget_child37(read_preview_model_widget_trace(widget)),
            format_stack_frames(frames, 7)
        ));
        log::write_line(format!(
            "slot5-render-candidate-after-reactivate boundary={boundary} phase={phase} widget=0x{widget:x} mapped294={} resource911_live=[{}] children678=[{}] child37=[{}] context=[{context}] frames={}",
            format_optional_u32(read_preview_model_widget_trace(widget).mapped294),
            format_preview_resource_911_live_state(),
            format_preview_children_678(widget),
            format_preview_widget_child37(read_preview_model_widget_trace(widget)),
            format_stack_frames(frames, 7)
        ));
    }

    after
}

fn maybe_reactivate_911_after_post_widget_cleanup(
    boundary: &str,
    phase: &str,
    previous: PreviewResourceResolveMatch,
    current: PreviewResourceResolveMatch,
    context: &str,
    frames: &[usize],
) -> Option<PreviewResourceResolveMatch> {
    if !LAW_SLOT5_REACTIVATE_911_AFTER_CLEANUP_ENABLED {
        return None;
    }
    if LAST_SLOT5_PREVIEW_WIDGET_SEQ.load(Ordering::Relaxed) == 0 {
        return None;
    }
    if !slot5_post_widget_cleanup_context_match(context) {
        return None;
    }
    if previous.state != Some(1)
        || previous.marker != Some(0)
        || current.state != Some(0)
        || current.marker != Some(1)
    {
        return None;
    }
    if read_u32_field(current.entry_ptr, PREVIEW_RESOURCE_SLOT_ID_OFFSET)
        != Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    {
        return None;
    }

    let log_index = LAW_SLOT5_LATE_REACTIVATE_911_LOGS.fetch_add(1, Ordering::Relaxed);
    let state_written =
        write_u32_field_checked(current.entry_ptr, PREVIEW_RESOURCE_SLOT_STATE_OFFSET, 1);
    let marker_written =
        write_u32_field_checked(current.entry_ptr, PREVIEW_RESOURCE_SLOT_MARKER_OFFSET, 0);
    let after = preview_resource_resolve_match(
        current.table_base,
        LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID,
    );
    if let Some(after) = after {
        store_preview_resource_911_watch_snapshot(after);
    }

    if log_index < MAX_LAW_SLOT5_LATE_REACTIVATE_911_LOGS {
        log::write_line(format!(
            "law-slot5-late-reactivate-911 boundary={boundary} phase={phase} state_written={state_written} marker_written={marker_written} before=[{}] cleanup_after=[{}] after=[{}] post_widget=[{}] context=[{context}] frames={}",
            format_preview_resource_match_state(Some(previous)),
            format_preview_resource_match_state(Some(current)),
            format_preview_resource_match_state(after),
            format_slot5_post_preview_widget_probe(),
            format_stack_frames(frames, 7)
        ));
    }

    after
}

fn slot5_companion_1957_reactivation_skip_reason(
    widget: usize,
    context: &str,
    queue: Option<usize>,
    queue_has_1957: bool,
    resource_1957: Option<PreviewResourceResolveMatch>,
) -> Option<&'static str> {
    if !LAW_SLOT5_REACTIVATE_STALE_COMPANION_1957_ENABLED {
        return Some("flag_disabled");
    }
    if !slot5_post_widget_cleanup_context_match(context) {
        return Some("context_mismatch");
    }
    if widget == 0 {
        return Some("widget_missing");
    }
    if read_u32_field(widget, 0x2dc) != Some(u32::from(LAW_MASTER_LAYOUT_ID)) {
        return Some("layout_mismatch");
    }
    if !matches!(read_u32_field(widget, 0x2d8), Some(1957 | 4713)) {
        return Some("companion_mapped_not_1957_or_4713");
    }
    if queue.is_none() {
        return Some("queue_missing");
    }
    if !queue_has_1957 {
        return Some("queue_missing_1957");
    }
    let Some(entry) = resource_1957 else {
        return Some("resource1957_missing");
    };
    if read_u32_field(entry.entry_ptr, PREVIEW_RESOURCE_SLOT_ID_OFFSET) != Some(1957) {
        return Some("entry_id_mismatch");
    }
    if entry.state != Some(0) || entry.marker != Some(1) {
        return Some("resource1957_not_stale");
    }

    None
}

fn maybe_restore_slot5_companion_mapped2d8_4713(
    boundary: &str,
    phase: &str,
    widget: usize,
    context: &str,
    frames: &[usize],
) {
    if !slot5_post_widget_cleanup_context_match(context) || widget == 0 {
        return;
    }

    let mapped2d8 = read_u32_field(widget, 0x2d8);
    let layout2dc = read_u32_field(widget, 0x2dc);
    let queue = widget.checked_add(0x2c0);
    let queue_has_1957 =
        queue.is_some_and(|queue| preview_resource_attach_queue_len_and_contains(queue, 1957).1);
    let tracked = read_tracked_preview_resource_states();

    let state_index = COMPANION_MAPPING_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if state_index < MAX_COMPANION_MAPPING_STATE_LOGS {
        log::write_line(format!(
            "companion-mapping-state boundary={boundary} phase={phase} label=slot5 companion4713_test={} widget=0x{widget:x} mapped2d8={} layout2dc={} queue_has1957={queue_has_1957} queue_state=[{}] resource1957_live=[{}] resource911_live=[{}] companion=[{}] context=[{context}] frames={}",
            if LAW_SLOT5_RESTORE_COMPANION_MAPPED2D8_4713_ENABLED {
                "on"
            } else {
                "off"
            },
            format_optional_u32(mapped2d8),
            format_optional_u32(layout2dc),
            queue.map(format_preview_resource_attach_queue).unwrap_or_else(|| "none".to_string()),
            format_preview_resource_match_state(tracked.resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            format_companion_preview_widget_probe(widget),
            format_stack_frames(frames, 7)
        ));
    }

    if !LAW_SLOT5_RESTORE_COMPANION_MAPPED2D8_4713_ENABLED {
        return;
    }

    if layout2dc != Some(u32::from(LAW_MASTER_LAYOUT_ID))
        || mapped2d8 != Some(1957)
        || !queue_has_1957
    {
        return;
    }

    let write_ok = write_u32_field_checked(widget, 0x2d8, 4713);
    let after_mapped2d8 = read_u32_field(widget, 0x2d8);
    let restore_index = SLOT5_COMPANION_MAPPING_RESTORE_LOGS.fetch_add(1, Ordering::Relaxed);
    if restore_index < MAX_SLOT5_COMPANION_MAPPING_RESTORE_LOGS {
        log::write_line(format!(
            "slot5-companion-mapping-restored boundary={boundary} phase={phase} widget=0x{widget:x} write_ok={write_ok} before_mapped2d8=1957 after_mapped2d8={} queue_state=[{}] resource1957_live=[{}] resource911_live=[{}] companion=[{}] context=[{context}] frames={}",
            format_optional_u32(after_mapped2d8),
            queue.map(format_preview_resource_attach_queue).unwrap_or_else(|| "none".to_string()),
            format_preview_resource_match_state(tracked.resource_1957),
            format_preview_resource_match_state(tracked.resource_911),
            format_companion_preview_widget_probe(widget),
            format_stack_frames(frames, 7)
        ));
    }
}

fn maybe_reactivate_stale_companion_1957(
    boundary: &str,
    phase: &str,
    widget: usize,
    context: &str,
    frames: &[usize],
) -> Option<PreviewResourceResolveMatch> {
    let queue = widget.checked_add(0x2c0);
    let queue_has_1957 =
        queue.is_some_and(|queue| preview_resource_attach_queue_len_and_contains(queue, 1957).1);
    let states = read_tracked_preview_resource_states();
    let before = states.resource_1957;
    let skip_reason = slot5_companion_1957_reactivation_skip_reason(
        widget,
        context,
        queue,
        queue_has_1957,
        before,
    );
    let should_log_skip = slot5_post_widget_cleanup_context_match(context)
        && (read_u32_field(widget, 0x2d8) == Some(1957) || queue_has_1957);

    if let Some(reason) = skip_reason {
        if should_log_skip
            && COMPANION_1957_REACTIVATE_LOGS.fetch_add(1, Ordering::Relaxed)
                < MAX_COMPANION_1957_REACTIVATE_LOGS
        {
            log::write_line(format!(
                "law-slot5-companion1957-reactivate-skip reason={reason} boundary={boundary} phase={phase} widget=0x{widget:x} queue={} queue_has1957={queue_has_1957} resource1957=[{}] resource911_live=[{}] companion=[{}] context=[{context}] frames={}",
                format_optional_address(queue),
                format_preview_resource_match_state(before),
                format_preview_resource_911_live_state(),
                format_companion_preview_widget_probe(widget),
                format_stack_frames(frames, 7)
            ));
        }
        return before;
    }

    let before = before?;
    let state_written =
        write_u32_field_checked(before.entry_ptr, PREVIEW_RESOURCE_SLOT_STATE_OFFSET, 1);
    let marker_written =
        write_u32_field_checked(before.entry_ptr, PREVIEW_RESOURCE_SLOT_MARKER_OFFSET, 0);
    let after = preview_resource_resolve_match(before.table_base, 1957);

    if COMPANION_1957_REACTIVATE_LOGS.fetch_add(1, Ordering::Relaxed)
        < MAX_COMPANION_1957_REACTIVATE_LOGS
    {
        log::write_line(format!(
            "law-slot5-companion1957-reactivate boundary={boundary} phase={phase} widget=0x{widget:x} state_written={state_written} marker_written={marker_written} before=[{}] after=[{}] queue_state=[{}] resource911_live=[{}] companion=[{}] context=[{context}] frames={}",
            format_preview_resource_match_state(Some(before)),
            format_preview_resource_match_state(after),
            queue.map(format_preview_resource_attach_queue).unwrap_or_else(|| "none".to_string()),
            format_preview_resource_911_live_state(),
            format_companion_preview_widget_probe(widget),
            format_stack_frames(frames, 7)
        ));
        log::write_line(format!(
            "slot5-companion-render-after-1957-reactivate boundary={boundary} phase={phase} widget=0x{widget:x} resource1957_live=[{}] resource911_live=[{}] companion=[{}] context=[{context}] frames={}",
            format_preview_resource_match_state(after),
            format_preview_resource_911_live_state(),
            format_companion_preview_widget_probe(widget),
            format_stack_frames(frames, 7)
        ));
    }

    after
}

fn maybe_refresh_slot5_preview_after_late_ready(
    boundary: &str,
    phase: &str,
    widget: usize,
    preview_variant: u32,
    context: &str,
    frames: &[usize],
) {
    if !LAW_SLOT5_REFRESH_PREVIEW_AFTER_LATE_READY_ENABLED
        || !LAW_SLOT5_REPLAY_COMPANION_PREVIEW_AFTER_READY_ENABLED
    {
        return;
    }
    if SLOT5_LATE_READY_REFRESH_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let original_address = COSTUME_COMPANION_PREVIEW_UPDATE_ORIGINAL.load(Ordering::Acquire);
    let stored_widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
    let widget_seq = LAST_SLOT5_PREVIEW_WIDGET_SEQ.load(Ordering::Relaxed);
    let companion_widget = LAST_SLOT5_COMPANION_PREVIEW_WIDGET.load(Ordering::Relaxed);
    let companion_seq = LAST_SLOT5_COMPANION_PREVIEW_WIDGET_SEQ.load(Ordering::Relaxed);
    let companion_layout = LAST_SLOT5_COMPANION_PREVIEW_LAYOUT.load(Ordering::Relaxed);
    let companion_force_refresh =
        LAST_SLOT5_COMPANION_PREVIEW_FORCE_REFRESH.load(Ordering::Relaxed);
    let companion_scene_available =
        LAST_SLOT5_COMPANION_PREVIEW_SCENE_AVAILABLE.load(Ordering::Relaxed);
    let trace = read_preview_model_widget_trace(widget);
    let resource_911 = read_tracked_preview_resource_states().resource_911;
    let child8 = read_preview_child8_snapshot(widget);
    let valid_companion = companion_widget != 0
        && companion_seq != 0
        && companion_layout == u32::from(LAW_MASTER_LAYOUT_ID) as usize
        && stored_widget == widget
        && widget_seq != 0;
    let already_replayed =
        SLOT5_LATE_READY_REPLAYED_COMPANION_SEQ.load(Ordering::Relaxed) == companion_seq;
    let ready = original_address != 0
        && valid_companion
        && !already_replayed
        && slot5_post_widget_cleanup_context_match(context)
        && slot5_custom_preview_variant() == Some(preview_variant)
        && trace.mapped294 == Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
        && resource_911.is_some_and(|state| state.state == Some(1) && state.marker == Some(0))
        && child8.is_slot5_good();
    let log_index = SLOT5_LATE_READY_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
    if log_index < MAX_SLOT5_LATE_READY_REFRESH_LOGS {
        log::write_line(format!(
            "law-slot5-refresh-after-late-ready-check boundary={boundary} phase={phase} ready={ready} replay=companion-preview widget=0x{widget:x} companion_widget={} preview_variant={preview_variant} original=0x{original_address:x} widget_seq={widget_seq} companion_seq={companion_seq} already_replayed={already_replayed} valid_companion={valid_companion} companion_args=[layout={} stored_force_refresh={} stored_scene_available={} replay_force_refresh=1 replay_scene_available=0] resource1957_live=[{}] resource911_live=[{}] child8=[{}] widget_live=[{}] companion_live=[{}] context=[{context}] frames={}",
            format_optional_address((companion_widget != 0).then_some(companion_widget)),
            format_optional_u32_from_usize(companion_layout),
            format_optional_i32_from_usize(companion_force_refresh),
            format_optional_i32_from_usize(companion_scene_available),
            format_preview_resource_1957_live_state(),
            format_preview_resource_match_state(resource_911),
            format_preview_child8_snapshot(child8),
            format_preview_model_widget_trace(trace),
            if companion_widget == 0 {
                "none".to_string()
            } else {
                format_companion_preview_widget_probe(companion_widget)
            },
            format_stack_frames(frames, 8)
        ));
    }
    if !ready {
        SLOT5_LATE_READY_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
        return;
    }

    SLOT5_LATE_READY_REPLAYED_WIDGET_SEQ.store(widget_seq, Ordering::Relaxed);
    SLOT5_LATE_READY_REPLAYED_COMPANION_SEQ.store(companion_seq, Ordering::Relaxed);
    let candidate_seq_before = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let before_probe = format_slot5_post_preview_widget_probe();
    let before_companion_probe = format_companion_preview_widget_probe(companion_widget);
    let before_resource = format_preview_resource_911_live_state();
    let before_resource_1957 = format_preview_resource_1957_live_state();
    let original: CostumeCompanionPreviewUpdateFn =
        unsafe { std::mem::transmute(original_address) };
    let result = unsafe { original(companion_widget, u32::from(LAW_MASTER_LAYOUT_ID), 1, 0) };
    let candidate_seq_after = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let consumed = candidate_seq_after > candidate_seq_before;
    let after_probe = format_slot5_post_preview_widget_probe();
    let after_companion_probe = format_companion_preview_widget_probe(companion_widget);
    let after_resource = format_preview_resource_911_live_state();
    let after_resource_1957 = format_preview_resource_1957_live_state();
    let log_index = SLOT5_LATE_READY_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
    if log_index < MAX_SLOT5_LATE_READY_REFRESH_LOGS {
        let line = if consumed {
            "slot5-companion-ready-consumed"
        } else {
            "slot5-companion-ready-not-consumed"
        };
        log::write_line(format!(
            "{line} source=companion-preview-replay render_candidate={consumed} boundary={boundary} phase={phase} widget=0x{widget:x} companion_widget=0x{companion_widget:x} preview_variant={preview_variant} widget_seq={widget_seq} companion_seq={companion_seq} result=0x{result:x} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} before_resource1957=[{before_resource_1957}] after_resource1957=[{after_resource_1957}] before_resource=[{before_resource}] after_resource=[{after_resource}] before_main=[{before_probe}] after_main=[{after_probe}] before_companion=[{before_companion_probe}] after_companion=[{after_companion_probe}] context=[{context}] frames={}",
            format_stack_frames(frames, 8)
        ));
        log::write_line(format!(
            "law-slot5-companion-preview-replay-after-ready result=0x{result:x} before_main=[{before_probe}] after_main=[{after_probe}] before_companion=[{before_companion_probe}] after_companion=[{after_companion_probe}] resource1957_before=[{before_resource_1957}] resource1957_after=[{after_resource_1957}] resource_before=[{before_resource}] resource_after=[{after_resource}] context=[{context}] frames={}",
            format_stack_frames(frames, 8)
        ));
    }

    SLOT5_LATE_READY_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
}

fn format_optional_i32_from_usize(value: usize) -> String {
    if value == usize::MAX {
        "none".to_string()
    } else {
        (value as i32).to_string()
    }
}

fn format_optional_u32_from_usize(value: usize) -> String {
    if value == usize::MAX {
        "none".to_string()
    } else {
        (value as u32).to_string()
    }
}

fn preview_model_widget_changed(
    before: Option<PreviewModelWidgetTrace>,
    after: PreviewModelWidgetTrace,
) -> bool {
    before.is_some_and(|before| {
        before.mapped294 != after.mapped294
            || before.visible2a0 != after.visible2a0
            || before.active2a1 != after.active2a1
            || before.field10 != after.field10
            || before.current_layout2dc != after.current_layout2dc
            || before.child58 != after.child58
            || before.child_b4 != after.child_b4
            || before.child_28 != after.child_28
            || before.child_30 != after.child_30
            || before.child_38 != after.child_38
            || before.child_48 != after.child_48
    })
}

fn log_preview_widget_final_state(
    line_label: &str,
    widget: usize,
    preview_variant: u32,
    visible: i32,
    effective_visible: i32,
    layout_id: i32,
    fallback: u32,
    after: PreviewModelWidgetTrace,
    after_resource: &str,
    context: &str,
    frames: &[usize],
) {
    if line_label == "oni-preview-widget-final" {
        remember_oni_preview_widget_state(widget, preview_variant);
        log_main_widget_final_state(
            "oni",
            widget,
            u32::from(LAW_EXTRA_SLOT_PREVIEW_TABLE_ONI_FALLBACK_SOURCE_ID),
            preview_variant,
            context,
            frames,
        );
    }
    if line_label == "slot5-preview-widget-final" {
        remember_slot5_preview_widget_state(
            widget,
            preview_variant,
            effective_visible,
            layout_id,
            fallback,
            after,
            context,
            frames,
        );
        log_main_widget_final_state(
            "slot5",
            widget,
            u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
            preview_variant,
            context,
            frames,
        );
    }
    let child_label = if line_label == "oni-preview-widget-final" {
        Some("oni")
    } else if line_label == "slot5-preview-widget-final" {
        Some("slot5")
    } else {
        None
    };
    if let Some(label) = child_label {
        log_preview_child_final(
            label,
            "widget-final",
            widget,
            preview_variant,
            context,
            frames,
        );
    }
    if line_label == "slot5-preview-widget-final" {
        log_slot5_preview_ready_for_render(widget, preview_variant, after, context, frames);
    }
    log::write_line(format!(
        "{line_label} widget=0x{widget:x} preview_variant={preview_variant} visible={visible} effective_visible={effective_visible} layout={layout_id} fallback={fallback} final=[{}] resource911_live=[{}] res_probe=[{after_resource}] context=[{context}] frames={}",
        format_preview_model_widget_trace(after),
        format_preview_resource_911_live_state(),
        format_stack_frames(frames, 6)
    ));
}

fn log_preview_widget_field_change(
    line_label: &str,
    widget: usize,
    preview_variant: u32,
    before: Option<PreviewModelWidgetTrace>,
    after: PreviewModelWidgetTrace,
    before_resource: Option<&str>,
    after_resource: &str,
    context: &str,
    frames: &[usize],
) {
    if !preview_model_widget_changed(before, after) {
        return;
    }
    let index = COSTUME_PREVIEW_WIDGET_FIELD_CHANGE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_WIDGET_FIELD_CHANGE_LOGS {
        return;
    }
    let before_text = before
        .map(format_preview_model_widget_trace)
        .unwrap_or_else(|| "none".to_string());
    let before_resource = before_resource.unwrap_or("none");
    log::write_line(format!(
        "widget-field-change label={line_label} widget=0x{widget:x} preview_variant={preview_variant} before=[{before_text}] after=[{}] before_res=[{before_resource}] after_res=[{after_resource}] resource911_live=[{}] context=[{context}] frames={}",
        format_preview_model_widget_trace(after),
        format_preview_resource_911_live_state(),
        format_stack_frames(frames, 7)
    ));
}

fn log_slot5_child8_preview_update_boundary(
    widget: usize,
    before_child8: Option<PreviewChild8Snapshot>,
    after_child8: Option<PreviewChild8Snapshot>,
    context: &str,
    frames: &[usize],
) {
    let index = SLOT5_CHILD8_PREVIEW_UPDATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_CHILD8_PREVIEW_UPDATE_LOGS {
        return;
    }
    let before = before_child8
        .map(format_preview_child8_snapshot)
        .unwrap_or_else(|| "none".to_string());
    let after = after_child8
        .map(format_preview_child8_snapshot)
        .unwrap_or_else(|| "none".to_string());
    let oni = load_preview_child8_snapshot("oni")
        .map(format_preview_child8_snapshot)
        .unwrap_or_else(|| "none".to_string());
    log::write_line(format!(
        "slot5-child8-before-preview-update widget=0x{widget:x} child8=[{before}] oni_baseline=[{oni}] resource911_live=[{}] context=[{context}] frames={}",
        format_preview_resource_911_live_state(),
        format_stack_frames(frames, 8)
    ));
    log::write_line(format!(
        "slot5-child8-after-preview-update widget=0x{widget:x} child8=[{after}] oni_baseline=[{oni}] resource911_live=[{}] context=[{context}] frames={}",
        format_preview_resource_911_live_state(),
        format_stack_frames(frames, 8)
    ));
    if let Some(before_child8) = before_child8 {
        poll_slot5_child8_watch("slot5-preview-update", "before", context, frames);
        if before_child8.is_slot5_good() {
            maybe_arm_slot5_child8_watch(
                "slot5-preview-update",
                "before",
                widget,
                before_child8,
                context,
                frames,
            );
        }
    }
    poll_slot5_child8_watch("slot5-preview-update", "after", context, frames);
    let restored = restore_slot5_child8_after_preview_update(widget, context, frames);
    if restored.is_some() {
        poll_slot5_child8_watch("slot5-preview-update", "after-restore", context, frames);
    }
}

fn format_preview_resource_resolve_table(table: usize, mapped_resource_id: u32) -> String {
    const SLOT_COUNT: usize = 0x521;
    const SLOT_STRIDE: usize = 0x1d0;
    const SLOT_BASE_OFFSET: usize = 0x10;
    const SLOT_ID_OFFSET: usize = 0x48;
    const SLOT_STATE_OFFSET: usize = 0x1c0;
    const SLOT_FLAGS_OFFSET: usize = 0x1c4;
    const SLOT_EXTRA_OFFSET: usize = 0x1c8;
    const SLOT_EMPTY_MARKER_OFFSET: usize = 0x1cc;

    if table == 0 {
        return "table=null".to_string();
    }

    let mut match_text = "match=none".to_string();
    let mut first_empty = None;
    let mut occupied = 0usize;

    for index in 0..SLOT_COUNT {
        let Some(entry) = table
            .checked_add(SLOT_BASE_OFFSET)
            .and_then(|base| base.checked_add(index.saturating_mul(SLOT_STRIDE)))
        else {
            break;
        };

        let id = read_u32_field(entry, SLOT_ID_OFFSET);
        let state = read_u32_field(entry, SLOT_STATE_OFFSET);
        let flags = read_u32_field(entry, SLOT_FLAGS_OFFSET);
        let extra = read_u32_field(entry, SLOT_EXTRA_OFFSET);
        let empty_marker = read_u32_field(entry, SLOT_EMPTY_MARKER_OFFSET);

        if id.is_some_and(|value| value != 0) || state.is_some_and(|value| value != 0) {
            occupied += 1;
        }

        if first_empty.is_none()
            && state == Some(0)
            && empty_marker == Some(0)
            && id.unwrap_or(0) == 0
        {
            first_empty = Some(index);
        }

        if id == Some(mapped_resource_id) {
            match_text = format!(
                "match=index:{index} entry=0x{entry:x} id={} state={} flags={} extra={} marker={}",
                format_optional_u32(id),
                format_optional_u32(state),
                format_optional_hex_u32(flags),
                format_optional_hex_u32(extra),
                format_optional_hex_u32(empty_marker)
            );
            break;
        }
    }

    format!(
        "table=0x{table:x} occupied={occupied} first_empty={} {match_text}",
        first_empty
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    )
}

fn preview_model_update_trace_scope(
    slot5_candidate: bool,
    global_candidate: bool,
    custom_candidate: bool,
    slot5_logged: usize,
    global_logged: usize,
    custom_logged: usize,
) -> Option<PreviewModelUpdateTraceScope> {
    if slot5_candidate && slot5_logged < MAX_COSTUME_PREVIEW_MODEL_SLOT5_TRACE_LOGS {
        return Some(PreviewModelUpdateTraceScope::Slot5);
    }

    if custom_candidate && custom_logged < MAX_COSTUME_PREVIEW_MODEL_CUSTOM_TRACE_LOGS {
        return Some(PreviewModelUpdateTraceScope::Custom);
    }

    if global_candidate && global_logged < MAX_COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS {
        return Some(PreviewModelUpdateTraceScope::Global);
    }

    None
}

fn preview_model_update_branch_label(visible: i32) -> &'static str {
    if visible == 0 {
        "conditional-or-hide"
    } else {
        "visible"
    }
}

fn preview_model_update_scope_label(scope: PreviewModelUpdateTraceScope) -> &'static str {
    match scope {
        PreviewModelUpdateTraceScope::Slot5 => "slot5",
        PreviewModelUpdateTraceScope::Global => "global",
        PreviewModelUpdateTraceScope::Custom => "custom",
    }
}

fn log_compact_preview_model_update_diff(
    widget: usize,
    preview_variant: u32,
    visible: i32,
    effective_visible: i32,
    layout_id: i32,
    fallback: u32,
    before: Option<PreviewModelWidgetTrace>,
    after: PreviewModelWidgetTrace,
    before_resource: Option<&str>,
    after_resource: &str,
    slot5_context: Option<(&'static str, CostumeObjectUpdateTrace)>,
    before_child8: Option<PreviewChild8Snapshot>,
    after_child8: Option<PreviewChild8Snapshot>,
    frames: &[usize],
) {
    let custom_variant = current_law_extra_slot_probe_variant_id().map(u32::from);
    let diff_kind = if layout_id == i32::from(LAW_MASTER_LAYOUT_ID)
        && preview_variant == u32::from(LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID)
    {
        Some((
            "oni-preview-update",
            &COSTUME_PREVIEW_MODEL_DIFF_ONI_LOGS,
            MAX_COSTUME_PREVIEW_MODEL_DIFF_ONI_LOGS,
        ))
    } else if slot5_context.is_some_and(|(label, _)| label == "slot5-custom")
        && custom_variant == Some(preview_variant)
    {
        Some((
            "slot5-preview-update",
            &COSTUME_PREVIEW_MODEL_DIFF_SLOT5_LOGS,
            MAX_COSTUME_PREVIEW_MODEL_DIFF_SLOT5_LOGS,
        ))
    } else {
        None
    };
    let Some((line_label, counter, max_logs)) = diff_kind else {
        return;
    };
    let index = counter.fetch_add(1, Ordering::Relaxed);
    if index >= max_logs {
        return;
    }
    let context = slot5_context
        .map(|(_, trace)| format_costume_object_update_trace(trace))
        .unwrap_or_else(|| {
            format_last_costume_object_update_context(
                LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed),
            )
        });
    let live_resource_911 = format_preview_resource_911_live_state();
    if line_label == "slot5-preview-update" {
        log_slot5_child8_preview_update_boundary(
            widget,
            before_child8,
            after_child8,
            &context,
            frames,
        );
    }
    log::write_line(format!(
        "{line_label} widget=0x{widget:x} preview_variant={preview_variant} visible={visible} effective_visible={effective_visible} layout={layout_id} fallback={fallback} before_mapped294={} mapped294={} before_visible2a0={} visible2a0={} before_active2a1={} active2a1={} child58={} child48={} child37=[{}] resource911=[{}] resource911_live=[{}] context=[{}] frames={}",
        before
            .and_then(|trace| trace.mapped294)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        format_optional_u32(after.mapped294),
        before
            .and_then(|trace| trace.visible2a0)
            .map(|value| format!("0x{value:02x}"))
            .unwrap_or_else(|| "none".to_string()),
        format_optional_u8(after.visible2a0),
        before
            .and_then(|trace| trace.active2a1)
            .map(|value| format!("0x{value:02x}"))
            .unwrap_or_else(|| "none".to_string()),
        format_optional_u8(after.active2a1),
        format_optional_address(after.child58),
        format_optional_address(after.child_48),
        format_preview_widget_child37(after),
        format_preview_resource_911_state(),
        live_resource_911,
        context,
        format_stack_frames(frames, 6)
    ));
    let final_label = if line_label == "oni-preview-update" {
        "oni-preview-widget-final"
    } else {
        "slot5-preview-widget-final"
    };
    log_preview_widget_final_state(
        final_label,
        widget,
        preview_variant,
        visible,
        effective_visible,
        layout_id,
        fallback,
        after,
        after_resource,
        &context,
        frames,
    );
    log_preview_widget_field_change(
        line_label,
        widget,
        preview_variant,
        before,
        after,
        before_resource,
        after_resource,
        &context,
        frames,
    );
}

fn log_costume_preview_branch_call(
    label: &str,
    widget: usize,
    layout_id: Option<u32>,
    preview_variant: Option<u32>,
    visible: Option<i32>,
    before: Option<PreviewModelWidgetTrace>,
    after: PreviewModelWidgetTrace,
    before_resource: Option<String>,
    after_resource: String,
) {
    if !preview_model_branch_trace_candidate(layout_id, preview_variant) {
        return;
    }
    let index = COSTUME_PREVIEW_BRANCH_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_BRANCH_TRACE_LOGS {
        return;
    }
    let before_text = before
        .map(format_preview_model_widget_trace)
        .unwrap_or_else(|| "none".to_string());
    let before_resource_text = before_resource.as_deref().unwrap_or("none").to_string();
    let frames = capture_stack_trace();
    log_slot5_result1957_consumer_boundary(label, "after-preview-branch", &frames);
    log::write_line(format!(
        "Costume preview branch label={label} widget=0x{widget:x} layout={} preview_variant={} visible={} before=[{before_text}] after=[{}] before_res=[{before_resource_text}] after_res=[{after_resource}] frames={}",
        layout_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        preview_variant
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        visible
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        format_preview_model_widget_trace(after),
        format_stack_frames(&frames, 6)
    ));
}

fn log_costume_preview_resource_attach(
    queue: usize,
    mapped_resource_id: u32,
    render_context: u32,
    widget_context: i32,
    result: usize,
    before: Option<String>,
    after: String,
    before_queue: Option<(usize, bool)>,
    after_queue: Option<(usize, bool)>,
    before_queue_probe: Option<&PreviewResourceQueueProbe>,
    after_queue_probe: Option<&PreviewResourceQueueProbe>,
    before_911: Option<PreviewResourceResolveMatch>,
    after_911: Option<PreviewResourceResolveMatch>,
) {
    if mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID {
        record_preview_resource_911_attach(queue, result);
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    if tracked_preview_resource_id(mapped_resource_id) {
        log_preview_resource_flow_attach(
            mapped_resource_id,
            render_context,
            widget_context,
            result,
            before_queue,
            after_queue,
            before_queue_probe,
            after_queue_probe,
            before_911,
            after_911,
            &frames,
        );
    }
    let index = COSTUME_PREVIEW_RESOURCE_ATTACH_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_ATTACH_TRACE_LOGS {
        return;
    }
    if let Some((label, trace)) = last_law_ready_timeline_trace_label() {
        log_law_ready_timeline(
            "resource-attach",
            format!(
                "slot_label={label} queue=0x{queue:x} mapped_resource={mapped_resource_id} render_context={render_context} widget_context={widget_context} result=0x{result:x} context=[{}] before=[{before}] after=[{after}] frames={}",
                format_costume_object_update_trace(trace),
                format_stack_frames(&frames, 6)
            ),
        );
    }
    log::write_line(format!(
        "Costume preview resource-attach queue=0x{queue:x} mapped_resource={mapped_resource_id} render_context={render_context} widget_context={widget_context} result=0x{result:x} before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 6)
    ));
}

fn log_costume_preview_resource_resolve(
    table: usize,
    mapped_resource_id: u32,
    render_context: u32,
    widget_context: i32,
    param5: i32,
    result: usize,
    before: Option<String>,
    after: String,
    before_match: Option<PreviewResourceResolveMatch>,
    after_match: Option<PreviewResourceResolveMatch>,
    before_911: Option<PreviewResourceResolveMatch>,
    after_911: Option<PreviewResourceResolveMatch>,
) {
    if tracked_preview_resource_id(mapped_resource_id) {
        LAST_PREVIEW_RESOURCE_TABLE.store(table, Ordering::Relaxed);
    }
    if mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID {
        record_preview_resource_911_resolve(table, result);
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    if tracked_preview_resource_id(mapped_resource_id) {
        log_preview_resource_flow_resolve(
            mapped_resource_id,
            render_context,
            widget_context,
            param5,
            result,
            before_match,
            after_match,
            before_911,
            after_911,
            &frames,
        );
    }
    let index = COSTUME_PREVIEW_RESOURCE_RESOLVE_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_RESOLVE_TRACE_LOGS {
        return;
    }
    log::write_line(format!(
        "Costume preview resource-resolve table=0x{table:x} mapped_resource={mapped_resource_id} render_context={render_context} widget_context={widget_context} param5={param5} result=0x{result:x} before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 7)
    ));
}

fn log_preview_resource_flow_attach(
    mapped_resource_id: u32,
    render_context: u32,
    widget_context: i32,
    result: usize,
    before_queue: Option<(usize, bool)>,
    after_queue: Option<(usize, bool)>,
    before_queue_probe: Option<&PreviewResourceQueueProbe>,
    after_queue_probe: Option<&PreviewResourceQueueProbe>,
    before_911: Option<PreviewResourceResolveMatch>,
    after_911: Option<PreviewResourceResolveMatch>,
    frames: &[usize],
) {
    let index = COSTUME_PREVIEW_RESOURCE_FLOW_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_FLOW_LOGS {
        return;
    }
    let (label, trace) = preview_resource_flow_label();
    let caller_rva = format_preview_resource_caller_rva(frames);
    let after_911 = maybe_reactivate_stale_preview_911(
        mapped_resource_id,
        &caller_rva,
        trace,
        before_queue_probe,
        after_queue_probe,
        after_911,
        frames,
    );
    let activation = format_resource_911_activation_attempt(before_911, after_911);
    let path_args = PreviewResource911PathArgs {
        op: "attach",
        caller_rva: &caller_rva,
        label,
        mapped_resource_id,
        render_context,
        widget_context,
        param5: None,
        result,
        before: before_911,
        after: after_911,
        before_queue,
        after_queue,
        before_queue_probe,
        after_queue_probe,
        trace,
        frames,
    };
    log_preview_resource_911_queue_state(&path_args);
    log_preview_resource_911_path(path_args);
    log_resource_911_reactivation(
        "attach",
        &caller_rva,
        label,
        mapped_resource_id,
        trace,
        &preview_resource_trace_context(trace),
        before_911,
        after_911,
        frames,
    );
    if mapped_resource_id == LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID {
        log_preview_resource_911_change(
            "sibling-after-643-attach",
            &caller_rva,
            label,
            mapped_resource_id,
            trace,
            before_911,
            after_911,
            frames,
        );
    }
    log::write_line(format!(
        "preview-resource-flow id={mapped_resource_id} label={label} op=attach caller_rva={caller_rva} callsite={} phase=before_after render_context={render_context} widget_context={widget_context} result=0x{result:x} activation911={activation} before=[{}] after=[{}] sibling911_before=[{}] sibling911_after=[{}] context=[{}] known_callers={} frames={}",
        preview_resource_callsite_label(&caller_rva),
        format_preview_resource_queue_state(before_queue),
        format_preview_resource_queue_state(after_queue),
        format_preview_resource_match_state(before_911),
        format_preview_resource_match_state(after_911),
        preview_resource_trace_context(trace),
        format_known_preview_resource_callers(frames),
        format_stack_frames(frames, 7)
    ));
}

fn log_preview_resource_flow_resolve(
    mapped_resource_id: u32,
    render_context: u32,
    widget_context: i32,
    param5: i32,
    result: usize,
    before_match: Option<PreviewResourceResolveMatch>,
    after_match: Option<PreviewResourceResolveMatch>,
    before_911: Option<PreviewResourceResolveMatch>,
    after_911: Option<PreviewResourceResolveMatch>,
    frames: &[usize],
) {
    let index = COSTUME_PREVIEW_RESOURCE_FLOW_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_RESOURCE_FLOW_LOGS {
        return;
    }
    let (label, trace) = preview_resource_flow_label();
    let caller_rva = format_preview_resource_caller_rva(frames);
    let activation = if mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID {
        format_resource_911_activation_attempt(before_match, after_match)
    } else {
        format_resource_911_activation_attempt(before_911, after_911)
    };
    if mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID {
        log_preview_resource_911_path(PreviewResource911PathArgs {
            op: "resolve",
            caller_rva: &caller_rva,
            label,
            mapped_resource_id,
            render_context,
            widget_context,
            param5: Some(param5),
            result,
            before: before_match,
            after: after_match,
            before_queue: None,
            after_queue: None,
            before_queue_probe: None,
            after_queue_probe: None,
            trace,
            frames,
        });
    }
    if mapped_resource_id == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID {
        log_resource_911_reactivation(
            "resolve",
            &caller_rva,
            label,
            mapped_resource_id,
            trace,
            &preview_resource_trace_context(trace),
            before_match,
            after_match,
            frames,
        );
        log_preview_resource_911_change(
            "resolve",
            &caller_rva,
            label,
            mapped_resource_id,
            trace,
            before_match,
            after_match,
            frames,
        );
        remember_preview_resource_911_observation(
            label,
            &caller_rva,
            mapped_resource_id,
            trace,
            after_match,
            frames,
        );
    } else if mapped_resource_id == LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID {
        log_resource_911_reactivation(
            "sibling-after-643-resolve",
            &caller_rva,
            label,
            mapped_resource_id,
            trace,
            &preview_resource_trace_context(trace),
            before_911,
            after_911,
            frames,
        );
        log_preview_resource_911_change(
            "sibling-after-643-resolve",
            &caller_rva,
            label,
            mapped_resource_id,
            trace,
            before_911,
            after_911,
            frames,
        );
    }
    log::write_line(format!(
        "preview-resource-flow id={mapped_resource_id} label={label} op=resolve caller_rva={caller_rva} callsite={} phase=before_after render_context={render_context} widget_context={widget_context} param5={param5} result=0x{result:x} activation911={activation} before=[{}] after=[{}] sibling911_before=[{}] sibling911_after=[{}] payload_after=[{}] sibling911_payload_after=[{}] context=[{}] known_callers={} frames={}",
        preview_resource_callsite_label(&caller_rva),
        format_preview_resource_match_state(before_match),
        format_preview_resource_match_state(after_match),
        format_preview_resource_match_state(before_911),
        format_preview_resource_match_state(after_911),
        format_preview_resource_entry_payload(after_match),
        format_preview_resource_entry_payload(after_911),
        preview_resource_trace_context(trace),
        format_known_preview_resource_callers(frames),
        format_stack_frames(frames, 7)
    ));
}

fn law_custom_preview_model_visible_arg(
    preview_variant: u32,
    custom_variant: Option<u16>,
    visible: i32,
    layout_id: i32,
) -> (i32, bool) {
    if !LAW_EXTRA_SLOT_FORCE_PREVIEW_VISIBLE_DIAGNOSTIC_ENABLED
        || visible != 0
        || layout_id != i32::from(LAW_MASTER_LAYOUT_ID)
    {
        return (visible, false);
    }

    if custom_variant.is_some_and(|variant| preview_variant == u32::from(variant)) {
        (1, true)
    } else {
        (visible, false)
    }
}

fn law_custom_preview_conditional_branch_diagnostic(
    preview_variant: u32,
    custom_variant: Option<u16>,
    visible: i32,
    layout_id: i32,
    fallback: u32,
) -> bool {
    LAW_EXTRA_SLOT_FORCE_CONDITIONAL_PREVIEW_DIAGNOSTIC_ENABLED
        && visible == 0
        && layout_id == i32::from(LAW_MASTER_LAYOUT_ID)
        && fallback == 0
        && custom_variant.is_some_and(|variant| preview_variant == u32::from(variant))
}

fn log_costume_preview_model_update(
    widget: usize,
    preview_variant: u32,
    visible: i32,
    effective_visible: i32,
    forced_visible: bool,
    forced_conditional: bool,
    layout_id: i32,
    fallback: u32,
    before: Option<PreviewModelWidgetTrace>,
    after: PreviewModelWidgetTrace,
    before_resource: Option<String>,
    after_resource: String,
    before_preview_resources: Option<TrackedPreviewResourceStates>,
    after_preview_resources: Option<TrackedPreviewResourceStates>,
    before_child8: Option<PreviewChild8Snapshot>,
    after_child8: Option<PreviewChild8Snapshot>,
) {
    let global_candidate =
        preview_model_update_global_trace_candidate(preview_variant, visible, layout_id, fallback);
    let custom_candidate = preview_model_update_custom_trace_candidate(preview_variant, fallback);
    let slot5_context = last_law_ready_timeline_trace_label();
    let slot5_candidate = slot5_context.is_some_and(|(label, _)| label == "slot5-custom");
    let Some(scope) = preview_model_update_trace_scope(
        slot5_candidate,
        global_candidate,
        custom_candidate,
        COSTUME_PREVIEW_MODEL_SLOT5_TRACE_LOGS.load(Ordering::Relaxed),
        COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS.load(Ordering::Relaxed),
        COSTUME_PREVIEW_MODEL_CUSTOM_TRACE_LOGS.load(Ordering::Relaxed),
    ) else {
        return;
    };
    match scope {
        PreviewModelUpdateTraceScope::Slot5 => {
            let index = COSTUME_PREVIEW_MODEL_SLOT5_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
            if index >= MAX_COSTUME_PREVIEW_MODEL_SLOT5_TRACE_LOGS {
                return;
            }
        }
        PreviewModelUpdateTraceScope::Global => {
            let index = COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
            if index >= MAX_COSTUME_PREVIEW_MODEL_GLOBAL_TRACE_LOGS {
                return;
            }
        }
        PreviewModelUpdateTraceScope::Custom => {
            let index = COSTUME_PREVIEW_MODEL_CUSTOM_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
            if index >= MAX_COSTUME_PREVIEW_MODEL_CUSTOM_TRACE_LOGS {
                return;
            }
        }
    };
    let before_text = before
        .map(format_preview_model_widget_trace)
        .unwrap_or_else(|| "none".to_string());
    let before_resource_text = before_resource.as_deref().unwrap_or("none").to_string();
    let gate_probe = format_preview_model_update_gate_probe(layout_id);
    let frames = capture_stack_trace();
    log_compact_preview_model_update_diff(
        widget,
        preview_variant,
        visible,
        effective_visible,
        layout_id,
        fallback,
        before,
        after,
        before_resource.as_deref(),
        &after_resource,
        slot5_context,
        before_child8,
        after_child8,
        &frames,
    );
    log_preview_model_resource_state(
        preview_variant,
        visible,
        effective_visible,
        layout_id,
        fallback,
        before_preview_resources,
        after_preview_resources,
        slot5_context,
        &frames,
    );
    if let Some((label, trace)) = slot5_context {
        log_law_ready_timeline(
            "preview-model-update",
            format!(
                "slot_label={label} scope={} widget=0x{widget:x} preview_variant={preview_variant} visible={visible} effective_visible={effective_visible} layout={layout_id} fallback={fallback} context=[{}] before=[{before_text}] after=[{}] before_res=[{before_resource_text}] after_res=[{after_resource}] frames={}",
                preview_model_update_scope_label(scope),
                format_costume_object_update_trace(trace),
                format_preview_model_widget_trace(after),
                format_stack_frames(&frames, 6)
            ),
        );
    }
    log::write_line(format!(
        "Costume preview model-update scope={} widget=0x{widget:x} preview_variant={preview_variant} visible={visible} effective_visible={effective_visible} forced_visible={forced_visible} forced_conditional={forced_conditional} branch={} layout={layout_id} fallback={fallback} context=[{}] gates=[{gate_probe}] before=[{before_text}] after=[{}] before_res=[{before_resource_text}] after_res=[{after_resource}] frames={}",
        preview_model_update_scope_label(scope),
        if forced_conditional {
            "conditional-visible-diagnostic"
        } else {
            preview_model_update_branch_label(effective_visible)
        },
        format_last_costume_object_update_context(LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed)),
        format_preview_model_widget_trace(after),
        format_stack_frames(&frames, 6)
    ));
}

fn log_costume_companion_preview_update(
    widget: usize,
    layout_id: u32,
    force_refresh: i32,
    scene_available: i32,
    result: usize,
    before: Option<String>,
    after: String,
) {
    if !is_interesting_companion_preview_update(widget, layout_id, force_refresh, scene_available) {
        return;
    }
    let index = COSTUME_COMPANION_PREVIEW_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_COMPANION_PREVIEW_TRACE_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let layout_preview = preview_model_update_layout_preview_value(layout_id as i32);
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume companion preview-update widget=0x{widget:x} layout={layout_id} layout_preview={} force_refresh={force_refresh} scene_available={scene_available} result=0x{result:x} before=[{before}] after=[{after}] frames={}",
        format_optional_u32(layout_preview),
        format_stack_frames(&frames, 7)
    ));
}

fn read_launch_costume_state_trace(state: usize) -> Option<LaunchCostumeStateTrace> {
    if state == 0 {
        return None;
    }

    let ptr_f0 = read_usize_field(state, 0xf0).filter(|address| *address != 0);
    let ptr_430 = read_usize_field(state, 0x430).filter(|address| *address != 0);
    Some(LaunchCostumeStateTrace {
        state,
        index4: read_u16_field(state, 0x04),
        flags20: read_u32_field(state, 0x20),
        id_1d0: read_u32_field(state, 0x1d0),
        id_1d4: read_u32_field(state, 0x1d4),
        id_1d8: read_u32_field(state, 0x1d8),
        id_1dc: read_u16_field(state, 0x1dc),
        id_1e4: read_u16_field(state, 0x1e4),
        ptr_f0,
        ptr_f8: read_usize_field(state, 0xf8).filter(|address| *address != 0),
        ptr_430,
        ptr_430_7bd8: ptr_430.and_then(|address| read_usize_field(address, 0x7bd8)),
        object_14d: ptr_f0.and_then(|address| read_u8_field(address, 0x14d)),
        object_1dc: ptr_f0.and_then(|address| read_u16_field(address, 0x1dc)),
        object_232: ptr_f0.and_then(|address| read_u8_field(address, 0x232)),
        object_235: ptr_f0.and_then(|address| read_u8_field(address, 0x235)),
    })
}

fn format_launch_costume_state_trace(trace: LaunchCostumeStateTrace) -> String {
    format!(
        "state=0x{:x} index4={} flags20={} id1d0={} id1d4={} id1d8={} id1dc={} id1e4={} ptr_f0={} ptr_f8={} ptr430={} ptr430_7bd8={} obj14d={} obj1dc={} obj232={} obj235={}",
        trace.state,
        format_optional_u16_decimal(trace.index4),
        format_optional_hex_u32(trace.flags20),
        format_optional_u32(trace.id_1d0),
        format_optional_u32(trace.id_1d4),
        format_optional_u32(trace.id_1d8),
        format_optional_u16_decimal(trace.id_1dc),
        format_optional_u16_decimal(trace.id_1e4),
        format_optional_address(trace.ptr_f0),
        format_optional_address(trace.ptr_f8),
        format_optional_address(trace.ptr_430),
        format_optional_address(trace.ptr_430_7bd8),
        format_optional_u8(trace.object_14d),
        format_optional_u16_decimal(trace.object_1dc),
        format_optional_u8(trace.object_232),
        format_optional_u8(trace.object_235)
    )
}

fn law_launch_state_private_model_alias_target(trace: LaunchCostumeStateTrace) -> Option<u32> {
    if trace.id_1d0 != Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID))
        || trace.id_1e4 != Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u16)
    {
        return None;
    }

    Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID))
}

fn log_launch_costume_state_patch(detail: String) {
    let index = LAUNCH_COSTUME_STATE_PATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_LAUNCH_COSTUME_STATE_PATCH_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Launch costume private-model alias diagnostic {detail} frames={}",
        format_stack_frames(&frames, 6)
    ));
}

fn patch_launch_costume_state_private_model_alias(state: usize) -> Option<LaunchCostumeStateTrace> {
    let trace = read_launch_costume_state_trace(state);
    if !LAW_EXTRA_SLOT_LAUNCH_PRIVATE_MODEL_ALIAS_ENABLED || state == 0 {
        return trace;
    }
    let Some(trace) = trace else {
        return None;
    };
    let Some(target) = law_launch_state_private_model_alias_target(trace) else {
        return Some(trace);
    };
    let Some(address) = state.checked_add(LAUNCH_COSTUME_STATE_MODEL_RESOURCE_OFFSET) else {
        log_launch_costume_state_patch(format!(
            "state=0x{state:x} reason=address_overflow before=[{}]",
            format_launch_costume_state_trace(trace)
        ));
        return Some(trace);
    };

    let before = read_u32_field(state, LAUNCH_COSTUME_STATE_MODEL_RESOURCE_OFFSET);
    let patch = target.to_le_bytes();
    let written = win::write_process_memory(address, &patch);
    let after = read_u32_field(state, LAUNCH_COSTUME_STATE_MODEL_RESOURCE_OFFSET);
    let patched = after == Some(target);
    log_launch_costume_state_patch(format!(
        "state=0x{state:x} address=0x{address:x} target={target} before={} after={} patched={patched} write_ok={} written={} trace_before=[{}]",
        format_optional_u32(before),
        format_optional_u32(after),
        written == Some(patch.len()),
        written.unwrap_or(0),
        format_launch_costume_state_trace(trace)
    ));

    read_launch_costume_state_trace(state).or(Some(trace))
}

fn is_interesting_launch_costume_state_trace(trace: LaunchCostumeStateTrace) -> bool {
    [trace.id_1d0, trace.id_1d4, trace.id_1d8]
        .into_iter()
        .flatten()
        .any(is_law_menu_value)
        || [trace.id_1dc, trace.id_1e4]
            .into_iter()
            .flatten()
            .any(|value| is_law_menu_value(u32::from(value)))
}

fn log_launch_costume_state_call(
    phase: &str,
    state: usize,
    trace: Option<LaunchCostumeStateTrace>,
    result: Option<usize>,
) {
    let interesting = trace.is_some_and(is_interesting_launch_costume_state_trace);
    let logged = LAUNCH_COSTUME_STATE_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= LAUNCH_COSTUME_STATE_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = LAUNCH_COSTUME_STATE_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_LAUNCH_COSTUME_STATE_TRACE_LOGS {
        return;
    }

    let trace_text = trace
        .map(format_launch_costume_state_trace)
        .unwrap_or_else(|| "none".to_string());
    let result_text = result
        .map(|value| format!("0x{value:x}"))
        .unwrap_or_else(|| "pending".to_string());
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Launch costume state {phase} state=0x{state:x} result={result_text} trace=[{trace_text}] frames={}",
        format_stack_frames(&frames, 6)
    ));
}

fn read_costume_scene_post_available_detail(state: usize) -> String {
    let object = read_usize_field(state, COSTUME_SCENE_SELECTED_OBJECT_OFFSET);
    let object_vtable = object.and_then(|address| read_usize_absolute(address));
    let object_call20 = object_vtable.and_then(|address| read_usize_field(address, 0x20));
    let object_byte8 = object.and_then(|address| read_u8_field(address, 0x08));
    let object_flags30 = object.and_then(|address| read_u32_field(address, 0x30));
    let object_child58 = object.and_then(|address| read_usize_field(address, 0x58));
    let action_widget = read_game_global_pointer(GAME_GLOBAL_ACTION_WIDGET_RVA);
    let action_widget_flags = action_widget.and_then(|address| read_u32_field(address, 0x30));
    let costume_widget = read_game_global_pointer(GAME_GLOBAL_COSTUME_WIDGET_RVA);

    format!(
        "f0={} f8={} object100={} object108={} selected_object={} object_vtable={} object_call20={} object_byte8={} object_flags30={} object_child58={} action_widget={} action_flags30={} costume_widget={}",
        format_optional_address(read_usize_field(state, 0xf0)),
        format_optional_address(read_usize_field(state, 0xf8)),
        format_optional_address(read_usize_field(state, 0x100)),
        format_optional_address(read_usize_field(state, 0x108)),
        format_optional_address(object),
        format_optional_address(object_vtable),
        format_optional_address(object_call20),
        format_optional_u8(object_byte8),
        format_optional_hex_u32(object_flags30),
        format_optional_address(object_child58),
        format_optional_address(action_widget),
        format_optional_hex_u32(action_widget_flags),
        format_optional_address(costume_widget),
    )
}

fn read_costume_object_update_trace(state: usize) -> Option<CostumeObjectUpdateTrace> {
    if state == 0 {
        return None;
    }

    let count = read_u32_field(state, COSTUME_OBJECT_UPDATE_COUNT_OFFSET);
    let selected_index = read_u32_field(state, COSTUME_OBJECT_UPDATE_SELECTED_INDEX_OFFSET);
    let selected_layout = selected_index.and_then(|index| {
        read_costume_object_update_array_value(state, COSTUME_OBJECT_UPDATE_LAYOUTS_OFFSET, index)
    });
    let selected_slot = selected_index.and_then(|index| {
        read_costume_object_update_array_value(state, COSTUME_OBJECT_UPDATE_SLOTS_OFFSET, index)
    });
    let selected_kind = selected_index.and_then(|index| {
        read_costume_object_update_array_value(state, COSTUME_OBJECT_UPDATE_KINDS_OFFSET, index)
    });
    let selected_load_arg0 = selected_index.and_then(|index| {
        read_costume_object_update_array_value(state, COSTUME_OBJECT_UPDATE_LOAD_ARG0_OFFSET, index)
    });
    let selected_load_arg1 = selected_index.and_then(|index| {
        read_costume_object_update_array_value(state, COSTUME_OBJECT_UPDATE_LOAD_ARG1_OFFSET, index)
    });
    let selected_load_arg2 = selected_index.and_then(|index| {
        read_costume_object_update_array_value(state, COSTUME_OBJECT_UPDATE_LOAD_ARG2_OFFSET, index)
    });
    let selected_variant = selected_layout
        .zip(selected_slot)
        .and_then(|(layout, slot)| read_layout_slot_variant(layout, slot));
    let selected_model_resource = selected_variant.and_then(read_variant_model_resource);
    let selected_preview_mapping = selected_variant.and_then(read_variant_preview_mapping);
    let selected_preview_mapped_resource =
        selected_preview_mapping.map(preview_model_mapped_id_from_metadata_value);
    let object = read_usize_field(state, COSTUME_OBJECT_UPDATE_OBJECT_OFFSET)
        .filter(|address| *address != 0);
    let controller = read_usize_field(state, COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET)
        .filter(|address| *address != 0);

    Some(CostumeObjectUpdateTrace {
        count,
        selected_index,
        mode: read_u32_field(state, COSTUME_OBJECT_UPDATE_MODE_OFFSET),
        cached_layout: read_u32_field(state, COSTUME_OBJECT_UPDATE_CACHED_LAYOUT_OFFSET),
        refresh_flag: read_u8_field(state, COSTUME_OBJECT_UPDATE_FLAG_REFRESH_OFFSET),
        locked_flag: read_u8_field(state, COSTUME_OBJECT_UPDATE_FLAG_LOCKED_OFFSET),
        object,
        controller,
        object_child: object
            .and_then(|address| {
                read_usize_field(address, COSTUME_OBJECT_UPDATE_OBJECT_CHILD_OFFSET)
            })
            .filter(|address| *address != 0),
        object_pending: object.and_then(|address| {
            read_u8_field(address, COSTUME_OBJECT_UPDATE_OBJECT_PENDING_OFFSET)
        }),
        controller_model_loader: controller
            .and_then(|address| {
                read_usize_field(
                    address,
                    COSTUME_OBJECT_UPDATE_CONTROLLER_MODEL_LOADER_OFFSET,
                )
            })
            .filter(|address| *address != 0),
        selected_layout,
        selected_slot,
        selected_kind,
        selected_load_arg0,
        selected_load_arg1,
        selected_load_arg2,
        selected_variant,
        selected_model_resource,
        selected_preview_mapping,
        selected_preview_mapped_resource,
    })
}

fn read_costume_object_update_array_value(state: usize, offset: usize, index: u32) -> Option<u32> {
    if index >= COSTUME_LAYOUT_VARIANT_COUNT {
        return None;
    }
    read_u32_field(state, offset + index as usize * size_of::<u32>())
}

fn read_layout_slot_variant(layout_id: u32, slot_index: u32) -> Option<u16> {
    if layout_id >= COSTUME_LAYOUT_ROW_COUNT || slot_index >= COSTUME_LAYOUT_VARIANT_COUNT {
        return None;
    }
    let address = static_layouts_address()?
        + layout_id as usize * COSTUME_LAYOUT_ROW_STRIDE
        + COSTUME_LAYOUT_VARIANTS_OFFSET
        + slot_index as usize * size_of::<u16>();
    read_u16_absolute(address)
}

fn read_law_layout_active_count() -> Option<u8> {
    let address = static_layouts_address()?
        + LAW_MASTER_LAYOUT_ID as usize * COSTUME_LAYOUT_ROW_STRIDE
        + COSTUME_LAYOUT_AVAILABILITY_FLAG_OFFSET
        - 2;
    read_u8_absolute(address)
}

fn read_layout_variants(layout_id: u32) -> Vec<u16> {
    if layout_id >= COSTUME_LAYOUT_ROW_COUNT {
        return Vec::new();
    }

    (0..COSTUME_LAYOUT_VARIANT_COUNT)
        .filter_map(|slot| read_layout_slot_variant(layout_id, slot))
        .collect()
}

fn read_variant_model_resource(variant_id: u16) -> Option<u16> {
    let address = costume_variant_metadata_record_address(static_layouts_address()?, variant_id)?
        + COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET;
    read_u16_absolute(address)
}

fn read_variant_preview_mapping(variant_id: u16) -> Option<u16> {
    let address = costume_variant_metadata_record_address(static_layouts_address()?, variant_id)?
        + COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET;
    read_u16_absolute(address)
}

fn read_variant_flags(variant_id: u16) -> Option<u8> {
    let address = costume_variant_metadata_record_address(static_layouts_address()?, variant_id)?
        + COSTUME_VARIANT_METADATA_FLAGS_OFFSET;
    read_u8_absolute(address)
}

fn read_variant_metadata_raw(variant_id: u16) -> Option<[u8; COSTUME_VARIANT_METADATA_COPY_SIZE]> {
    let address = costume_variant_metadata_record_address(static_layouts_address()?, variant_id)?;
    let mut bytes = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
    read_exact_process_memory(address, &mut bytes).then_some(bytes)
}

fn log_costume_object_update(
    state: usize,
    before: Option<CostumeObjectUpdateTrace>,
    after: Option<CostumeObjectUpdateTrace>,
) {
    let interesting = before.is_some_and(is_interesting_costume_object_update_trace)
        || after.is_some_and(is_interesting_costume_object_update_trace)
        || law_custom_slot_active();
    let logged = COSTUME_OBJECT_UPDATE_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= COSTUME_OBJECT_UPDATE_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = COSTUME_OBJECT_UPDATE_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_UPDATE_TRACE_LOGS {
        return;
    }

    let before_text = before
        .map(format_costume_object_update_trace)
        .unwrap_or_else(|| "none".to_string());
    let after_text = after
        .map(format_costume_object_update_trace)
        .unwrap_or_else(|| "none".to_string());
    let entries = format_costume_object_update_entries(state, after.or(before));
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume object update state=0x{state:x} before=[{before_text}] after=[{after_text}] entries=[{entries}] frames={}",
        format_stack_frames(&frames, 8)
    ));
    log_law_slot5_builder_after_metadata_patch(state, after.or(before), &frames);
    log_law_slot_count_source_diff(state, after.or(before));
}

fn log_law_slot5_builder_after_metadata_patch(
    state: usize,
    trace: Option<CostumeObjectUpdateTrace>,
    frames: &[usize],
) {
    let Some(trace) = trace else {
        return;
    };
    if trace.selected_layout != Some(u32::from(LAW_MASTER_LAYOUT_ID))
        && trace.cached_layout != Some(u32::from(LAW_MASTER_LAYOUT_ID))
    {
        return;
    }
    if read_law_layout_active_count() != Some(5)
        || read_layout_slot_variant(
            u32::from(LAW_MASTER_LAYOUT_ID),
            LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32,
        ) != Some(LAW_CUSTOM_LAYOUT_ID)
    {
        return;
    }
    if let Some(static_layouts) = static_layouts_address() {
        maybe_patch_law_linkdata_only_ram_variant699_metadata_at_static_layouts(
            static_layouts,
            "costume_object_update",
        );
    }
    maybe_patch_law_linkdata_only_runtime_unlock_slot("costume_object_update");
    if LAW_SLOT_BUILDER_AFTER_METADATA_PATCH_LOGS.fetch_add(1, Ordering::Relaxed) >= 3 {
        return;
    }

    log::write_line(format!(
        "law-slot5-builder-after-metadata-patch count={} entries=[{}] variant_metadata=[{}] selected=[{}] frames={}",
        format_optional_u32(trace.count),
        format_costume_object_update_entries_fixed(state, 6),
        format_law_variant_metadata_summary(),
        format_costume_object_update_trace(trace),
        format_stack_frames(frames, 8)
    ));
}

fn log_law_slot_count_source_diff(state: usize, trace: Option<CostumeObjectUpdateTrace>) {
    let Some(trace) = trace else {
        return;
    };
    if trace.selected_layout != Some(u32::from(LAW_MASTER_LAYOUT_ID))
        && trace.cached_layout != Some(u32::from(LAW_MASTER_LAYOUT_ID))
    {
        return;
    }
    if LAW_SLOT_COUNT_DIFF_LOGS.fetch_add(1, Ordering::Relaxed) >= 2 {
        return;
    }

    let layout_active_count = read_law_layout_active_count();
    let layout_custom_slot =
        read_layout_slot_variant(u32::from(LAW_MASTER_LAYOUT_ID), LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32);
    let row_slot_value = read_costume_row_slot_value(
        LAW_MENU_ROW_ID as u32,
        LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32,
    );
    log::write_line(format!(
        "law-slot-count-source-diff count_runtime={} selected_slot={} selected_variant={} selected_model={} layout26_active_count={} layout26_slot4={} row70_slot4={} source=ram_vs_costume_object",
        format_optional_u32(trace.count),
        format_optional_u32(trace.selected_slot),
        format_optional_u16_decimal(trace.selected_variant),
        format_optional_u16_decimal(trace.selected_model_resource),
        format_optional_u8(layout_active_count),
        format_optional_u16_decimal(layout_custom_slot),
        format_optional_u16_decimal(row_slot_value)
    ));
    log_law_slot_builder_filter(state, trace);
}

fn log_law_slot_builder_filter(state: usize, trace: CostumeObjectUpdateTrace) {
    if trace.count != Some(4) {
        return;
    }

    let layout_active_count = read_law_layout_active_count();
    let layout_custom_slot =
        read_layout_slot_variant(u32::from(LAW_MASTER_LAYOUT_ID), LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32);
    if layout_active_count != Some(5) || layout_custom_slot != Some(LAW_CUSTOM_LAYOUT_ID) {
        return;
    }

    if LAW_SLOT_BUILDER_FILTER_LOGS.fetch_add(1, Ordering::Relaxed) >= 2 {
        return;
    }

    let row_slot_value = read_costume_row_slot_value(
        LAW_MENU_ROW_ID as u32,
        LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32,
    );
    let layout_variants = read_layout_variants(u32::from(LAW_MASTER_LAYOUT_ID));
    let frames = capture_stack_trace();
    log::write_line(format!(
        "law-slot-builder-filter count_runtime=4 layout26_active_count=5 layout26_slot4=699 row70_slot4={} layout26_variants={} raw_entries=[{}] variant_metadata=[{}] metadata699_source=[{}] selected=[{}] reason=layout_has_slot_but_runtime_builder_ignored_slot4 frames={}",
        format_optional_u16_decimal(row_slot_value),
        if layout_variants.is_empty() {
            "read_failed".to_string()
        } else {
            format_u16_list(&layout_variants)
        },
        format_costume_object_update_entries_fixed(state, 6),
        format_law_variant_metadata_summary(),
        format_law_variant699_metadata_source_diff(),
        format_costume_object_update_trace(trace),
        format_stack_frames(&frames, 8)
    ));
}

fn log_costume_object_pending_change(
    phase: &'static str,
    before: Option<CostumeObjectUpdateTrace>,
    after: Option<CostumeObjectUpdateTrace>,
) {
    let trace = after.or(before);
    let before_pending = before.and_then(|trace| trace.object_pending);
    let after_pending = after.and_then(|trace| trace.object_pending);
    log_costume_object_pending_value_change(phase, trace, before_pending, after_pending);
}

fn log_costume_object_pending_value_change(
    phase: &'static str,
    trace: Option<CostumeObjectUpdateTrace>,
    before_pending: Option<u8>,
    after_pending: Option<u8>,
) {
    let Some(trace) = trace else {
        return;
    };
    let Some(label) = law_ready_timeline_costume_trace_label(trace) else {
        return;
    };
    let changed = before_pending != after_pending;
    let slot5 = label == "slot5-custom";
    if !changed && !slot5 {
        return;
    }

    let index = COSTUME_OBJECT_PENDING_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_PENDING_TRACE_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log_law_ready_timeline(
        "pending-change",
        format!(
            "slot_label={label} phase={phase} before={} after={} changed={} context=[{}] frames={}",
            format_optional_u8(before_pending),
            format_optional_u8(after_pending),
            changed,
            format_costume_object_update_trace(trace),
            format_stack_frames(&frames, 6)
        ),
    );
    log::write_line(format!(
        "Costume object pending-change phase={phase} slot_label={label} before={} after={} changed={} context=[{}] frames={}",
        format_optional_u8(before_pending),
        format_optional_u8(after_pending),
        changed,
        format_costume_object_update_trace(trace),
        format_stack_frames(&frames, 8)
    ));

    if label == "slot5-custom" && before_pending == Some(1) && after_pending == Some(0) {
        log_slot5_pending_cleared_after_ready(phase, trace, &frames);
    }
}

fn log_slot5_pending_cleared_after_ready(
    phase: &'static str,
    trace: CostumeObjectUpdateTrace,
    frames: &[usize],
) {
    let object = LAST_SLOT5_READY_CHECKPOINT_OBJECT.load(Ordering::Relaxed);
    let loader = LAST_SLOT5_READY_CHECKPOINT_LOADER.load(Ordering::Relaxed);
    let seq = LAST_SLOT5_READY_CHECKPOINT_SEQ.load(Ordering::Relaxed);
    if object == 0 || seq == 0 {
        return;
    }

    let index = SLOT5_PENDING_CLEARED_AFTER_READY_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_PENDING_CLEARED_AFTER_READY_LOGS {
        return;
    }

    let expected_model = u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID);
    let current_load_model = read_u32_field(object, 0x18);
    let current_compare_model = read_u32_field(object, 0x378);
    let loader_still_valid =
        current_load_model == Some(expected_model) && current_compare_model == Some(expected_model);
    let checkpoint_pending = LAST_SLOT5_READY_CHECKPOINT_PENDING.load(Ordering::Relaxed);

    log::write_line(format!(
        "slot5-pending-cleared-after-ready phase={phase} object_pending=0x00 loader_still_valid={loader_still_valid} checkpoint_seq={seq} checkpoint_pending={} loader={} object={} checkpoint=[state28={} phase2c={} flags20={}] current=[{}] alias=[{}] context=[{}] frames={}",
        checkpoint_pending_text(checkpoint_pending),
        format_optional_address((loader != 0).then_some(loader)),
        format_optional_address((object != 0).then_some(object)),
        checkpoint_pending_text(LAST_SLOT5_READY_CHECKPOINT_STATE28.load(Ordering::Relaxed)),
        checkpoint_pending_text(LAST_SLOT5_READY_CHECKPOINT_PHASE2C.load(Ordering::Relaxed)),
        checkpoint_pending_text(LAST_SLOT5_READY_CHECKPOINT_FLAGS20.load(Ordering::Relaxed)),
        format_model_ready_object_detail(object),
        format_model_manager_alias_state(
            "slot5",
            expected_model,
            Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)),
        ),
        format_costume_object_update_trace(trace),
        format_stack_frames(frames, 8),
    ));
}

fn is_interesting_costume_object_apply_ready(object: usize, slot: u32, ready: u32) -> bool {
    object != 0
        && (law_custom_slot_active()
            || slot == 0
            || ready == 0
            || read_u32_field(object, COSTUME_SCENE_SELECTED_OBJECT_LAYOUT_OFFSET)
                .is_some_and(is_law_menu_value)
            || read_u32_field(object, COSTUME_SCENE_SELECTED_OBJECT_VARIANT_OFFSET)
                .is_some_and(is_law_menu_value))
}

fn format_costume_object_apply_ready_object(object: usize, slot: u32) -> String {
    if object == 0 {
        return "object=0x0".to_string();
    }

    let slot_index = slot as usize;
    let status_2ac = slot_index
        .checked_add(0x2ac)
        .and_then(|offset| read_u8_field(object, offset));
    let selector_290 = slot_index
        .checked_mul(size_of::<u32>())
        .and_then(|offset| 0x290usize.checked_add(offset))
        .and_then(|offset| read_u32_field(object, offset));
    let child58 = read_usize_field(object, COSTUME_OBJECT_UPDATE_OBJECT_CHILD_OFFSET)
        .filter(|address| *address != 0);

    format!(
        "object=0x{object:x} child58={} status2ac={} selector290={} pending2b4={} layout440={} variant448={} child28={} child38={} child48={} child54={} childb4={}",
        format_optional_address(child58),
        format_optional_u8(status_2ac),
        format_optional_u32(selector_290),
        format_optional_u8(read_u8_field(
            object,
            COSTUME_OBJECT_UPDATE_OBJECT_PENDING_OFFSET
        )),
        format_optional_u32(read_u32_field(
            object,
            COSTUME_SCENE_SELECTED_OBJECT_LAYOUT_OFFSET
        )),
        format_optional_u32(read_u32_field(
            object,
            COSTUME_SCENE_SELECTED_OBJECT_VARIANT_OFFSET
        )),
        format_optional_address(
            child58.and_then(|address| read_usize_field(address, 0x28))
        ),
        format_optional_address(
            child58.and_then(|address| read_usize_field(address, 0x38))
        ),
        format_optional_address(
            child58.and_then(|address| read_usize_field(address, 0x48))
        ),
        format_optional_u32(child58.and_then(|address| read_u32_field(address, 0x54))),
        format_optional_u32(child58.and_then(|address| read_u32_field(address, 0xb4))),
    )
}

fn format_costume_object_slot_fields(object: usize, slot_index: Option<u32>) -> String {
    let Some(slot_index) = slot_index else {
        return "slot=none status2ac=none selector290=none".to_string();
    };
    let slot_index = slot_index as usize;
    let status_2ac = slot_index
        .checked_add(0x2ac)
        .and_then(|offset| read_u8_field(object, offset));
    let selector_290 = slot_index
        .checked_mul(size_of::<u32>())
        .and_then(|offset| 0x290usize.checked_add(offset))
        .and_then(|offset| read_u32_field(object, offset));
    format!(
        "slot={slot_index} status2ac={} selector290={}",
        format_optional_u8(status_2ac),
        format_optional_u32(selector_290),
    )
}

fn format_costume_object_refresh_preview_object(
    object: usize,
    trace: Option<CostumeObjectUpdateTrace>,
) -> String {
    if object == 0 {
        return "object=0x0".to_string();
    }
    let selected_index = trace.and_then(|trace| trace.selected_index);
    let child58 = read_usize_field(object, COSTUME_OBJECT_UPDATE_OBJECT_CHILD_OFFSET)
        .filter(|address| *address != 0);
    format!(
        "object=0x{object:x} child58={} pending2b4={} layout440={} variant448={} slot0=[{}] selected_slot=[{}] child28={} child38={} child48={} child54={} childb4={}",
        format_optional_address(child58),
        format_optional_u8(read_u8_field(
            object,
            COSTUME_OBJECT_UPDATE_OBJECT_PENDING_OFFSET
        )),
        format_optional_u32(read_u32_field(
            object,
            COSTUME_SCENE_SELECTED_OBJECT_LAYOUT_OFFSET
        )),
        format_optional_u32(read_u32_field(
            object,
            COSTUME_SCENE_SELECTED_OBJECT_VARIANT_OFFSET
        )),
        format_costume_object_slot_fields(object, Some(0)),
        format_costume_object_slot_fields(object, selected_index),
        format_optional_address(
            child58.and_then(|address| read_usize_field(address, 0x28))
        ),
        format_optional_address(
            child58.and_then(|address| read_usize_field(address, 0x38))
        ),
        format_optional_address(
            child58.and_then(|address| read_usize_field(address, 0x48))
        ),
        format_optional_u32(child58.and_then(|address| read_u32_field(address, 0x54))),
        format_optional_u32(child58.and_then(|address| read_u32_field(address, 0xb4))),
    )
}

fn read_costume_object_refresh_preview_trace(state: usize, preview_arg: u32) -> Option<String> {
    let trace = read_costume_object_update_trace(state);
    let object = trace
        .and_then(|trace| trace.object)
        .or_else(|| read_usize_field(state, COSTUME_OBJECT_UPDATE_OBJECT_OFFSET));
    let label = trace.and_then(law_ready_timeline_costume_trace_label);
    if label.is_none()
        && !law_custom_slot_active()
        && !object.is_some_and(|object| {
            read_u32_field(object, COSTUME_SCENE_SELECTED_OBJECT_LAYOUT_OFFSET)
                .is_some_and(is_law_menu_value)
                || read_u32_field(object, COSTUME_SCENE_SELECTED_OBJECT_VARIANT_OFFSET)
                    .is_some_and(is_law_menu_value)
        })
    {
        return None;
    }
    let trace_text = trace
        .map(format_costume_object_update_trace)
        .unwrap_or_else(|| "none".to_string());
    let object_text = object
        .map(|object| format_costume_object_refresh_preview_object(object, trace))
        .unwrap_or_else(|| "object=none".to_string());
    Some(format!(
        "slot_label={} preview_arg={preview_arg} state=[{trace_text}] object=[{object_text}] busy=[{}]",
        label.unwrap_or("other"),
        format_last_costume_object_busy_check(),
    ))
}

fn format_costume_object_refresh_preview_identity(
    trace: Option<CostumeObjectUpdateTrace>,
    preview_arg: u32,
) -> String {
    let Some(trace) = trace else {
        return format!(
            "selected_model_resource=none selected_preview_mapping=none selected_preview_mapped=none refresh_preview_arg={preview_arg}"
        );
    };
    format!(
        "selected_model_resource={} selected_preview_mapping={} selected_preview_mapped={} refresh_preview_arg={preview_arg}",
        format_optional_u16_decimal(trace.selected_model_resource),
        format_optional_u16_decimal(trace.selected_preview_mapping),
        format_optional_u32(trace.selected_preview_mapped_resource),
    )
}

fn read_refresh_preview_table_probe(preview_arg: u32) -> RefreshPreviewTableProbe {
    let module = win::main_module() as usize;
    if module == 0 {
        return RefreshPreviewTableProbe {
            preview_arg,
            status: "module_missing",
            global: None,
            vector: None,
            start: None,
            end: None,
            count: None,
            entry: None,
            width: None,
            height: None,
            raw: None,
        };
    }

    let global_address = module + COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_GLOBAL_RVA;
    let Some(global) = read_usize_absolute(global_address).filter(|address| *address != 0) else {
        return RefreshPreviewTableProbe {
            preview_arg,
            status: "global_missing",
            global: None,
            vector: None,
            start: Some(global_address),
            end: None,
            count: None,
            entry: None,
            width: None,
            height: None,
            raw: None,
        };
    };
    let Some(vector) = read_usize_field(global, 0x18).filter(|address| *address != 0) else {
        return RefreshPreviewTableProbe {
            preview_arg,
            status: "vector_missing",
            global: Some(global),
            vector: None,
            start: None,
            end: None,
            count: None,
            entry: None,
            width: None,
            height: None,
            raw: None,
        };
    };
    let Some(start) = read_usize_field(vector, 0x08).filter(|address| *address != 0) else {
        return RefreshPreviewTableProbe {
            preview_arg,
            status: "start_missing",
            global: Some(global),
            vector: Some(vector),
            start: None,
            end: None,
            count: None,
            entry: None,
            width: None,
            height: None,
            raw: None,
        };
    };
    let Some(end) = read_usize_field(vector, 0x10).filter(|address| *address >= start) else {
        return RefreshPreviewTableProbe {
            preview_arg,
            status: "end_invalid",
            global: Some(global),
            vector: Some(vector),
            start: Some(start),
            end: None,
            count: None,
            entry: None,
            width: None,
            height: None,
            raw: None,
        };
    };
    let count = (end - start) / COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_STRIDE;

    if (COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_SPECIAL_START
        ..=COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_SPECIAL_END)
        .contains(&preview_arg)
    {
        return RefreshPreviewTableProbe {
            preview_arg,
            status: "special_0x351_0x35d",
            global: Some(global),
            vector: Some(vector),
            start: Some(start),
            end: Some(end),
            count: Some(count),
            entry: None,
            width: Some(0),
            height: Some(0),
            raw: None,
        };
    }

    let arg_index = preview_arg as usize;
    if arg_index >= count {
        return RefreshPreviewTableProbe {
            preview_arg,
            status: "out_of_range",
            global: Some(global),
            vector: Some(vector),
            start: Some(start),
            end: Some(end),
            count: Some(count),
            entry: None,
            width: None,
            height: None,
            raw: None,
        };
    }

    let entry = start + arg_index * COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_STRIDE;
    let mut raw = [0u8; COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_STRIDE];
    if !read_exact_process_memory(entry, &mut raw) {
        return RefreshPreviewTableProbe {
            preview_arg,
            status: "read_failed",
            global: Some(global),
            vector: Some(vector),
            start: Some(start),
            end: Some(end),
            count: Some(count),
            entry: Some(entry),
            width: None,
            height: None,
            raw: None,
        };
    }

    let width = if preview_arg < COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_DIMENSION_LIMIT {
        Some(u32::from_le_bytes([
            raw[COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_WIDTH_OFFSET],
            raw[COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_WIDTH_OFFSET + 1],
            raw[COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_WIDTH_OFFSET + 2],
            raw[COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_WIDTH_OFFSET + 3],
        ]))
    } else {
        None
    };
    let height = if preview_arg < COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_DIMENSION_LIMIT {
        Some(u32::from_le_bytes([
            raw[COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_HEIGHT_OFFSET],
            raw[COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_HEIGHT_OFFSET + 1],
            raw[COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_HEIGHT_OFFSET + 2],
            raw[COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_HEIGHT_OFFSET + 3],
        ]))
    } else {
        None
    };
    let status =
        if raw.iter().all(|byte| *byte == 0) || matches!((width, height), (Some(0), Some(0))) {
            "empty_or_zero"
        } else {
            "in_range"
        };

    RefreshPreviewTableProbe {
        preview_arg,
        status,
        global: Some(global),
        vector: Some(vector),
        start: Some(start),
        end: Some(end),
        count: Some(count),
        entry: Some(entry),
        width,
        height,
        raw: Some(raw),
    }
}

fn format_refresh_preview_table_probe(preview_arg: u32) -> String {
    format_refresh_preview_table_probe_detail(read_refresh_preview_table_probe(preview_arg))
}

fn format_refresh_preview_table_probe_detail(probe: RefreshPreviewTableProbe) -> String {
    format!(
        "arg={} status={} global={} vector={} start={} end={} count={} entry={} width={} height={} raw={}",
        probe.preview_arg,
        probe.status,
        format_optional_address(probe.global),
        format_optional_address(probe.vector),
        format_optional_address(probe.start),
        format_optional_address(probe.end),
        probe
            .count
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        format_optional_address(probe.entry),
        format_optional_u32(probe.width),
        format_optional_u32(probe.height),
        probe
            .raw
            .map(|raw| format_bytes(&raw))
            .unwrap_or_else(|| "none".to_string())
    )
}

fn format_refresh_preview_table_probe_short(probe: RefreshPreviewTableProbe) -> String {
    let raw_head = probe
        .raw
        .map(|raw| format_bytes(&raw[..16]))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "id={} status={} entry={} width={} height={} raw16={}",
        probe.preview_arg,
        probe.status,
        format_optional_address(probe.entry),
        format_optional_u32(probe.width),
        format_optional_u32(probe.height),
        raw_head
    )
}

fn format_refresh_preview_table_compare(
    preview_arg: u32,
    trace: Option<CostumeObjectUpdateTrace>,
) -> String {
    let model_resource = trace
        .and_then(|trace| trace.selected_model_resource)
        .map(u32::from);
    let preview_mapping = trace
        .and_then(|trace| trace.selected_preview_mapping)
        .map(u32::from);
    let preview_mapped = trace.and_then(|trace| trace.selected_preview_mapped_resource);
    let ids = [
        Some(preview_arg),
        model_resource,
        preview_mapping,
        preview_mapped,
        Some(LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID),
        Some(LAW_EXTRA_SLOT_PREVIEW_TABLE_MAPPING_SOURCE_ID),
        Some(LAW_EXTRA_SLOT_PREVIEW_TABLE_ONI_FALLBACK_SOURCE_ID),
        Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID),
    ];
    let mut seen = Vec::new();
    let probes = ids
        .into_iter()
        .flatten()
        .filter(|id| {
            if seen.contains(id) {
                false
            } else {
                seen.push(*id);
                true
            }
        })
        .map(|id| format_refresh_preview_table_probe_short(read_refresh_preview_table_probe(id)))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "refresh_arg={} model={} mapping={} mapped={} probes=[{}]",
        preview_arg,
        format_optional_u32(model_resource),
        format_optional_u32(preview_mapping),
        format_optional_u32(preview_mapped),
        probes
    )
}

fn refresh_preview_table_probe_is_empty(probe: RefreshPreviewTableProbe) -> bool {
    probe.status == "empty_or_zero"
}

fn refresh_preview_table_probe_is_valid_source(probe: RefreshPreviewTableProbe) -> bool {
    probe.status == "in_range" && probe.width.unwrap_or(0) > 0 && probe.height.unwrap_or(0) > 0
}

fn preview_table_clone_guard_interesting(
    trace: Option<CostumeObjectUpdateTrace>,
    preview_arg: u32,
) -> bool {
    preview_arg == LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID
        || trace
            .and_then(|trace| trace.selected_slot)
            .is_some_and(|slot| slot == LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32)
}

fn log_preview_table_clone_guard(
    trace: Option<CostumeObjectUpdateTrace>,
    preview_arg: u32,
    arg_ok: bool,
    layout_ok: bool,
    slot_ok: bool,
    model_ok: bool,
    mapping_ok: bool,
) {
    if !preview_table_clone_guard_interesting(trace, preview_arg) {
        return;
    }
    let index =
        COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_GUARD_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_GUARD_LOGS {
        return;
    }

    let target = read_refresh_preview_table_probe(LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID);
    let source294 =
        read_refresh_preview_table_probe(LAW_EXTRA_SLOT_PREVIEW_TABLE_MAPPING_SOURCE_ID);
    let source308 =
        read_refresh_preview_table_probe(LAW_EXTRA_SLOT_PREVIEW_TABLE_ONI_FALLBACK_SOURCE_ID);
    log::write_line(format!(
        "preview-table-clone-guard preview_arg={} selected_layout={} selected_slot={} selected_variant={} selected_model_resource={} selected_preview_mapping={} current_probe_variant={} arg_ok={} layout_ok={} slot_ok={} model_ok={} mapping_ok={} target_empty={} source294_valid={} source308_valid={} target=[{}] source294=[{}] source308=[{}]",
        preview_arg,
        format_optional_u32(trace.and_then(|trace| trace.selected_layout)),
        format_optional_u32(trace.and_then(|trace| trace.selected_slot)),
        format_optional_u16_decimal(trace.and_then(|trace| trace.selected_variant)),
        format_optional_u16_decimal(trace.and_then(|trace| trace.selected_model_resource)),
        format_optional_u16_decimal(trace.and_then(|trace| trace.selected_preview_mapping)),
        current_law_extra_slot_probe_variant_id()
            .map(|variant| variant.to_string())
            .unwrap_or_else(|| "none".to_string()),
        arg_ok,
        layout_ok,
        slot_ok,
        model_ok,
        mapping_ok,
        refresh_preview_table_probe_is_empty(target),
        refresh_preview_table_probe_is_valid_source(source294),
        refresh_preview_table_probe_is_valid_source(source308),
        format_refresh_preview_table_probe_short(target),
        format_refresh_preview_table_probe_short(source294),
        format_refresh_preview_table_probe_short(source308)
    ));
}

fn should_clone_law_preview_table_292(
    trace: Option<CostumeObjectUpdateTrace>,
    preview_arg: u32,
) -> bool {
    if !LAW_EXTRA_SLOT_PREVIEW_TABLE_CLONE_292_ENABLED {
        return false;
    }
    if preview_arg != LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID {
        return false;
    }
    let Some(trace) = trace else {
        log_preview_table_clone_guard(None, preview_arg, true, false, false, false, false);
        return false;
    };
    let layout_ok = trace.selected_layout == Some(u32::from(LAW_MASTER_LAYOUT_ID));
    let slot_ok = trace.selected_slot == Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32);
    let model_ok =
        trace.selected_model_resource == Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID);
    let mapping_ok = trace.selected_preview_mapping
        == Some(LAW_EXTRA_SLOT_PREVIEW_TABLE_MAPPING_SOURCE_ID as u16);
    let result = layout_ok && slot_ok && model_ok && mapping_ok;
    if !result {
        log_preview_table_clone_guard(
            Some(trace),
            preview_arg,
            true,
            layout_ok,
            slot_ok,
            model_ok,
            mapping_ok,
        );
    }
    result
}

fn maybe_clone_law_preview_table_292(
    trace: Option<CostumeObjectUpdateTrace>,
    preview_arg: u32,
) -> Option<String> {
    if !should_clone_law_preview_table_292(trace, preview_arg) {
        return None;
    }

    let target = read_refresh_preview_table_probe(LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID);
    if !refresh_preview_table_probe_is_empty(target) {
        return Some(format!(
            "skipped target_status={} target_width={} target_height={}",
            target.status,
            format_optional_u32(target.width),
            format_optional_u32(target.height)
        ));
    }

    let mapping_source =
        read_refresh_preview_table_probe(LAW_EXTRA_SLOT_PREVIEW_TABLE_MAPPING_SOURCE_ID);
    let fallback_source =
        read_refresh_preview_table_probe(LAW_EXTRA_SLOT_PREVIEW_TABLE_ONI_FALLBACK_SOURCE_ID);
    let source = if refresh_preview_table_probe_is_valid_source(mapping_source) {
        mapping_source
    } else if refresh_preview_table_probe_is_valid_source(fallback_source) {
        fallback_source
    } else {
        return Some(format!(
            "failed no_valid_source source294=[{}] source308=[{}] target=[{}]",
            format_refresh_preview_table_probe_short(mapping_source),
            format_refresh_preview_table_probe_short(fallback_source),
            format_refresh_preview_table_probe_short(target)
        ));
    };

    let (Some(target_entry), Some(source_raw)) = (target.entry, source.raw) else {
        return Some(format!(
            "failed missing_address_or_raw source=[{}] target=[{}]",
            format_refresh_preview_table_probe_short(source),
            format_refresh_preview_table_probe_short(target)
        ));
    };
    let write_result = win::write_process_memory(target_entry, &source_raw);
    let after = read_refresh_preview_table_probe(LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID);
    Some(format!(
        "source={} target={} before_width={} before_height={} after_width={} after_height={} write={} before=[{}] source_probe=[{}] after=[{}]",
        source.preview_arg,
        target.preview_arg,
        format_optional_u32(target.width),
        format_optional_u32(target.height),
        format_optional_u32(after.width),
        format_optional_u32(after.height),
        write_result
            .map(|written| written.to_string())
            .unwrap_or_else(|| "failed".to_string()),
        format_refresh_preview_table_probe_short(target),
        format_refresh_preview_table_probe_short(source),
        format_refresh_preview_table_probe_short(after)
    ))
}

fn try_preview_table_clone_292(trace: Option<CostumeObjectUpdateTrace>, preview_arg: u32) {
    if let Some(clone_detail) = maybe_clone_law_preview_table_292(trace, preview_arg) {
        let (counter, max_logs) = if clone_detail.starts_with("skipped ") {
            (
                &COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_SKIPPED_LOGS,
                MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_SKIPPED_LOGS,
            )
        } else {
            (
                &COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_LOGS,
                MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_CLONE_LOGS,
            )
        };
        let index = counter.fetch_add(1, Ordering::Relaxed);
        if index < max_logs {
            log::write_line(format!("preview-table-clone {clone_detail}"));
        }
    }
}

fn log_preview_table_before_original(preview_arg: u32) {
    if preview_arg != LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID {
        return;
    }
    let index =
        COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_BEFORE_ORIGINAL_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_BEFORE_ORIGINAL_LOGS {
        return;
    }
    log::write_line(format!(
        "preview-table-before-original {}",
        format_refresh_preview_table_probe(preview_arg)
    ));
}

fn log_preview_table_after_original(preview_arg: u32) {
    if preview_arg != LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID {
        return;
    }
    let index =
        COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_AFTER_ORIGINAL_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TABLE_AFTER_ORIGINAL_LOGS {
        return;
    }
    log::write_line(format!(
        "preview-table-after-original {}",
        format_refresh_preview_table_probe(preview_arg)
    ));
}

fn log_costume_object_refresh_preview(
    state: usize,
    preview_arg: u32,
    before: Option<String>,
    after: Option<String>,
) {
    if before.is_none() && after.is_none() {
        return;
    }
    let index = COSTUME_OBJECT_REFRESH_PREVIEW_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TRACE_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let after = after.unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    if let Some(trace) = read_costume_object_update_trace(state) {
        if let Some(label) = law_ready_timeline_costume_trace_label(trace) {
            log_law_ready_timeline(
                "refresh-preview",
                format!(
                    "slot_label={label} preview_arg={preview_arg} before=[{before}] after=[{after}] frames={}",
                    format_stack_frames(&frames, 6)
                ),
            );
        }
    }
    log::write_line(format!(
        "Costume object refresh-preview state=0x{state:x} preview_arg={preview_arg} before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 8)
    ));
}

fn log_costume_object_refresh_preview_callsite(
    state: usize,
    preview_arg: u32,
    before: Option<String>,
    after: Option<String>,
) {
    if before.is_none() && after.is_none() {
        return;
    }
    let index = COSTUME_OBJECT_REFRESH_PREVIEW_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_REFRESH_PREVIEW_TRACE_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let after = after.unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    let trace = read_costume_object_update_trace(state);
    let identity = format_costume_object_refresh_preview_identity(trace, preview_arg);
    let preview_table_probe = format_refresh_preview_table_probe(preview_arg);
    let preview_table_compare = format_refresh_preview_table_compare(preview_arg, trace);
    if let Some(trace) = trace {
        if let Some(label) = law_ready_timeline_costume_trace_label(trace) {
            log_law_ready_timeline(
                "refresh-preview-callsite",
                format!(
                    "slot_label={label} caller_rva=0x{:x} identity=[{identity}] preview_table=[{preview_table_probe}] preview_table_compare=[{preview_table_compare}] context=[{}] before=[{before}] after=[{after}] frames={}",
                    COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA,
                    format_costume_object_update_trace(trace),
                    format_stack_frames(&frames, 6)
                ),
            );
        }
    }
    log::write_line(format!("preview-table-probe {preview_table_probe}"));
    log::write_line(format!("preview-table-compare {preview_table_compare}"));
    log::write_line(format!(
        "Costume object refresh-preview callsite caller_rva=0x{:x} state=0x{state:x} identity=[{identity}] preview_table=[{preview_table_probe}] preview_table_compare=[{preview_table_compare}] before=[{before}] after=[{after}] frames={}",
        COSTUME_OBJECT_REFRESH_PREVIEW_PRIMARY_CALLSITE_RVA,
        format_stack_frames(&frames, 8)
    ));
}

fn log_costume_object_apply_ready(
    object: usize,
    slot: u32,
    ready: u32,
    before: Option<String>,
    after: String,
) {
    let index = COSTUME_OBJECT_APPLY_READY_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_APPLY_READY_TRACE_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let update_context = format_last_costume_object_update_context(object);
    let busy_context = format_last_costume_object_busy_check();
    let frames = capture_stack_trace();
    if let Some(trace) = last_costume_object_update_trace_for_object(object) {
        if let Some(label) = law_ready_timeline_costume_trace_label(trace) {
            log_law_ready_timeline(
                "apply-ready",
                format!(
                    "slot_label={label} slot_arg={slot} ready={ready} busy=[{busy_context}] context=[{}] before=[{before}] after=[{after}] frames={}",
                    format_costume_object_update_trace(trace),
                    format_stack_frames(&frames, 6)
                ),
            );
            if label == "slot5-custom" {
                let index = SLOT5_APPLY_READY_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
                if index < MAX_SLOT5_APPLY_READY_STATE_LOGS {
                    log::write_line(format!(
                        "slot5-apply-ready-state object=0x{object:x} slot_arg={slot} ready={ready} busy=[{busy_context}] before=[{before}] after=[{after}] loader_object=[{}] alias=[{}] context=[{}] frames={}",
                        format_loader_object_final_state(
                            "slot5",
                            LAST_SLOT5_MODEL_READY_LOADER.load(Ordering::Relaxed),
                            LAST_SLOT5_MODEL_READY_OBJECT.load(Ordering::Relaxed),
                            u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                        ),
                        format_model_manager_alias_state(
                            "slot5",
                            u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                            Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)),
                        ),
                        format_costume_object_update_trace(trace),
                        format_stack_frames(&frames, 8)
                    ));
                }
            }
        }
    }
    log::write_line(format!(
        "Costume object apply-ready object=0x{object:x} slot={slot} ready={ready} busy=[{busy_context}] update=[{update_context}] before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 8)
    ));
}

fn format_last_costume_object_update_context(object: usize) -> String {
    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    let last_object = LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed);
    if state == 0 {
        return "none".to_string();
    }
    let matched = last_object == object;
    let Some(trace) = read_costume_object_update_trace(state) else {
        return format!(
            "state=0x{state:x} last_object=0x{last_object:x} matched={matched} trace=none"
        );
    };
    format!(
        "state=0x{state:x} last_object=0x{last_object:x} matched={matched} selected_index={} selected_layout={} selected_slot={} selected_kind={} object_pending={} load_args={}/{}/{} selected_variant={} selected_model_resource={} selected_preview_mapping={} selected_preview_mapped={}",
        format_optional_u32(trace.selected_index),
        format_optional_u32(trace.selected_layout),
        format_optional_u32(trace.selected_slot),
        format_optional_u32(trace.selected_kind),
        format_optional_u8(trace.object_pending),
        format_optional_u32(trace.selected_load_arg0),
        format_optional_u32(trace.selected_load_arg1),
        format_optional_u32(trace.selected_load_arg2),
        format_optional_u16(trace.selected_variant),
        format_optional_u16(trace.selected_model_resource),
        format_optional_u16(trace.selected_preview_mapping),
        format_optional_u32(trace.selected_preview_mapped_resource),
    )
}

fn format_last_costume_object_busy_check() -> String {
    let seq = LAST_COSTUME_OBJECT_BUSY_CHECK_SEQ.load(Ordering::Relaxed);
    if seq == 0 {
        return "none".to_string();
    }
    let model = LAST_COSTUME_OBJECT_BUSY_CHECK_MODEL.load(Ordering::Relaxed) as u32;
    let arg1 = LAST_COSTUME_OBJECT_BUSY_CHECK_ARG1.load(Ordering::Relaxed) as u32;
    let arg2 = LAST_COSTUME_OBJECT_BUSY_CHECK_ARG2.load(Ordering::Relaxed) as u32;
    let slot_label = match (model, arg1, arg2) {
        (308, color, 0) if color == u32::from(u16::MAX) => "slot3-oni",
        (resource, color, 0)
            if resource == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
                && color == u32::from(u16::MAX) =>
        {
            "slot5-custom"
        }
        _ => "other",
    };
    format!(
        "seq={seq} slot_label={slot_label} loader=0x{:x} args={}/{}/{} original_result={} result={} semantic=busy_or_pending",
        LAST_COSTUME_OBJECT_BUSY_CHECK_LOADER.load(Ordering::Relaxed),
        model,
        arg1,
        arg2,
        LAST_COSTUME_OBJECT_BUSY_CHECK_ORIGINAL_RESULT.load(Ordering::Relaxed),
        LAST_COSTUME_OBJECT_BUSY_CHECK_RESULT.load(Ordering::Relaxed),
    )
}

fn record_last_costume_object_busy_check(
    loader: usize,
    model_resource: u32,
    arg1: u32,
    arg2: u32,
    original_result: u64,
    result: u64,
) {
    LAST_COSTUME_OBJECT_BUSY_CHECK_LOADER.store(loader, Ordering::Relaxed);
    LAST_COSTUME_OBJECT_BUSY_CHECK_MODEL.store(model_resource as usize, Ordering::Relaxed);
    LAST_COSTUME_OBJECT_BUSY_CHECK_ARG1.store(arg1 as usize, Ordering::Relaxed);
    LAST_COSTUME_OBJECT_BUSY_CHECK_ARG2.store(arg2 as usize, Ordering::Relaxed);
    LAST_COSTUME_OBJECT_BUSY_CHECK_ORIGINAL_RESULT
        .store(original_result as usize, Ordering::Relaxed);
    LAST_COSTUME_OBJECT_BUSY_CHECK_RESULT.store(result as usize, Ordering::Relaxed);
    LAST_COSTUME_OBJECT_BUSY_CHECK_SEQ.fetch_add(1, Ordering::Relaxed);
}

fn last_costume_object_update_trace_for_object(object: usize) -> Option<CostumeObjectUpdateTrace> {
    if object == 0 {
        return None;
    }
    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    let last_object = LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed);
    if state == 0 || last_object != object {
        return None;
    }
    read_costume_object_update_trace(state)
}

fn last_costume_object_update_trace_for_parent(parent: usize) -> Option<CostumeObjectUpdateTrace> {
    if parent == 0 {
        return None;
    }
    let trace = last_costume_object_update_trace_for_object(
        LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed),
    )?;
    let object_child = trace.object_child.unwrap_or_default();
    let preview_child = LAST_COSTUME_PREVIEW_WIDGET_CHILD.load(Ordering::Relaxed);
    if parent == object_child || parent == preview_child {
        Some(trace)
    } else {
        None
    }
}

fn last_law_ready_timeline_trace_label() -> Option<(&'static str, CostumeObjectUpdateTrace)> {
    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    if state == 0 {
        return None;
    }
    let trace = read_costume_object_update_trace(state)?;
    law_ready_timeline_costume_trace_label(trace).map(|label| (label, trace))
}

fn law_ready_timeline_costume_trace_label(trace: CostumeObjectUpdateTrace) -> Option<&'static str> {
    if trace.selected_layout != Some(u32::from(LAW_MASTER_LAYOUT_ID)) {
        return None;
    }
    match (
        trace.selected_slot,
        trace.selected_variant,
        trace.selected_load_arg0,
        trace.selected_load_arg1,
        trace.selected_load_arg2,
    ) {
        (Some(slot), Some(variant), Some(model), Some(color), Some(arg2))
            if slot == LAW_EXTRA_SLOT_SOURCE_SLOT_INDEX as u32
                && variant == LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID
                && model == 308
                && color == u32::from(u16::MAX)
                && arg2 == 0 =>
        {
            Some("slot3-oni")
        }
        (Some(slot), Some(variant), Some(model), Some(color), Some(arg2))
            if slot == LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32
                && current_law_extra_slot_probe_variant_id() == Some(variant)
                && model == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
                && color == u32::from(u16::MAX)
                && arg2 == 0 =>
        {
            Some("slot5-custom")
        }
        _ => None,
    }
}

fn log_law_ready_timeline(stage: &str, detail: String) {
    let index = LAW_READY_TIMELINE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_LAW_READY_TIMELINE_LOGS {
        return;
    }
    let seq = LAW_READY_TIMELINE_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    log::write_line(format!(
        "Law ready timeline seq={seq} stage={stage} {detail}"
    ));
}

fn is_interesting_costume_object_model_ready_check(
    model_resource: u32,
    arg1: u32,
    arg2: u32,
) -> bool {
    model_resource == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || model_resource == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)
        || model_resource == u32::from(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
        || model_resource == u32::from(LAW_EXTRA_SLOT_METADATA_SOURCE_LAYOUT_ID)
        || is_law_menu_value(model_resource)
        || is_law_menu_value(arg1)
        || is_law_menu_value(arg2)
        || law_custom_slot_active()
}

#[derive(Debug, Clone, Copy)]
struct ModelReadyMatch {
    object: usize,
    flags20: u8,
    load_model: u32,
    load_color: u32,
    state28: u32,
    phase2c: u32,
    attach08: usize,
    token10: usize,
    attach_model30: u32,
    attach_color34: u16,
    field22c: u32,
    wait_3c4: u32,
    wait_3c8: u32,
}

fn find_costume_object_model_ready_match(
    loader: usize,
    model_resource: u32,
    arg1: u32,
    arg2: u32,
) -> Option<ModelReadyMatch> {
    if loader == 0 {
        return None;
    }

    let root = read_usize_field(loader, 0x78).filter(|address| *address != 0)?;
    let sentinel = read_usize_field(root, 0x18).filter(|address| *address != 0)?;
    let mut node = read_usize_absolute(sentinel)?;
    let mut count = 0usize;

    while node != 0 && node != sentinel && count < 24 {
        let object = read_usize_field(node, 0x10).filter(|address| *address != 0);
        let is_match = object
            .and_then(|address| {
                let object_model = read_u32_field(address, 0x378)?;
                let object_arg1 = read_u32_field(address, 0x37c)?;
                let object_arg2 = read_u32_field(address, 0x384)?;
                Some(object_model == model_resource && object_arg1 == arg1 && object_arg2 == arg2)
            })
            .unwrap_or(false);
        if is_match {
            let object = object?;
            return Some(ModelReadyMatch {
                object,
                flags20: read_u8_field(object, 0x20).unwrap_or_default(),
                load_model: read_u32_field(object, 0x18).unwrap_or(u32::MAX),
                load_color: read_u32_field(object, 0x1c).unwrap_or(u32::MAX),
                state28: read_u32_field(object, 0x28).unwrap_or(u32::MAX),
                phase2c: read_u32_field(object, 0x2c).unwrap_or(u32::MAX),
                attach08: read_usize_field(object, 0x08).unwrap_or_default(),
                token10: read_usize_field(object, 0x10).unwrap_or_default(),
                attach_model30: read_u32_field(object, 0x30).unwrap_or(u32::MAX),
                attach_color34: read_u16_field(object, 0x34).unwrap_or(u16::MAX),
                field22c: read_u32_field(object, 0x22c).unwrap_or(u32::MAX),
                wait_3c4: read_u32_field(object, 0x3c4).unwrap_or(u32::MAX),
                wait_3c8: read_u32_field(object, 0x3c8).unwrap_or(u32::MAX),
            });
        }
        node = read_usize_absolute(node)?;
        count += 1;
    }

    None
}

fn costume_object_update_trace_has_finalized_law_private_model(
    trace: CostumeObjectUpdateTrace,
) -> bool {
    trace.selected_layout == Some(u32::from(LAW_MASTER_LAYOUT_ID))
        && trace.selected_slot == Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32)
        && trace.selected_load_arg0
            == Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID))
        && trace.selected_load_arg1 == Some(u32::from(u16::MAX))
        && trace.selected_load_arg2 == Some(0)
        && trace.selected_model_resource == Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        && trace.object_pending == Some(0)
}

fn update_law_private_model_late_ready_gate(trace: Option<CostumeObjectUpdateTrace>) {
    let allowed = trace.is_some_and(costume_object_update_trace_has_finalized_law_private_model);
    LAW_PRIVATE_MODEL_READY_LATE_OVERRIDE_ALLOWED.store(allowed, Ordering::Relaxed);
}

fn law_private_model_ready_override(
    loader: usize,
    model_resource: u32,
    arg1: u32,
    arg2: u32,
) -> Option<String> {
    if model_resource != u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || arg1 != u32::from(u16::MAX)
        || arg2 != 0
    {
        return None;
    }

    let matched = find_costume_object_model_ready_match(loader, model_resource, arg1, arg2)?;
    if LAW_EXTRA_SLOT_LATE_MODEL_READY_OVERRIDE_ENABLED
        && LAW_PRIVATE_MODEL_READY_LATE_OVERRIDE_ALLOWED.load(Ordering::Relaxed)
    {
        return Some(format!(
            "late_forced=true object=0x{:x} detail=[{}]",
            matched.object,
            format_model_ready_match_detail(matched)
        ));
    }
    if !LAW_EXTRA_SLOT_MODEL_READY_OVERRIDE_ENABLED {
        return None;
    }
    Some(format!(
        "forced=true object=0x{:x} detail=[{}]",
        matched.object,
        format_model_ready_match_detail(matched)
    ))
}

fn patch_law_private_model_ready_flag_if_selected(
    loader: usize,
    model_resource: u32,
    arg1: u32,
    arg2: u32,
) -> Option<String> {
    if !LAW_EXTRA_SLOT_MODEL_READY_FLAG_PATCH_ENABLED
        || model_resource != u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || arg1 != u32::from(u16::MAX)
        || arg2 != 0
        || !LAW_PRIVATE_MODEL_READY_LATE_OVERRIDE_ALLOWED.load(Ordering::Relaxed)
    {
        return None;
    }

    let matched = find_costume_object_model_ready_match(loader, model_resource, arg1, arg2)?;
    let target = matched.flags20 | 0x01;
    if target == matched.flags20 {
        return Some(format!(
            "flag_patch=already object=0x{:x} detail=[{}]",
            matched.object,
            format_model_ready_match_detail(matched)
        ));
    }

    let patch_address = matched.object + 0x20;
    let patch_bytes = [target];
    let write_ok = matches!(
        win::write_process_memory(patch_address, &patch_bytes),
        Some(written) if written == patch_bytes.len()
    );
    let after = read_u8_field(matched.object, 0x20).unwrap_or_default();
    Some(format!(
        "flag_patch=true object=0x{:x} before=0x{:02x} target=0x{:02x} after=0x{:02x} write_ok={} detail=[{}]",
        matched.object,
        matched.flags20,
        target,
        after,
        write_ok,
        format_model_ready_match_detail(matched)
    ))
}

fn format_model_ready_match_detail(matched: ModelReadyMatch) -> String {
    format!(
        "load={}/{} state28={} phase2c={} flags20=0x{:02x} attach08={} token10={} attach30/34={}/{} f22c={} wait={}/{}",
        matched.load_model,
        matched.load_color,
        matched.state28,
        matched.phase2c,
        matched.flags20,
        format_optional_address(Some(matched.attach08).filter(|address| *address != 0)),
        format_optional_address(Some(matched.token10).filter(|address| *address != 0)),
        matched.attach_model30,
        matched.attach_color34,
        matched.field22c,
        matched.wait_3c4,
        matched.wait_3c8,
    )
}

fn format_model_ready_object_detail(object: usize) -> String {
    format!(
        "load={}/{} state28={} phase2c={} flags20={} attach08={} token10={} attach30/34={}/{} f22c={} wait={}/{}",
        format_optional_u32(read_u32_field(object, 0x18)),
        format_optional_u32(read_u32_field(object, 0x1c)),
        format_optional_u32(read_u32_field(object, 0x28)),
        format_optional_u32(read_u32_field(object, 0x2c)),
        format_optional_u8(read_u8_field(object, 0x20)),
        format_optional_address(read_usize_field(object, 0x08).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(object, 0x10).filter(|address| *address != 0)),
        format_optional_u32(read_u32_field(object, 0x30)),
        format_optional_u16_decimal(read_u16_field(object, 0x34)),
        format_optional_u32(read_u32_field(object, 0x22c)),
        format_optional_u32(read_u32_field(object, 0x3c4)),
        format_optional_u32(read_u32_field(object, 0x3c8)),
    )
}

fn record_loader_object_match(
    slot_label: &'static str,
    loader: usize,
    matched: Option<ModelReadyMatch>,
) {
    let object = matched.map(|matched| matched.object).unwrap_or_default();
    match slot_label {
        "slot3-oni" => {
            LAST_ONI_MODEL_READY_LOADER.store(loader, Ordering::Relaxed);
            LAST_ONI_MODEL_READY_OBJECT.store(object, Ordering::Relaxed);
            LAST_ONI_MODEL_READY_SEQ.fetch_add(1, Ordering::Relaxed);
        }
        "slot5-custom" => {
            LAST_SLOT5_MODEL_READY_LOADER.store(loader, Ordering::Relaxed);
            LAST_SLOT5_MODEL_READY_OBJECT.store(object, Ordering::Relaxed);
            LAST_SLOT5_MODEL_READY_SEQ.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
}

fn loader_ready_checkpoint_match(matched: ModelReadyMatch) -> bool {
    matched.state28 == 7 && matched.phase2c == 2 && matched.flags20 == 0x05
}

fn current_law_ready_timeline_trace_for_label(
    slot_label: &'static str,
) -> Option<CostumeObjectUpdateTrace> {
    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    if state == 0 {
        return None;
    }
    let trace = read_costume_object_update_trace(state)?;
    (law_ready_timeline_costume_trace_label(trace) == Some(slot_label)).then_some(trace)
}

fn checkpoint_pending_text(value: usize) -> String {
    if value == usize::MAX {
        "none".to_string()
    } else {
        format!("0x{:02x}", value & 0xff)
    }
}

fn store_loader_ready_checkpoint(
    slot_label: &'static str,
    loader: usize,
    matched: ModelReadyMatch,
    trace: Option<CostumeObjectUpdateTrace>,
    frames: &[usize],
) {
    if !loader_ready_checkpoint_match(matched) {
        return;
    }

    let pending = trace
        .and_then(|trace| trace.object_pending)
        .map(usize::from)
        .unwrap_or(usize::MAX);
    let (
        log_name,
        model,
        source_model,
        last_loader,
        last_object,
        seq,
        last_pending,
        last_flags20,
        last_state28,
        last_phase2c,
    ) = match slot_label {
        "slot3-oni" => (
            "oni-loader-ready-checkpoint",
            308,
            None,
            &LAST_ONI_READY_CHECKPOINT_LOADER,
            &LAST_ONI_READY_CHECKPOINT_OBJECT,
            &LAST_ONI_READY_CHECKPOINT_SEQ,
            &LAST_ONI_READY_CHECKPOINT_PENDING,
            &LAST_ONI_READY_CHECKPOINT_FLAGS20,
            &LAST_ONI_READY_CHECKPOINT_STATE28,
            &LAST_ONI_READY_CHECKPOINT_PHASE2C,
        ),
        "slot5-custom" => (
            "slot5-loader-ready-checkpoint",
            u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
            Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)),
            &LAST_SLOT5_READY_CHECKPOINT_LOADER,
            &LAST_SLOT5_READY_CHECKPOINT_OBJECT,
            &LAST_SLOT5_READY_CHECKPOINT_SEQ,
            &LAST_SLOT5_READY_CHECKPOINT_PENDING,
            &LAST_SLOT5_READY_CHECKPOINT_FLAGS20,
            &LAST_SLOT5_READY_CHECKPOINT_STATE28,
            &LAST_SLOT5_READY_CHECKPOINT_PHASE2C,
        ),
        _ => return,
    };

    if seq.load(Ordering::Relaxed) != 0 && last_object.load(Ordering::Relaxed) == matched.object {
        return;
    }

    last_loader.store(loader, Ordering::Relaxed);
    last_object.store(matched.object, Ordering::Relaxed);
    last_pending.store(pending, Ordering::Relaxed);
    last_flags20.store(matched.flags20 as usize, Ordering::Relaxed);
    last_state28.store(matched.state28 as usize, Ordering::Relaxed);
    last_phase2c.store(matched.phase2c as usize, Ordering::Relaxed);
    let checkpoint_seq = seq.fetch_add(1, Ordering::Relaxed) + 1;

    let index = LOADER_READY_CHECKPOINT_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_LOADER_READY_CHECKPOINT_LOGS {
        let context = trace
            .map(format_costume_object_update_trace)
            .unwrap_or_else(|| "none".to_string());
        log::write_line(format!(
            "{log_name} seq={checkpoint_seq} model={model} loader=0x{loader:x} object=0x{:x} state28={} phase2c={} flags20=0x{:02x} attach08={} token10={} wait3c4={} wait3c8={} object_pending={} detail=[{}] context=[{context}] frames={}",
            matched.object,
            matched.state28,
            matched.phase2c,
            matched.flags20,
            format_optional_address((matched.attach08 != 0).then_some(matched.attach08)),
            format_optional_address((matched.token10 != 0).then_some(matched.token10)),
            matched.wait_3c4,
            matched.wait_3c8,
            checkpoint_pending_text(pending),
            format_model_ready_match_detail(matched),
            format_stack_frames(frames, 8),
        ));
    }

    let alias_index = MODEL_MANAGER_ALIAS_CHECKPOINT_LOGS.fetch_add(1, Ordering::Relaxed);
    if alias_index < MAX_MODEL_MANAGER_ALIAS_CHECKPOINT_LOGS {
        log::write_line(format!(
            "model-manager-alias-checkpoint {} frames={}",
            format_model_manager_alias_state(
                if slot_label == "slot5-custom" {
                    "slot5"
                } else {
                    "oni"
                },
                model,
                source_model,
            ),
            format_stack_frames(frames, 8),
        ));
    }

    if slot_label == "slot5-custom" && pending == 1 {
        let index = SLOT5_PENDING_READY_GAP_LOGS.fetch_add(1, Ordering::Relaxed);
        if index < MAX_SLOT5_PENDING_READY_GAP_LOGS {
            log::write_line(format!(
                "slot5-pending-ready-gap ready=true object_pending=0x01 checkpoint_seq={checkpoint_seq} loader=0x{loader:x} object=0x{:x} model={model} detail=[{}] context=[{}] frames={}",
                matched.object,
                format_model_ready_match_detail(matched),
                trace
                    .map(format_costume_object_update_trace)
                    .unwrap_or_else(|| "none".to_string()),
                format_stack_frames(frames, 8),
            ));
        }
    }

    if slot_label == "slot5-custom" {
        maybe_request_slot5_refresh_at_ready_checkpoint(
            checkpoint_seq,
            loader,
            matched,
            trace,
            frames,
        );
    }
}

fn slot5_ready_checkpoint_consumable(
    matched: ModelReadyMatch,
    trace: Option<CostumeObjectUpdateTrace>,
) -> bool {
    let Some(trace) = trace else {
        return false;
    };
    trace.selected_layout == Some(u32::from(LAW_MASTER_LAYOUT_ID))
        && trace.selected_slot == Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32)
        && trace.selected_variant == current_law_extra_slot_probe_variant_id()
        && trace.selected_model_resource == Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        && trace.selected_preview_mapped_resource
            == Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
        && trace.selected_load_arg0
            == Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID))
        && trace.selected_load_arg1 == Some(u32::from(u16::MAX))
        && trace.selected_load_arg2 == Some(0)
        && trace.object_pending == Some(0)
        && trace.controller.is_some_and(|address| address != 0)
        && trace
            .controller_model_loader
            .is_some_and(|address| address != 0)
        && loader_ready_checkpoint_match(matched)
}

fn slot5_current_child8_status_text() -> (bool, String) {
    let widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
    if widget == 0 {
        return (false, "none".to_string());
    }
    let snapshot = read_preview_child8_snapshot(widget);
    (
        snapshot.is_slot5_good(),
        format_preview_child8_snapshot(snapshot),
    )
}

fn slot5_ready_checkpoint_defer_reason(resource_911_active: bool, child8_good: bool) -> String {
    let mut reasons = Vec::new();
    if !resource_911_active {
        reasons.push("resource911_stale");
    }
    if !child8_good {
        reasons.push("child8_missing");
    }
    if reasons.is_empty() {
        "ready".to_string()
    } else {
        reasons.join(" ")
    }
}

fn preview_resource_911_is_stale(entry: Option<PreviewResourceResolveMatch>) -> bool {
    entry.is_some_and(|entry| {
        read_u32_field(entry.entry_ptr, PREVIEW_RESOURCE_SLOT_ID_OFFSET)
            == Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
            && entry.state == Some(0)
            && entry.marker == Some(1)
    })
}

fn slot5_checkpoint_queue911_text(resource_911: Option<PreviewResourceResolveMatch>) -> String {
    let widget_queue = LAST_SLOT5_PREVIEW_WIDGET_QUEUE.load(Ordering::Relaxed);
    let model_queue = LAST_SLOT5_PREVIEW_MODEL_QUEUE.load(Ordering::Relaxed);
    let (queue_source, queue) = if widget_queue != 0 {
        ("widget_queue", widget_queue)
    } else if model_queue != 0 {
        ("model_queue", model_queue)
    } else {
        ("none", 0)
    };
    if queue == 0 {
        return "queue_source=none queue=none queue_has911=false queue_index=none queue_status=queue_missing queue_state=[none]".to_string();
    }

    let probe = preview_resource_attach_queue_probe(queue, LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID);
    let queue_has911 = !probe.entries.is_empty();
    let queue_status = if !queue_has911 {
        "queue_missing"
    } else if preview_resource_match_is_active(resource_911) {
        "queue_already_present_active"
    } else if preview_resource_911_is_stale(resource_911) {
        "queue_already_present_stale"
    } else {
        "queue_already_present_unknown"
    };

    format!(
        "queue_source={queue_source} queue=0x{queue:x} queue_has911={queue_has911} queue_index={} queue_status={queue_status} queue_state=[{}]",
        format_preview_resource_queue_first_index(Some(&probe)),
        format_preview_resource_attach_queue(queue)
    )
}

fn slot5_window_trace_match(trace: CostumeObjectUpdateTrace) -> bool {
    trace.selected_layout == Some(u32::from(LAW_MASTER_LAYOUT_ID))
        && trace.selected_slot == Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32)
        && trace.selected_variant == current_law_extra_slot_probe_variant_id()
        && trace.selected_model_resource == Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        && trace.selected_preview_mapped_resource
            == Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
}

fn slot5_window_frame_boundary<'a>(fallback: &'a str, frames: &[usize]) -> String {
    const TARGETS: &[(usize, &str)] = &[
        (0x149864f, "game+0x149864f"),
        (0x1490acd, "game+0x1490acd"),
        (0x1490b95, "game+0x1490b95"),
        (0x149169b, "game+0x149169b"),
        (0x16150de, "game+0x16150de"),
        (0x1589443, "game+0x1589443"),
        (0x16150fa, "game+0x16150fa"),
    ];
    frames
        .iter()
        .find_map(|frame| {
            let rva = game_frame_rva(*frame)?;
            TARGETS
                .iter()
                .find_map(|(target, label)| (rva == *target).then_some((*label).to_string()))
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn slot5_ordering_boundary_target(boundary: &str) -> bool {
    matches!(
        boundary,
        "game+0x1490acd" | "game+0x1490b95" | "game+0x149169b"
    )
}

fn slot5_ordering_next_action(controller_present: bool, has_consumable_widget_state: bool) -> &'static str {
    if controller_present && has_consumable_widget_state {
        "consume_before_controller_loss"
    } else if controller_present {
        "trace_dispatch"
    } else if has_consumable_widget_state {
        "move_911_attach_earlier"
    } else {
        "move_widget_build_earlier"
    }
}

fn slot5_pre_controller_loss_main_state() -> (usize, bool, String, bool, String, String) {
    let widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
    let widget_seq = LAST_SLOT5_PREVIEW_WIDGET_SEQ.load(Ordering::Relaxed);
    let model_queue = LAST_SLOT5_PREVIEW_MODEL_QUEUE.load(Ordering::Relaxed);
    let widget_queue = LAST_SLOT5_PREVIEW_WIDGET_QUEUE.load(Ordering::Relaxed);
    let queue = if model_queue != 0 {
        model_queue
    } else {
        widget_queue
    };
    let queue_probe = (queue != 0).then(|| {
        preview_resource_attach_queue_probe(queue, LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    });
    let queue_has911 = queue_probe
        .as_ref()
        .is_some_and(|probe| !probe.entries.is_empty());
    let queue_text = if queue != 0 {
        format!(
            "queue=0x{queue:x} queue_has911={queue_has911} queue_index={} model_queue={} widget_queue={} queue_state=[{}]",
            format_preview_resource_queue_first_index(queue_probe.as_ref()),
            format_optional_address((model_queue != 0).then_some(model_queue)),
            format_optional_address((widget_queue != 0).then_some(widget_queue)),
            format_preview_resource_attach_queue(queue)
        )
    } else {
        "queue=none queue_has911=false queue_index=none model_queue=none widget_queue=none queue_state=[none]".to_string()
    };
    let (child8_good, child8) = slot5_current_child8_status_text();
    let widget_text = if widget != 0 && widget_seq != 0 {
        format!("widget=0x{widget:x} widget_seq={widget_seq}")
    } else {
        "widget=none widget_seq=0".to_string()
    };

    (
        widget,
        queue_has911,
        queue_text,
        child8_good,
        child8,
        widget_text,
    )
}

fn maybe_refresh_slot5_before_controller_loss(
    checkpoint_seq: usize,
    boundary: &str,
    phase: &str,
    state: usize,
    trace: CostumeObjectUpdateTrace,
    frames: &[usize],
    has_consumable_widget_state: bool,
) {
    if !LAW_SLOT5_REFRESH_WHEN_WIDGET_EXISTS_BEFORE_CONTROLLER_LOSS_ENABLED
        || !has_consumable_widget_state
        || !preview_resource_match_is_active(read_tracked_preview_resource_states().resource_911)
        || !slot5_checkpoint_snapshot_valid()
        || trace.controller.is_none()
        || trace.controller_model_loader.is_none()
        || state == 0
        || SLOT5_PRE_CONTROLLER_LOSS_REFRESH_REPLAYED_SEQ.load(Ordering::Relaxed) == checkpoint_seq
    {
        return;
    }
    let original_address = COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        let index = SLOT5_PRE_CONTROLLER_LOSS_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
        if index < MAX_SLOT5_PRE_CONTROLLER_LOSS_REFRESH_LOGS {
            log::write_line(format!(
                "slot5-before-controller-loss-not-consumed reason=no_original boundary={boundary} phase={phase} checkpoint_seq={checkpoint_seq} state=0x{state:x} resource911_live=[{}] child8=[{}] context=[{}] frames={}",
                format_preview_resource_911_live_state(),
                slot5_current_child8_status_text().1,
                format_costume_object_update_trace(trace),
                format_stack_frames(frames, 8),
            ));
        }
        return;
    }
    if SLOT5_PRE_CONTROLLER_LOSS_REFRESH_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let preview_arg = LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID;
    let before = read_costume_object_refresh_preview_trace(state, preview_arg)
        .unwrap_or_else(|| "none".to_string());
    let candidate_seq_before = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let original: CostumeObjectRefreshPreviewFn = unsafe { std::mem::transmute(original_address) };
    unsafe { original(state, preview_arg) };
    let after_trace = read_costume_object_update_trace(state);
    let after = read_costume_object_refresh_preview_trace(state, preview_arg)
        .unwrap_or_else(|| "none".to_string());
    let candidate_seq_after = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let consumed = candidate_seq_after > candidate_seq_before;
    SLOT5_PRE_CONTROLLER_LOSS_REFRESH_REPLAYED_SEQ.store(checkpoint_seq, Ordering::Relaxed);

    let index = SLOT5_PRE_CONTROLLER_LOSS_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_PRE_CONTROLLER_LOSS_REFRESH_LOGS {
        log::write_line(format!(
            "law-slot5-refresh-before-controller-loss-run boundary={boundary} phase={phase} checkpoint_seq={checkpoint_seq} state=0x{state:x} preview_arg={preview_arg} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} loader_valid={} before=[{before}] after=[{after}] resource911_live=[{}] child8=[{}] before_context=[{}] after_context=[{}] frames={}",
            slot5_checkpoint_snapshot_valid(),
            format_preview_resource_911_live_state(),
            slot5_current_child8_status_text().1,
            format_costume_object_update_trace(trace),
            after_trace
                .map(format_costume_object_update_trace)
                .unwrap_or_else(|| "none".to_string()),
            format_stack_frames(frames, 8),
        ));
        let line = if consumed {
            "slot5-before-controller-loss-consumed"
        } else {
            "slot5-before-controller-loss-not-consumed"
        };
        let reason = if consumed { "candidate_update" } else { "no_candidate" };
        log::write_line(format!(
            "{line} reason={reason} boundary={boundary} phase={phase} checkpoint_seq={checkpoint_seq} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} resource911_live=[{}] child8=[{}] context=[{}] frames={}",
            format_preview_resource_911_live_state(),
            slot5_current_child8_status_text().1,
            after_trace
                .map(format_costume_object_update_trace)
                .unwrap_or_else(|| "none".to_string()),
            format_stack_frames(frames, 8),
        ));
    }

    SLOT5_PRE_CONTROLLER_LOSS_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
}

fn poll_slot5_pre_controller_loss_window(
    boundary: &str,
    phase: &str,
    trace: Option<CostumeObjectUpdateTrace>,
    frames: &[usize],
) {
    let request_seq = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_SEQ.load(Ordering::Relaxed);
    if request_seq == 0 {
        return;
    }
    let Some(trace) = trace else {
        return;
    };
    if !slot5_window_trace_match(trace) {
        return;
    }
    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    let frame_boundary = slot5_window_frame_boundary(boundary, frames);
    let controller_present = trace.controller.is_some() && trace.controller_model_loader.is_some();
    let (widget, queue_has911, queue_text, child8_good, child8, widget_text) =
        slot5_pre_controller_loss_main_state();
    let has_widget = widget != 0 && LAST_SLOT5_PREVIEW_WIDGET_SEQ.load(Ordering::Relaxed) != 0;
    let has_consumable_widget_state = has_widget || queue_has911 || child8_good;
    let snapshot_reason = slot5_checkpoint_snapshot_reason();
    let resource_911 = read_tracked_preview_resource_states().resource_911;
    if !SLOT5_CONTROLLER_RESTORE_BRANCH_ABORTED_LOGGED.swap(true, Ordering::AcqRel) {
        log::write_line(
            "slot5-controller-restore-branch-aborted reason=recycled_controller controller_restore_flags=disabled slot5-next-axis=read_only_ordering_trace"
                .to_string(),
        );
    }
    let axis = if !controller_present && has_consumable_widget_state {
        "slot5-widget-build-after-controller-loss confirmed=true slot5-next-axis=move_widget_or_911_path_earlier"
    } else if !controller_present {
        "slot5-controller-lost-before-widget confirmed=true slot5-next-axis=move_widget_or_911_path_earlier"
    } else if has_consumable_widget_state {
        "slot5-first-widget-before-controller-loss confirmed=true slot5-next-axis=consume_before_controller_loss"
    } else {
        "slot5-window-waiting-for-widget confirmed=false slot5-next-axis=trace_1490acd_1490b95"
    };
    let index = SLOT5_PRE_CONTROLLER_LOSS_WINDOW_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_PRE_CONTROLLER_LOSS_WINDOW_LOGS {
        log::write_line(format!(
            "slot5-pre-controller-loss-window boundary={frame_boundary} probe={boundary} phase={phase} checkpoint_seq={request_seq} state={} controller={} controller_loader={} controller_present={controller_present} snapshot_reason={snapshot_reason} loader_valid={} main_widget=[{widget_text}] queue911=[{queue_text}] child8=[{child8}] resource911_live=[{}] verdict=[{axis}] context=[{}] frames={}",
            format_optional_address((state != 0).then_some(state)),
            format_optional_address(trace.controller),
            format_optional_address(trace.controller_model_loader),
            slot5_checkpoint_snapshot_valid(),
            format_preview_resource_match_state(resource_911),
            format_costume_object_update_trace(trace),
            format_stack_frames(frames, 8),
        ));
    }

    if slot5_ordering_boundary_target(&frame_boundary) {
        let index = SLOT5_ORDERING_BOUNDARY_LOGS.fetch_add(1, Ordering::Relaxed);
        if index < MAX_SLOT5_ORDERING_BOUNDARY_LOGS {
            log::write_line(format!(
                "slot5-ordering-boundary boundary={frame_boundary} probe={boundary} phase={phase} checkpoint_seq={request_seq} controller={} controller_loader={} controller_state={} loader_valid={} widget=[{widget_text}] queue_has911={queue_has911} queue911=[{queue_text}] child8_good={child8_good} child8=[{child8}] resource911=[{}] candidate_seq={} snapshot_reason={snapshot_reason} slot5-next-action={} context=[{}] frames={}",
                format_optional_address(trace.controller),
                format_optional_address(trace.controller_model_loader),
                if controller_present { "present" } else { "none" },
                slot5_checkpoint_snapshot_valid(),
                format_preview_resource_match_state(resource_911),
                SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed),
                slot5_ordering_next_action(controller_present, has_consumable_widget_state),
                format_costume_object_update_trace(trace),
                format_stack_frames(frames, 8),
            ));
        }
    }

    if controller_present && has_consumable_widget_state {
        if !SLOT5_FIRST_WIDGET_BEFORE_CONTROLLER_LOSS_LOGGED.swap(true, Ordering::AcqRel) {
            log::write_line(format!(
                "slot5-first-widget-before-controller-loss boundary={frame_boundary} probe={boundary} phase={phase} checkpoint_seq={request_seq} state={} main_widget=[{widget_text}] queue911=[{queue_text}] child8=[{child8}] resource911_live=[{}] context=[{}] frames={}",
                format_optional_address((state != 0).then_some(state)),
                format_preview_resource_match_state(resource_911),
                format_costume_object_update_trace(trace),
                format_stack_frames(frames, 8),
            ));
        }
        maybe_refresh_slot5_before_controller_loss(
            request_seq,
            &frame_boundary,
            phase,
            state,
            trace,
            frames,
            has_consumable_widget_state,
        );
    } else if !controller_present && has_consumable_widget_state {
        if !SLOT5_WIDGET_AFTER_CONTROLLER_LOSS_LOGGED.swap(true, Ordering::AcqRel) {
            log::write_line(format!(
                "slot5-widget-after-controller-loss boundary={frame_boundary} probe={boundary} phase={phase} checkpoint_seq={request_seq} state={} main_widget=[{widget_text}] queue911=[{queue_text}] child8=[{child8}] resource911_live=[{}] verdict=[slot5-widget-build-after-controller-loss confirmed=true slot5-next-axis=move_widget_or_911_path_earlier] context=[{}] frames={}",
                format_optional_address((state != 0).then_some(state)),
                format_preview_resource_match_state(resource_911),
                format_costume_object_update_trace(trace),
                format_stack_frames(frames, 8),
            ));
        }
    } else if !controller_present {
        if !SLOT5_CONTROLLER_LOST_BEFORE_WIDGET_LOGGED.swap(true, Ordering::AcqRel) {
            log::write_line(format!(
                "slot5-controller-lost-before-widget boundary={frame_boundary} probe={boundary} phase={phase} checkpoint_seq={request_seq} state={} main_widget=[{widget_text}] queue911=[{queue_text}] child8=[{child8}] resource911_live=[{}] verdict=[slot5-controller-lost-before-widget confirmed=true slot5-next-axis=move_widget_or_911_path_earlier] context=[{}] frames={}",
                format_optional_address((state != 0).then_some(state)),
                format_preview_resource_match_state(resource_911),
                format_costume_object_update_trace(trace),
                format_stack_frames(frames, 8),
            ));
        }
    }
}

fn maybe_run_slot5_early_checkpoint_911_refresh(
    checkpoint_seq: usize,
    loader: usize,
    loader_object: usize,
    state_object: usize,
    controller: usize,
    controller_loader: usize,
    matched: ModelReadyMatch,
    trace: CostumeObjectUpdateTrace,
    resource_911: Option<PreviewResourceResolveMatch>,
    frames: &[usize],
) {
    if !LAW_SLOT5_EARLY_REACTIVATE_911_AT_READY_CHECKPOINT_ENABLED
        || !preview_resource_911_is_stale(resource_911)
    {
        return;
    }
    if SLOT5_EARLY_CHECKPOINT_911_REFRESH_REPLAYED_SEQ.load(Ordering::Relaxed) == checkpoint_seq {
        return;
    }
    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    if state == 0 || controller == 0 || controller_loader == 0 {
        return;
    }
    let original_address = COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.load(Ordering::Acquire);
    let queue911 = slot5_checkpoint_queue911_text(resource_911);
    if original_address == 0 {
        let index = SLOT5_EARLY_CHECKPOINT_911_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
        if index < MAX_SLOT5_EARLY_CHECKPOINT_911_REFRESH_LOGS {
            log::write_line(format!(
                "slot5-ready-checkpoint-early-not-consumed reason=no_original checkpoint_seq={checkpoint_seq} state=0x{state:x} loader=0x{loader:x} loader_object=0x{loader_object:x} state_object={} controller={} controller_loader={} resource911_live=[{}] queue911=[{}] child8=[{}] context=[{}] frames={}",
                format_optional_address((state_object != 0).then_some(state_object)),
                format_optional_address(Some(controller)),
                format_optional_address(Some(controller_loader)),
                format_preview_resource_match_state(resource_911),
                queue911,
                slot5_current_child8_status_text().1,
                format_costume_object_update_trace(trace),
                format_stack_frames(frames, 8),
            ));
        }
        return;
    }
    if SLOT5_EARLY_CHECKPOINT_911_REFRESH_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let before = resource_911;
    let Some(before_entry) = before else {
        SLOT5_EARLY_CHECKPOINT_911_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
        return;
    };
    let state_written = write_u32_field_checked(before_entry.entry_ptr, PREVIEW_RESOURCE_SLOT_STATE_OFFSET, 1);
    let marker_written = write_u32_field_checked(before_entry.entry_ptr, PREVIEW_RESOURCE_SLOT_MARKER_OFFSET, 0);
    let after_reactivate =
        preview_resource_resolve_match(before_entry.table_base, LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID);

    let candidate_seq_before = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let preview_arg = LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID;
    let before_refresh = read_costume_object_refresh_preview_trace(state, preview_arg)
        .unwrap_or_else(|| "none".to_string());
    let loader_valid_before = slot5_checkpoint_snapshot_valid();
    if state_written && marker_written {
        let original: CostumeObjectRefreshPreviewFn = unsafe { std::mem::transmute(original_address) };
        unsafe { original(state, preview_arg) };
    }
    let after_trace = read_costume_object_update_trace(state);
    let after_refresh = read_costume_object_refresh_preview_trace(state, preview_arg)
        .unwrap_or_else(|| "none".to_string());
    let candidate_seq_after = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let consumed = candidate_seq_after > candidate_seq_before;
    SLOT5_EARLY_CHECKPOINT_911_REFRESH_REPLAYED_SEQ.store(checkpoint_seq, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REPLAYED_SEQ.store(checkpoint_seq, Ordering::Relaxed);

    let index = SLOT5_EARLY_CHECKPOINT_911_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_EARLY_CHECKPOINT_911_REFRESH_LOGS {
        log::write_line(format!(
            "law-slot5-early-checkpoint-reactivate-911 checkpoint_seq={checkpoint_seq} state=0x{state:x} loader=0x{loader:x} loader_object=0x{loader_object:x} state_object={} controller={} controller_loader={} state_written={state_written} marker_written={marker_written} before=[{}] after=[{}] queue911=[{}] child8=[{}] detail=[{}] context=[{}] frames={}",
            format_optional_address((state_object != 0).then_some(state_object)),
            format_optional_address(Some(controller)),
            format_optional_address(Some(controller_loader)),
            format_preview_resource_match_state(before),
            format_preview_resource_match_state(after_reactivate),
            queue911,
            slot5_current_child8_status_text().1,
            format_model_ready_match_detail(matched),
            format_costume_object_update_trace(trace),
            format_stack_frames(frames, 8),
        ));
        let loader_valid_after = slot5_checkpoint_snapshot_valid();
        log::write_line(format!(
            "law-slot5-refresh-at-ready-after-early-911-run checkpoint_seq={checkpoint_seq} state=0x{state:x} preview_arg={preview_arg} controller=present loader_valid={} loader_valid_before={} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} before=[{before_refresh}] after=[{after_refresh}] resource911_live=[{}] child8=[{}] before_context=[{}] after_context=[{}] frames={}",
            loader_valid_after,
            loader_valid_before,
            format_preview_resource_911_live_state(),
            slot5_current_child8_status_text().1,
            format_costume_object_update_trace(trace),
            after_trace
                .map(format_costume_object_update_trace)
                .unwrap_or_else(|| "none".to_string()),
            format_stack_frames(frames, 8),
        ));
        let line = if consumed {
            "slot5-ready-checkpoint-early-consumed"
        } else {
            "slot5-ready-checkpoint-early-not-consumed"
        };
        let reason = if state_written && marker_written {
            if consumed {
                "candidate_update"
            } else {
                "no_candidate"
            }
        } else {
            "reactivate_write_failed"
        };
        log::write_line(format!(
            "{line} reason={reason} checkpoint_seq={checkpoint_seq} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} loader_valid={} resource911_live=[{}] child8=[{}] context=[{}] frames={}",
            slot5_checkpoint_snapshot_valid(),
            format_preview_resource_911_live_state(),
            slot5_current_child8_status_text().1,
            after_trace
                .map(format_costume_object_update_trace)
                .unwrap_or_else(|| "none".to_string()),
            format_stack_frames(frames, 8),
        ));
    }

    SLOT5_EARLY_CHECKPOINT_911_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
}

fn maybe_request_slot5_refresh_at_ready_checkpoint(
    checkpoint_seq: usize,
    loader: usize,
    matched: ModelReadyMatch,
    trace: Option<CostumeObjectUpdateTrace>,
    frames: &[usize],
) {
    let ready = LAW_SLOT5_REFRESH_PREVIEW_AT_READY_CHECKPOINT_ENABLED
        && slot5_ready_checkpoint_consumable(matched, trace);
    let Some(trace) = trace else {
        return;
    };
    if !ready {
        return;
    }
    if SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_SEQ.load(Ordering::Relaxed) == checkpoint_seq
        || SLOT5_READY_CHECKPOINT_REFRESH_REPLAYED_SEQ.load(Ordering::Relaxed) == checkpoint_seq
    {
        return;
    }

    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    let state_object = trace.object.unwrap_or_default();
    let loader_object = matched.object;
    let controller = trace.controller.unwrap_or_default();
    let controller_loader = trace.controller_model_loader.unwrap_or_default();
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_SEQ.store(checkpoint_seq, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE.store(state, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE_OBJECT.store(state_object, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_OBJECT.store(loader_object, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_LOADER.store(loader, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.store(controller, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER_LOADER
        .store(controller_loader, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_ATTACH08.store(matched.attach08, Ordering::Relaxed);
    SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_TOKEN10.store(matched.token10, Ordering::Relaxed);

    let resource_911 = read_tracked_preview_resource_states().resource_911;
    let resource_911_active = preview_resource_match_is_active(resource_911);
    let (child8_good, child8) = slot5_current_child8_status_text();
    let defer_reason = slot5_ready_checkpoint_defer_reason(resource_911_active, child8_good);
    let index = SLOT5_READY_CHECKPOINT_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_READY_CHECKPOINT_REFRESH_LOGS {
        log::write_line(format!(
            "slot5-ready-checkpoint-consumable checkpoint_seq={checkpoint_seq} state={} loader=0x{loader:x} loader_object=0x{loader_object:x} state_object={} controller={} controller_loader={} preview_arg={} resource911_live=[{}] child8=[{}] detail=[{}] context=[{}] frames={}",
            format_optional_address((state != 0).then_some(state)),
            format_optional_address((state_object != 0).then_some(state_object)),
            format_optional_address((controller != 0).then_some(controller)),
            format_optional_address((controller_loader != 0).then_some(controller_loader)),
            LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID,
            format_preview_resource_911_live_state(),
            child8,
            format_model_ready_match_detail(matched),
            format_costume_object_update_trace(trace),
            format_stack_frames(frames, 8),
        ));
        log::write_line(format!(
            "slot5-ready-checkpoint-deferred reason={defer_reason} checkpoint_seq={checkpoint_seq} state={} preview_arg={} loader=0x{loader:x} loader_object=0x{loader_object:x} state_object={} controller={} controller_loader={} attach08={} token10={} resource911_live=[{}] child8=[{}] context=[{}] frames={}",
            format_optional_address((state != 0).then_some(state)),
            LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID,
            format_optional_address((state_object != 0).then_some(state_object)),
            format_optional_address((controller != 0).then_some(controller)),
            format_optional_address((controller_loader != 0).then_some(controller_loader)),
            format_optional_address((matched.attach08 != 0).then_some(matched.attach08)),
            format_optional_address((matched.token10 != 0).then_some(matched.token10)),
            format_preview_resource_match_state(resource_911),
            child8,
            format_costume_object_update_trace(trace),
            format_stack_frames(frames, 8),
        ));
    }

    maybe_run_slot5_early_checkpoint_911_refresh(
        checkpoint_seq,
        loader,
        loader_object,
        state_object,
        controller,
        controller_loader,
        matched,
        trace,
        resource_911,
        frames,
    );
    poll_slot5_pre_controller_loss_window("slot5-loader-ready-checkpoint", "after", Some(trace), frames);
}

fn poll_slot5_controller_state(
    boundary: &str,
    phase: &str,
    trace: Option<CostumeObjectUpdateTrace>,
    frames: &[usize],
) {
    let Some(trace) = trace else {
        return;
    };
    if trace.selected_slot != Some(LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32)
        || trace.selected_variant != current_law_extra_slot_probe_variant_id()
        || trace.selected_model_resource != Some(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || trace.selected_preview_mapped_resource
            != Some(LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID)
    {
        return;
    }

    let previous_controller =
        LAST_SLOT5_CONTROLLER_STATE.swap(trace.controller.unwrap_or_default(), Ordering::Relaxed);
    let previous_controller_loader = LAST_SLOT5_CONTROLLER_LOADER_STATE.swap(
        trace.controller_model_loader.unwrap_or_default(),
        Ordering::Relaxed,
    );
    let current_controller = trace.controller.unwrap_or_default();
    let current_controller_loader = trace.controller_model_loader.unwrap_or_default();
    let lost_controller = (previous_controller != 0 || previous_controller_loader != 0)
        && (current_controller == 0 || current_controller_loader == 0);
    if !lost_controller {
        return;
    }

    let index = SLOT5_CONTROLLER_STATE_CHANGE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_SLOT5_CONTROLLER_STATE_CHANGE_LOGS {
        return;
    }
    let (_, child8) = slot5_current_child8_status_text();
    let checkpoint_loader_object =
        SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_OBJECT.load(Ordering::Relaxed);
    log::write_line(format!(
        "slot5-controller-state-change before=present after=none boundary={boundary} phase={phase} previous_controller={} previous_controller_loader={} controller={} controller_loader={} state_object={} loader_object={} resource911_live=[{}] child8=[{}] mapped294={} selected_slot={} cached_layout={} context=[{}] frames={}",
        format_optional_address((previous_controller != 0).then_some(previous_controller)),
        format_optional_address(
            (previous_controller_loader != 0).then_some(previous_controller_loader)
        ),
        format_optional_address((current_controller != 0).then_some(current_controller)),
        format_optional_address((current_controller_loader != 0).then_some(current_controller_loader)),
        format_optional_address(trace.object),
        format_optional_address((checkpoint_loader_object != 0).then_some(checkpoint_loader_object)),
        format_preview_resource_911_live_state(),
        child8,
        format_optional_u32(trace.selected_preview_mapped_resource),
        format_optional_u32(trace.selected_slot),
        format_optional_u32(trace.cached_layout),
        format_costume_object_update_trace(trace),
        format_stack_frames(frames, 8),
    ));
}

fn slot5_checkpoint_snapshot_reason() -> &'static str {
    let request_state = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE.load(Ordering::Relaxed);
    let request_loader = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_LOADER.load(Ordering::Relaxed);
    let request_object = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_OBJECT.load(Ordering::Relaxed);
    let request_attach08 = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_ATTACH08.load(Ordering::Relaxed);
    let request_token10 = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_TOKEN10.load(Ordering::Relaxed);
    if request_state == 0 {
        return "state_mismatch";
    }
    if request_loader == 0 || request_object == 0 {
        return "loader_missing";
    }
    let Some(matched) = find_costume_object_model_ready_match(
        request_loader,
        u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
        u32::from(u16::MAX),
        0,
    ) else {
        return "loader_missing";
    };
    if matched.object != request_object {
        return "object_mismatch";
    }
    if matched.state28 != 7 {
        return "state_mismatch";
    }
    if matched.phase2c != 2 {
        return "phase_mismatch";
    }
    if matched.flags20 != 0x05 {
        return "flags_mismatch";
    }
    if matched.attach08 != request_attach08 {
        return "attach_mismatch";
    }
    if matched.token10 != request_token10 {
        return "token_mismatch";
    }
    "ok"
}

fn slot5_checkpoint_snapshot_valid() -> bool {
    slot5_checkpoint_snapshot_reason() == "ok"
}

fn slot5_saved_controller_loader_valid() -> bool {
    let controller = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.load(Ordering::Relaxed);
    let controller_loader =
        SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER_LOADER.load(Ordering::Relaxed);
    controller != 0
        && controller_loader != 0
        && read_usize_field(
            controller,
            COSTUME_OBJECT_UPDATE_CONTROLLER_MODEL_LOADER_OFFSET,
        ) == Some(controller_loader)
}

fn slot5_checkpoint_late_refresh_skip_reason(
    state: usize,
    trace: Option<CostumeObjectUpdateTrace>,
) -> Option<&'static str> {
    let request_seq = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_SEQ.load(Ordering::Relaxed);
    if request_seq == 0 {
        return Some("no_checkpoint");
    }
    if SLOT5_911_READY_CHECKPOINT_REFRESH_REPLAYED_SEQ.load(Ordering::Relaxed) == request_seq {
        return Some("already_replayed");
    }
    if !slot5_checkpoint_snapshot_valid() {
        return Some(slot5_checkpoint_snapshot_reason());
    }
    let request_state = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE.load(Ordering::Relaxed);
    if state == 0 || request_state == 0 || state != request_state {
        return Some("state_mismatch");
    }
    let Some(trace) = trace else {
        return Some("state_mismatch");
    };
    if trace.controller.is_none() || trace.controller_model_loader.is_none() {
        if !LAW_SLOT5_TEMP_RESTORE_CONTROLLER_FOR_LATE_REFRESH_ENABLED
            || !slot5_saved_controller_loader_valid()
        {
            return Some("controller_lost");
        }
    } else if trace.controller
        != Some(SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.load(Ordering::Relaxed))
        || trace.controller_model_loader
            != Some(
                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER_LOADER.load(Ordering::Relaxed),
            )
    {
        return Some("controller_lost");
    }
    if !preview_resource_match_is_active(read_tracked_preview_resource_states().resource_911) {
        return Some("resource911_not_active");
    }
    if !slot5_current_child8_status_text().0 {
        return Some("child8_not_good");
    }
    if COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.load(Ordering::Acquire) == 0 {
        return Some("no_original");
    }
    None
}

fn maybe_run_slot5_refresh_after_911_ready_with_checkpoint(
    boundary: &'static str,
    state: usize,
    trace: Option<CostumeObjectUpdateTrace>,
) {
    if !LAW_SLOT5_REFRESH_PREVIEW_AFTER_911_READY_WITH_CHECKPOINT_ENABLED {
        return;
    }
    let request_seq = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_SEQ.load(Ordering::Relaxed);
    if request_seq == 0 {
        return;
    }
    let skip_reason = slot5_checkpoint_late_refresh_skip_reason(state, trace);
    if let Some(reason) = skip_reason {
        if matches!(
            reason,
            "controller_lost"
                | "object_mismatch"
                | "loader_missing"
                | "state_mismatch"
                | "phase_mismatch"
                | "flags_mismatch"
                | "attach_mismatch"
                | "token_mismatch"
                | "no_candidate"
                | "already_replayed"
                | "no_original"
        ) {
            let index = SLOT5_911_READY_CHECKPOINT_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
            if index < MAX_SLOT5_911_READY_CHECKPOINT_REFRESH_LOGS {
                let frames = capture_stack_trace();
                log::write_line(format!(
                    "slot5-refresh-after-911-ready-skip reason={reason} boundary={boundary} checkpoint_seq={request_seq} controller_snapshot_valid={} snapshot_reason={} state={} request_state={} resource911_live=[{}] child8=[{}] context=[{}] frames={}",
                    slot5_checkpoint_snapshot_valid(),
                    slot5_checkpoint_snapshot_reason(),
                    format_optional_address((state != 0).then_some(state)),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE.load(Ordering::Relaxed)
                            )
                    ),
                    format_preview_resource_911_live_state(),
                    slot5_current_child8_status_text().1,
                    trace
                        .map(format_costume_object_update_trace)
                        .unwrap_or_else(|| "none".to_string()),
                    format_stack_frames(&frames, 8),
                ));
                log::write_line(format!(
                    "slot5-controller-snapshot-valid={} reason={} boundary={boundary} checkpoint_seq={request_seq} request_state={} request_state_object={} request_loader={} request_loader_object={} request_controller={} request_controller_loader={} request_attach08={} request_token10={}",
                    slot5_checkpoint_snapshot_valid(),
                    slot5_checkpoint_snapshot_reason(),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE.load(Ordering::Relaxed)
                            )
                    ),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE_OBJECT.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE_OBJECT.load(Ordering::Relaxed)
                            )
                    ),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_LOADER.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_LOADER.load(Ordering::Relaxed)
                            )
                    ),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_OBJECT.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_OBJECT.load(Ordering::Relaxed)
                            )
                    ),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.load(Ordering::Relaxed)
                            )
                    ),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER_LOADER.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER_LOADER.load(Ordering::Relaxed)
                            )
                    ),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_ATTACH08.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_ATTACH08.load(Ordering::Relaxed)
                            )
                    ),
                    format_optional_address(
                        (SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_TOKEN10.load(Ordering::Relaxed) != 0)
                            .then_some(
                                SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_TOKEN10.load(Ordering::Relaxed)
                            )
                    ),
                ));
            }
        }
        return;
    }
    if SLOT5_911_READY_CHECKPOINT_REFRESH_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let original_address = COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.load(Ordering::Acquire);
    let preview_arg = LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID;
    let frames = capture_stack_trace();
    let before = read_costume_object_refresh_preview_trace(state, preview_arg)
        .unwrap_or_else(|| "none".to_string());
    let before_resource = format_preview_resource_911_live_state();
    let candidate_seq_before = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let before_current_controller =
        read_usize_field(state, COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET).unwrap_or_default();
    let saved_controller =
        SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.load(Ordering::Relaxed);
    let use_temp_controller = LAW_SLOT5_TEMP_RESTORE_CONTROLLER_FOR_LATE_REFRESH_ENABLED
        && before_current_controller == 0
        && saved_controller != 0
        && slot5_saved_controller_loader_valid();
    if use_temp_controller
        && !write_usize_field_checked(
            state,
            COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET,
            saved_controller,
        )
    {
        let index = SLOT5_911_READY_CHECKPOINT_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
        if index < MAX_SLOT5_911_READY_CHECKPOINT_REFRESH_LOGS {
            log::write_line(format!(
                "slot5-temp-controller-refresh-skip reason=restore_write_failed boundary={boundary} checkpoint_seq={request_seq} state=0x{state:x} before_controller={} temp_controller={} snapshot_reason={} resource911_live=[{}] child8=[{}] frames={}",
                format_optional_address((before_current_controller != 0).then_some(before_current_controller)),
                format_optional_address(Some(saved_controller)),
                slot5_checkpoint_snapshot_reason(),
                format_preview_resource_911_live_state(),
                slot5_current_child8_status_text().1,
                format_stack_frames(&frames, 8),
            ));
        }
        SLOT5_911_READY_CHECKPOINT_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
        return;
    }
    let original: CostumeObjectRefreshPreviewFn = unsafe { std::mem::transmute(original_address) };
    unsafe { original(state, preview_arg) };
    let restored_back = if use_temp_controller {
        write_usize_field_checked(
            state,
            COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET,
            before_current_controller,
        )
    } else {
        true
    };
    let after_trace = read_costume_object_update_trace(state);
    let after = read_costume_object_refresh_preview_trace(state, preview_arg)
        .unwrap_or_else(|| "none".to_string());
    let after_resource = format_preview_resource_911_live_state();
    let candidate_seq_after = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let consumed = candidate_seq_after > candidate_seq_before;
    SLOT5_911_READY_CHECKPOINT_REFRESH_REPLAYED_SEQ.store(request_seq, Ordering::Relaxed);

    let index = SLOT5_911_READY_CHECKPOINT_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_911_READY_CHECKPOINT_REFRESH_LOGS {
        if use_temp_controller {
            log::write_line(format!(
                "slot5-temp-controller-refresh-run boundary={boundary} checkpoint_seq={request_seq} state=0x{state:x} preview_arg={preview_arg} before_controller={} temp_controller={} restored_back={} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} snapshot_reason={} saved_controller_loader_valid={} resource911_live=[{}] child8=[{}] frames={}",
                format_optional_address((before_current_controller != 0).then_some(before_current_controller)),
                format_optional_address(Some(saved_controller)),
                restored_back,
                slot5_checkpoint_snapshot_reason(),
                slot5_saved_controller_loader_valid(),
                format_preview_resource_911_live_state(),
                slot5_current_child8_status_text().1,
                format_stack_frames(&frames, 8),
            ));
        }
        log::write_line(format!(
            "slot5-refresh-after-911-ready-run boundary={boundary} checkpoint_seq={request_seq} state=0x{state:x} preview_arg={preview_arg} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} controller_snapshot_valid={} snapshot_reason={} used_temp_controller={} restored_back={} before=[{before}] after=[{after}] before_resource911=[{before_resource}] after_resource911=[{after_resource}] before_context=[{}] after_context=[{}] frames={}",
            slot5_checkpoint_snapshot_valid(),
            slot5_checkpoint_snapshot_reason(),
            use_temp_controller,
            restored_back,
            trace
                .map(format_costume_object_update_trace)
                .unwrap_or_else(|| "none".to_string()),
            after_trace
                .map(format_costume_object_update_trace)
                .unwrap_or_else(|| "none".to_string()),
            format_stack_frames(&frames, 8),
        ));
        let line = if consumed {
            "slot5-ready-checkpoint-consumed"
        } else {
            "slot5-ready-checkpoint-not-consumed"
        };
        let reason = if consumed {
            "candidate_update"
        } else {
            "no_candidate"
        };
        log::write_line(format!(
            "{line} reason={reason} source=after-911-ready-with-checkpoint boundary={boundary} checkpoint_seq={request_seq} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} resource911_live=[{}] child8=[{}] context=[{}] frames={}",
            format_preview_resource_911_live_state(),
            slot5_current_child8_status_text().1,
            after_trace
                .map(format_costume_object_update_trace)
                .unwrap_or_else(|| "none".to_string()),
            format_stack_frames(&frames, 8),
        ));
    }

    SLOT5_911_READY_CHECKPOINT_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
}

fn slot5_saved_controller_vtable() -> Option<usize> {
    let controller = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.load(Ordering::Relaxed);
    if controller == 0 {
        return None;
    }
    read_usize_absolute(controller)
        .filter(|vtable| preview_result_pointer_kind(Some(*vtable)) == "pointer")
}

fn slot5_saved_controller_loader_current() -> Option<usize> {
    let controller = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.load(Ordering::Relaxed);
    if controller == 0 {
        return None;
    }
    read_usize_field(controller, COSTUME_OBJECT_UPDATE_CONTROLLER_MODEL_LOADER_OFFSET)
}

fn slot5_widget_ready_temp_controller_skip_reason(
    request_seq: usize,
    state: usize,
    widget: usize,
    before_current_controller: usize,
    same_state_ptr: bool,
    same_widget_ptr: bool,
    snapshot_reason: &'static str,
) -> Option<&'static str> {
    if !LAW_SLOT5_TEMP_RESTORE_CONTROLLER_AT_WIDGET_READY_ENABLED {
        return Some("disabled");
    }
    if request_seq == 0 {
        return Some("no_checkpoint");
    }
    if SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_REPLAYED_SEQ.load(Ordering::Relaxed)
        == request_seq
    {
        return Some("already_replayed");
    }
    if state == 0 || !same_state_ptr {
        return Some("state_mismatch");
    }
    if widget == 0 || !same_widget_ptr {
        return Some("widget_mismatch");
    }
    if !matches!(snapshot_reason, "ok" | "loader_missing") {
        return Some(snapshot_reason);
    }
    if !preview_resource_match_is_active(read_tracked_preview_resource_states().resource_911) {
        return Some("resource911_not_active");
    }
    if !slot5_current_child8_status_text().0 {
        return Some("child8_not_good");
    }
    if before_current_controller != 0 {
        return Some("controller_present");
    }
    let saved_controller_loader =
        SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER_LOADER.load(Ordering::Relaxed);
    if saved_controller_loader == 0
        || slot5_saved_controller_vtable().is_none()
        || slot5_saved_controller_loader_current().is_none()
    {
        return Some("invalid_saved_controller");
    }
    if COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.load(Ordering::Acquire) == 0 {
        return Some("no_original");
    }
    None
}

fn maybe_run_slot5_widget_ready_temp_controller_refresh(
    widget: usize,
    phase: &str,
    context: &str,
    frames: &[usize],
) {
    if !LAW_SLOT5_TEMP_RESTORE_CONTROLLER_AT_WIDGET_READY_ENABLED {
        return;
    }

    let request_seq = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_SEQ.load(Ordering::Relaxed);
    let state = LAST_COSTUME_OBJECT_UPDATE_STATE.load(Ordering::Relaxed);
    let request_state = SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_STATE.load(Ordering::Relaxed);
    let stored_widget = LAST_SLOT5_PREVIEW_WIDGET.load(Ordering::Relaxed);
    let same_state_ptr = state != 0 && state == request_state;
    let same_widget_ptr = widget != 0 && widget == stored_widget;
    let snapshot_reason = slot5_checkpoint_snapshot_reason();
    let saved_controller =
        SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER.load(Ordering::Relaxed);
    let saved_controller_loader =
        SLOT5_READY_CHECKPOINT_REFRESH_REQUEST_CONTROLLER_LOADER.load(Ordering::Relaxed);
    let saved_controller_vtable = slot5_saved_controller_vtable();
    let saved_controller_loader_current = slot5_saved_controller_loader_current();
    let saved_controller_loader_link_valid =
        saved_controller_loader_current == Some(saved_controller_loader);
    let before_current_controller =
        read_usize_field(state, COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET).unwrap_or_default();
    let child8 = slot5_current_child8_status_text().1;
    let skip_reason = slot5_widget_ready_temp_controller_skip_reason(
        request_seq,
        state,
        widget,
        before_current_controller,
        same_state_ptr,
        same_widget_ptr,
        snapshot_reason,
    );

    if let Some(reason) = skip_reason {
        if !matches!(reason, "disabled" | "already_replayed") {
            let index =
                SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
            if index < MAX_SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS {
                log::write_line(format!(
                    "slot5-widget-ready-controller-restore-skip reason={reason} phase={phase} checkpoint_seq={request_seq} state={} request_state={} widget={} stored_widget={} same_state_ptr={same_state_ptr} same_widget_ptr={same_widget_ptr} saved_controller={} saved_controller_loader_expected={} saved_controller_loader_current={} saved_controller_vtable={} saved_controller_loader_link_valid={} current_controller_before={} snapshot_reason={snapshot_reason} allowed_for_diagnostic=false resource911_live=[{}] child8=[{}] context=[{context}] frames={}",
                    format_optional_address((state != 0).then_some(state)),
                    format_optional_address((request_state != 0).then_some(request_state)),
                    format_optional_address((widget != 0).then_some(widget)),
                    format_optional_address((stored_widget != 0).then_some(stored_widget)),
                    format_optional_address((saved_controller != 0).then_some(saved_controller)),
                    format_optional_address(
                        (saved_controller_loader != 0).then_some(saved_controller_loader)
                    ),
                    format_optional_address(saved_controller_loader_current),
                    format_optional_address(saved_controller_vtable),
                    saved_controller_loader_link_valid,
                    format_optional_address(
                        (before_current_controller != 0).then_some(before_current_controller)
                    ),
                    format_preview_resource_911_live_state(),
                    child8,
                    format_stack_frames(frames, 8),
                ));
            }
        }
        return;
    }

    if SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_IN_PROGRESS.swap(true, Ordering::AcqRel) {
        return;
    }

    let allowed_for_diagnostic = snapshot_reason == "loader_missing";
    let restore_loader_link = LAW_SLOT5_TEMP_RESTORE_CONTROLLER_LOADER_LINK_AT_WIDGET_READY_ENABLED
        && saved_controller_loader_current != Some(saved_controller_loader);
    let index = SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS {
        log::write_line(format!(
            "slot5-widget-ready-controller-restore-check phase={phase} checkpoint_seq={request_seq} state=0x{state:x} request_state={} widget=0x{widget:x} stored_widget={} same_state_ptr={same_state_ptr} same_widget_ptr={same_widget_ptr} saved_controller={} saved_controller_loader_expected={} saved_controller_loader_current={} saved_controller_vtable={} saved_controller_loader_link_valid={} restore_loader_link={} current_controller_before={} snapshot_reason={snapshot_reason} allowed_for_diagnostic={allowed_for_diagnostic} resource911_live=[{}] child8=[{}] context=[{context}] frames={}",
            format_optional_address((request_state != 0).then_some(request_state)),
            format_optional_address((stored_widget != 0).then_some(stored_widget)),
            format_optional_address(Some(saved_controller)),
            format_optional_address(Some(saved_controller_loader)),
            format_optional_address(saved_controller_loader_current),
            format_optional_address(saved_controller_vtable),
            saved_controller_loader_link_valid,
            restore_loader_link,
            format_optional_address(
                (before_current_controller != 0).then_some(before_current_controller)
            ),
            format_preview_resource_911_live_state(),
            child8,
            format_stack_frames(frames, 8),
        ));
    }

    if !write_usize_field_checked(
        state,
        COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET,
        saved_controller,
    ) {
        let index = SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
        if index < MAX_SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS {
            log::write_line(format!(
                "slot5-widget-ready-controller-restore-skip reason=restore_write_failed phase={phase} checkpoint_seq={request_seq} state=0x{state:x} temp_controller={} snapshot_reason={snapshot_reason} resource911_live=[{}] child8=[{}] frames={}",
                format_optional_address(Some(saved_controller)),
                format_preview_resource_911_live_state(),
                slot5_current_child8_status_text().1,
                format_stack_frames(frames, 8),
            ));
        }
        SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
        return;
    }

    let loader_link_written = if restore_loader_link {
        write_usize_field_checked(
            saved_controller,
            COSTUME_OBJECT_UPDATE_CONTROLLER_MODEL_LOADER_OFFSET,
            saved_controller_loader,
        )
    } else {
        true
    };
    if !loader_link_written {
        let restored_controller = write_usize_field_checked(
            state,
            COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET,
            before_current_controller,
        );
        let index = SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
        if index < MAX_SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS {
            log::write_line(format!(
                "slot5-widget-ready-controller-restore-skip reason=loader_link_write_failed phase={phase} checkpoint_seq={request_seq} state=0x{state:x} temp_controller={} saved_controller_loader_expected={} saved_controller_loader_current={} restored_controller={} snapshot_reason={snapshot_reason} resource911_live=[{}] child8=[{}] frames={}",
                format_optional_address(Some(saved_controller)),
                format_optional_address(Some(saved_controller_loader)),
                format_optional_address(saved_controller_loader_current),
                restored_controller,
                format_preview_resource_911_live_state(),
                slot5_current_child8_status_text().1,
                format_stack_frames(frames, 8),
            ));
        }
        SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
        return;
    }

    let preview_arg = LAW_EXTRA_SLOT_PREVIEW_TABLE_TARGET_ID;
    let before = read_costume_object_refresh_preview_trace(state, preview_arg)
        .unwrap_or_else(|| "none".to_string());
    let candidate_seq_before = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let original_address = COSTUME_OBJECT_REFRESH_PREVIEW_CALLSITE_ORIGINAL.load(Ordering::Acquire);
    let original: CostumeObjectRefreshPreviewFn = unsafe { std::mem::transmute(original_address) };
    unsafe { original(state, preview_arg) };
    let restored_loader_link = if restore_loader_link {
        write_usize_field_checked(
            saved_controller,
            COSTUME_OBJECT_UPDATE_CONTROLLER_MODEL_LOADER_OFFSET,
            saved_controller_loader_current.unwrap_or_default(),
        )
    } else {
        true
    };
    let restored_controller = write_usize_field_checked(
        state,
        COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET,
        before_current_controller,
    );
    let current_controller_after =
        read_usize_field(state, COSTUME_OBJECT_UPDATE_CONTROLLER_OFFSET).unwrap_or_default();
    let saved_controller_loader_after = slot5_saved_controller_loader_current();
    let after = read_costume_object_refresh_preview_trace(state, preview_arg)
        .unwrap_or_else(|| "none".to_string());
    let candidate_seq_after = SLOT5_RENDER_CANDIDATE_SEQ.load(Ordering::Relaxed);
    let consumed = candidate_seq_after > candidate_seq_before;
    SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_REPLAYED_SEQ.store(request_seq, Ordering::Relaxed);

    let index = SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < MAX_SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_LOGS {
        log::write_line(format!(
            "law-slot5-widget-ready-temp-controller-loader-link-refresh-run phase={phase} checkpoint_seq={request_seq} state=0x{state:x} preview_arg={preview_arg} before_controller={} temp_controller={} current_controller_after={} saved_controller_loader_before={} saved_controller_loader_expected={} saved_controller_loader_after={} restored_controller={restored_controller} restored_loader_link={restored_loader_link} same_state_ptr={same_state_ptr} same_widget_ptr={same_widget_ptr} saved_controller_vtable={} snapshot_reason={snapshot_reason} allowed_for_diagnostic={allowed_for_diagnostic} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} before=[{before}] after=[{after}] resource911_live=[{}] child8=[{}] context=[{context}] frames={}",
            format_optional_address(
                (before_current_controller != 0).then_some(before_current_controller)
            ),
            format_optional_address(Some(saved_controller)),
            format_optional_address((current_controller_after != 0).then_some(current_controller_after)),
            format_optional_address(saved_controller_loader_current),
            format_optional_address(Some(saved_controller_loader)),
            format_optional_address(saved_controller_loader_after),
            format_optional_address(saved_controller_vtable),
            format_preview_resource_911_live_state(),
            slot5_current_child8_status_text().1,
            format_stack_frames(frames, 8),
        ));
        let line = if consumed {
            "slot5-widget-ready-temp-controller-consumed"
        } else {
            "slot5-widget-ready-temp-controller-not-consumed"
        };
        let reason = if consumed { "candidate_update" } else { "no_candidate" };
        log::write_line(format!(
            "{line} reason={reason} phase={phase} checkpoint_seq={request_seq} candidate_seq_before={candidate_seq_before} candidate_seq_after={candidate_seq_after} restored_controller={restored_controller} restored_loader_link={restored_loader_link} current_controller_after={} saved_controller_loader_after={} resource911_live=[{}] child8=[{}] context=[{context}] frames={}",
            format_optional_address((current_controller_after != 0).then_some(current_controller_after)),
            format_optional_address(saved_controller_loader_after),
            format_preview_resource_911_live_state(),
            slot5_current_child8_status_text().1,
            format_stack_frames(frames, 8),
        ));
    }

    SLOT5_WIDGET_READY_TEMP_CONTROLLER_REFRESH_IN_PROGRESS.store(false, Ordering::Release);
}

fn format_loader_object_final_state(
    label: &str,
    loader: usize,
    object: usize,
    model: u32,
) -> String {
    let status_manager =
        read_game_global_pointer(MODEL_RESOURCE_STATUS_MANAGER_RVA).unwrap_or_default();
    let wait3c4 = read_u32_field(object, 0x3c4);
    let wait3c8 = read_u32_field(object, 0x3c8);
    let matched = if loader != 0 {
        find_costume_object_model_ready_match(loader, model, u32::from(u16::MAX), 0)
            .map(|matched| matched.object == object)
    } else {
        None
    };
    format!(
        "label={label} loader={} object={} expected_model={model} matched_loader_object={} detail=[{}] compare_ids={}/{}/{} wait3c4_status=[{}] wait3c8_status=[{}] object_children=[child08={} child10={} child58={} child78={} childb0={}]",
        format_optional_address((loader != 0).then_some(loader)),
        format_optional_address((object != 0).then_some(object)),
        matched
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        if object != 0 {
            format_model_ready_object_detail(object)
        } else {
            "none".to_string()
        },
        format_optional_u32(read_u32_field(object, 0x378)),
        format_optional_u32(read_u32_field(object, 0x37c)),
        format_optional_u32(read_u32_field(object, 0x384)),
        wait3c4
            .map(|id| format_model_resource_manager_entry(status_manager, id))
            .unwrap_or_else(|| "none".to_string()),
        wait3c8
            .map(|id| format_model_resource_manager_entry(status_manager, id))
            .unwrap_or_else(|| "none".to_string()),
        format_optional_address(read_usize_field(object, 0x08).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(object, 0x10).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(object, 0x58).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(object, 0x78).filter(|address| *address != 0)),
        format_optional_address(read_usize_field(object, 0xb0).filter(|address| *address != 0)),
    )
}

fn format_model_manager_alias_state(label: &str, target: u32, source: Option<u32>) -> String {
    let manager = read_game_global_pointer(MODEL_RESOURCE_MANAGER_RVA).unwrap_or_default();
    let target_ptr = model_resource_manager_entry_address(manager, target)
        .and_then(|address| read_usize_field(address, MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET))
        .unwrap_or_default();
    let target_state = model_resource_manager_entry_address(manager, target)
        .and_then(|address| read_u32_field(address, MODEL_RESOURCE_MANAGER_ENTRY_STATE_OFFSET));
    let (source_text, source_ptr, source_state) = source
        .map(|source| {
            let ptr = model_resource_manager_entry_address(manager, source)
                .and_then(|address| {
                    read_usize_field(address, MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET)
                })
                .unwrap_or_default();
            let state = model_resource_manager_entry_address(manager, source).and_then(|address| {
                read_u32_field(address, MODEL_RESOURCE_MANAGER_ENTRY_STATE_OFFSET)
            });
            (
                format_model_resource_manager_entry(manager, source),
                ptr,
                state,
            )
        })
        .unwrap_or_else(|| ("none".to_string(), 0, None));
    let alias_ptr_equals_source =
        source.is_some() && target_ptr != 0 && source_ptr != 0 && target_ptr == source_ptr;
    format!(
        "label={label} manager={} target=[{}] source=[{}] target_ptr={} source_ptr={} target_state={} source_state={} alias_ptr_equals_source={} ram_clone_done={}",
        format_optional_address((manager != 0).then_some(manager)),
        format_model_resource_manager_entry(manager, target),
        source_text,
        format_optional_address((target_ptr != 0).then_some(target_ptr)),
        format_optional_address((source_ptr != 0).then_some(source_ptr)),
        format_optional_u32(target_state),
        format_optional_u32(source_state),
        alias_ptr_equals_source,
        LAW_PRIVATE_MODEL_RAM_CLONE_DONE.load(Ordering::Relaxed)
    )
}

fn loader_object_activation_diff_reason(oni_object: usize, slot5_object: usize) -> &'static str {
    if slot5_object == 0 {
        return "slot5_loader_missing";
    }
    if oni_object == 0 {
        return "oni_baseline_missing";
    }
    let oni_state28 = read_u32_field(oni_object, 0x28);
    let slot5_state28 = read_u32_field(slot5_object, 0x28);
    let oni_phase2c = read_u32_field(oni_object, 0x2c);
    let slot5_phase2c = read_u32_field(slot5_object, 0x2c);
    let oni_flags20 = read_u8_field(oni_object, 0x20).unwrap_or_default();
    let slot5_flags20 = read_u8_field(slot5_object, 0x20).unwrap_or_default();
    let oni_attach = read_usize_field(oni_object, 0x08).unwrap_or_default();
    let slot5_attach = read_usize_field(slot5_object, 0x08).unwrap_or_default();
    let manager = read_game_global_pointer(MODEL_RESOURCE_MANAGER_RVA).unwrap_or_default();
    let slot5_target = model_resource_manager_entry_address(
        manager,
        u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
    )
    .and_then(|address| read_usize_field(address, MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET))
    .unwrap_or_default();
    let slot5_source = model_resource_manager_entry_address(
        manager,
        u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID),
    )
    .and_then(|address| read_usize_field(address, MODEL_RESOURCE_MANAGER_ENTRY_POINTER_OFFSET))
    .unwrap_or_default();

    if (slot5_flags20 & 0x01) == 0 && (oni_flags20 & 0x01) != 0 {
        "apply_ready_missing"
    } else if oni_state28 != slot5_state28 || oni_phase2c != slot5_phase2c {
        "loader_state_diff"
    } else if oni_attach != 0 && slot5_attach == 0 {
        "model_binding_missing"
    } else if slot5_target != 0 && slot5_target == slot5_source {
        "alias_only"
    } else {
        "no_diff_found"
    }
}

fn log_loader_object_final_compare(source: &str, boundary: &str, frames: &[usize]) {
    let index = LOADER_OBJECT_FINAL_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_LOADER_OBJECT_FINAL_LOGS {
        return;
    }

    let oni_loader = LAST_ONI_MODEL_READY_LOADER.load(Ordering::Relaxed);
    let oni_object = LAST_ONI_MODEL_READY_OBJECT.load(Ordering::Relaxed);
    let slot5_loader = LAST_SLOT5_MODEL_READY_LOADER.load(Ordering::Relaxed);
    let slot5_object = LAST_SLOT5_MODEL_READY_OBJECT.load(Ordering::Relaxed);
    let oni = format_loader_object_final_state("oni", oni_loader, oni_object, 308);
    let slot5 = format_loader_object_final_state(
        "slot5",
        slot5_loader,
        slot5_object,
        u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
    );

    log::write_line(format!(
        "oni-late-loader-object-snapshot source={source} boundary={boundary} seq={} {oni} frames={}",
        LAST_ONI_MODEL_READY_SEQ.load(Ordering::Relaxed),
        format_stack_frames(frames, 8)
    ));
    log::write_line(format!(
        "slot5-late-loader-object-snapshot source={source} boundary={boundary} seq={} {slot5} frames={}",
        LAST_SLOT5_MODEL_READY_SEQ.load(Ordering::Relaxed),
        format_stack_frames(frames, 8)
    ));

    let alias_index = MODEL_MANAGER_ALIAS_STATE_LOGS.fetch_add(1, Ordering::Relaxed);
    if alias_index < MAX_MODEL_MANAGER_ALIAS_STATE_LOGS {
        log::write_line(format!(
            "model-manager-alias-state {} frames={}",
            format_model_manager_alias_state(
                "slot5",
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)),
            ),
            format_stack_frames(frames, 8)
        ));
        log::write_line(format!(
            "model-manager-alias-state {} frames={}",
            format_model_manager_alias_state("oni", 308, None),
            format_stack_frames(frames, 8)
        ));
    }

    let diff_index = RENDER_OBJECT_ACTIVATION_DIFF_LOGS.fetch_add(1, Ordering::Relaxed);
    if diff_index < MAX_RENDER_OBJECT_ACTIVATION_DIFF_LOGS {
        let reason = loader_object_activation_diff_reason(oni_object, slot5_object);
        log::write_line(format!(
            "slot5-render-object-activation-diff source={source} boundary={boundary} reason={reason} oni=[{}] slot5=[{}] slot5_alias=[{}] resource911_live=[{}] frames={}",
            format_loader_object_final_state("oni", oni_loader, oni_object, 308),
            format_loader_object_final_state(
                "slot5",
                slot5_loader,
                slot5_object,
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
            ),
            format_model_manager_alias_state(
                "slot5",
                u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID),
                Some(u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)),
            ),
            format_preview_resource_911_live_state(),
            format_stack_frames(frames, 8)
        ));
    }
}

fn format_model_ready_hook_detail(
    patch_detail: Option<String>,
    override_detail: Option<String>,
) -> Option<String> {
    match (patch_detail, override_detail) {
        (Some(patch), Some(override_detail)) => Some(format!("{patch};{override_detail}")),
        (Some(patch), None) => Some(patch),
        (None, Some(override_detail)) => Some(override_detail),
        (None, None) => None,
    }
}

fn format_costume_object_model_ready_loader(
    loader: usize,
    model_resource: u32,
    arg1: u32,
    arg2: u32,
) -> String {
    if loader == 0 {
        return "loader=0x0".to_string();
    }

    let root = read_usize_field(loader, 0x78).filter(|address| *address != 0);
    let sentinel = root
        .and_then(|address| read_usize_field(address, 0x18))
        .filter(|address| *address != 0);
    let first = sentinel.and_then(read_usize_absolute);
    let mut entries = Vec::new();
    let mut node = first.unwrap_or(0);
    let mut count = 0usize;
    let mut matches = 0usize;

    if let Some(sentinel) = sentinel {
        while node != 0 && node != sentinel && count < 24 {
            let object = read_usize_field(node, 0x10).filter(|address| *address != 0);
            let object_model = object.and_then(|address| read_u32_field(address, 0x378));
            let object_arg1 = object.and_then(|address| read_u32_field(address, 0x37c));
            let object_arg2 = object.and_then(|address| read_u32_field(address, 0x384));
            let is_match = object_model == Some(model_resource)
                && object_arg1 == Some(arg1)
                && object_arg2 == Some(arg2);
            if is_match {
                matches += 1;
            }
            if is_match || object_model.is_some_and(is_interesting_model_ready_resource) {
                let detail = object
                    .map(format_model_ready_object_detail)
                    .unwrap_or_else(|| "none".to_string());
                entries.push(format!(
                    "{}:node=0x{node:x},obj={},ids={}/{}/{},detail=[{}]",
                    count,
                    format_optional_address(object),
                    format_optional_u32(object_model),
                    format_optional_u32(object_arg1),
                    format_optional_u32(object_arg2),
                    detail,
                ));
            }
            let Some(next) = read_usize_absolute(node) else {
                break;
            };
            node = next;
            count += 1;
        }
    }

    format!(
        "loader=0x{loader:x} root={} sentinel={} first={} scanned={} matches={} entries=[{}]",
        format_optional_address(root),
        format_optional_address(sentinel),
        format_optional_address(first),
        count,
        matches,
        if entries.is_empty() {
            "none".to_string()
        } else {
            entries.join(";")
        }
    )
}

fn is_interesting_model_ready_resource(value: u32) -> bool {
    matches!(value, 26 | 227 | 272 | 292 | 308)
        || value == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || value == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID)
}

fn log_costume_object_model_ready_check(
    loader: usize,
    model_resource: u32,
    arg1: u32,
    arg2: u32,
    original_result: u64,
    result: u64,
    override_detail: Option<String>,
    before: Option<String>,
    after: String,
) {
    let index = COSTUME_OBJECT_MODEL_READY_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_OBJECT_MODEL_READY_TRACE_LOGS {
        return;
    }
    let before = before.unwrap_or_else(|| "none".to_string());
    let override_detail = override_detail.unwrap_or_else(|| "forced=false".to_string());
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume object busy/pending check loader=0x{loader:x} args={model_resource}/{arg1}/{arg2} original_result={original_result} result={result} semantic=busy_or_pending override=[{override_detail}] before=[{before}] after=[{after}] frames={}",
        format_stack_frames(&frames, 8)
    ));
}

fn log_law_ready_timeline_model_ready_check(
    loader: usize,
    model_resource: u32,
    arg1: u32,
    arg2: u32,
    original_result: u64,
    result: u64,
) {
    let slot_label = match (model_resource, arg1, arg2) {
        (308, color, 0) if color == u32::from(u16::MAX) => "slot3-oni",
        (model, color, 0)
            if model == u32::from(LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
                && color == u32::from(u16::MAX) =>
        {
            "slot5-custom"
        }
        _ => return,
    };
    let matched = find_costume_object_model_ready_match(loader, model_resource, arg1, arg2);
    record_loader_object_match(slot_label, loader, matched);
    let trace = current_law_ready_timeline_trace_for_label(slot_label);
    if let Some(matched) = matched {
        let frames = capture_stack_trace();
        store_loader_ready_checkpoint(slot_label, loader, matched, trace, &frames);
    }
    let matched = matched
        .map(format_model_ready_match_detail)
        .unwrap_or_else(|| "match=none".to_string());
    let frames = capture_stack_trace();
    log_law_ready_timeline(
        "busy-check",
        format!(
            "slot_label={slot_label} loader=0x{loader:x} args={model_resource}/{arg1}/{arg2} original_result={original_result} result={result} semantic=busy_or_pending match=[{matched}] context=[{}] frames={}",
            trace
                .map(format_costume_object_update_trace)
                .unwrap_or_else(|| {
                    format_last_costume_object_update_context(
                        LAST_COSTUME_OBJECT_UPDATE_OBJECT.load(Ordering::Relaxed)
                    )
                }),
            format_stack_frames(&frames, 6)
        ),
    );
}

fn is_interesting_costume_object_update_trace(trace: CostumeObjectUpdateTrace) -> bool {
    trace.selected_layout.is_some_and(is_law_menu_value)
        || trace
            .selected_variant
            .is_some_and(|value| is_law_menu_value(value as u32))
        || trace
            .selected_model_resource
            .is_some_and(|value| value == LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID)
        || trace.selected_slot.is_some_and(|slot| slot >= 3)
        || trace.cached_layout.is_some_and(is_law_menu_value)
}

fn format_costume_object_update_trace(trace: CostumeObjectUpdateTrace) -> String {
    format!(
        "count={} selected={} mode={} cached_layout={} flags44c={} flags44d={} object={} object_child={} object_pending={} controller={} controller_loader={} selected_layout={} selected_slot={} selected_kind={} load_args={}/{}/{} selected_variant={} selected_model_resource={} selected_preview_mapping={} selected_preview_mapped={}",
        format_optional_u32(trace.count),
        format_optional_u32(trace.selected_index),
        format_optional_u32(trace.mode),
        format_optional_u32(trace.cached_layout),
        format_optional_u8(trace.refresh_flag),
        format_optional_u8(trace.locked_flag),
        format_optional_address(trace.object),
        format_optional_address(trace.object_child),
        format_optional_u8(trace.object_pending),
        format_optional_address(trace.controller),
        format_optional_address(trace.controller_model_loader),
        format_optional_u32(trace.selected_layout),
        format_optional_u32(trace.selected_slot),
        format_optional_u32(trace.selected_kind),
        format_optional_u32(trace.selected_load_arg0),
        format_optional_u32(trace.selected_load_arg1),
        format_optional_u32(trace.selected_load_arg2),
        format_optional_u16_decimal(trace.selected_variant),
        format_optional_u16_decimal(trace.selected_model_resource),
        format_optional_u16_decimal(trace.selected_preview_mapping),
        format_optional_u32(trace.selected_preview_mapped_resource),
    )
}

fn format_costume_object_update_entries(
    state: usize,
    trace: Option<CostumeObjectUpdateTrace>,
) -> String {
    let Some(trace) = trace else {
        return "none".to_string();
    };
    let Some(count) = trace.count else {
        return "count=none".to_string();
    };
    let entry_count = count.min(6);
    (0..entry_count)
        .map(|index| {
            let layout = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_LAYOUTS_OFFSET,
                index,
            );
            let slot = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_SLOTS_OFFSET,
                index,
            );
            let kind = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_KINDS_OFFSET,
                index,
            );
            let load_arg0 = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_LOAD_ARG0_OFFSET,
                index,
            );
            let load_arg1 = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_LOAD_ARG1_OFFSET,
                index,
            );
            let load_arg2 = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_LOAD_ARG2_OFFSET,
                index,
            );
            let variant = layout
                .zip(slot)
                .and_then(|(layout, slot)| read_layout_slot_variant(layout, slot));
            let model = variant.and_then(read_variant_model_resource);
            let preview_mapping = variant.and_then(read_variant_preview_mapping);
            let preview_mapped = preview_mapping.map(preview_model_mapped_id_from_metadata_value);
            format!(
                "{}:layout={} slot={} kind={} args={}/{}/{} variant={} model={} preview_mapping={} preview_mapped={}",
                index,
                format_optional_u32(layout),
                format_optional_u32(slot),
                format_optional_u32(kind),
                format_optional_u32(load_arg0),
                format_optional_u32(load_arg1),
                format_optional_u32(load_arg2),
                format_optional_u16_decimal(variant),
                format_optional_u16_decimal(model),
                format_optional_u16_decimal(preview_mapping),
                format_optional_u32(preview_mapped),
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn format_costume_object_update_entries_fixed(state: usize, entry_count: u32) -> String {
    (0..entry_count)
        .map(|index| {
            let layout = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_LAYOUTS_OFFSET,
                index,
            );
            let slot = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_SLOTS_OFFSET,
                index,
            );
            let kind = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_KINDS_OFFSET,
                index,
            );
            let load_arg0 = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_LOAD_ARG0_OFFSET,
                index,
            );
            let load_arg1 = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_LOAD_ARG1_OFFSET,
                index,
            );
            let load_arg2 = read_costume_object_update_array_value(
                state,
                COSTUME_OBJECT_UPDATE_LOAD_ARG2_OFFSET,
                index,
            );
            let variant = layout
                .zip(slot)
                .and_then(|(layout, slot)| read_layout_slot_variant(layout, slot));
            let model = variant.and_then(read_variant_model_resource);
            let preview_mapping = variant.and_then(read_variant_preview_mapping);
            let preview_mapped = preview_mapping.map(preview_model_mapped_id_from_metadata_value);
            format!(
                "{}:layout={} slot={} kind={} args={}/{}/{} variant={} model={} preview_mapping={} preview_mapped={}",
                index,
                format_optional_u32(layout),
                format_optional_u32(slot),
                format_optional_u32(kind),
                format_optional_u32(load_arg0),
                format_optional_u32(load_arg1),
                format_optional_u32(load_arg2),
                format_optional_u16_decimal(variant),
                format_optional_u16_decimal(model),
                format_optional_u16_decimal(preview_mapping),
                format_optional_u32(preview_mapped),
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn format_law_variant_metadata_summary() -> String {
    [57, 58, 555, 586, LAW_CUSTOM_LAYOUT_ID]
        .iter()
        .map(|variant| {
            let model = read_variant_model_resource(*variant);
            let preview_mapping = read_variant_preview_mapping(*variant);
            let preview_mapped = preview_mapping.map(preview_model_mapped_id_from_metadata_value);
            let flags = read_variant_flags(*variant);
            format!(
                "{}:model={} preview_mapping={} preview_mapped={} flags={}",
                variant,
                format_optional_u16_decimal(model),
                format_optional_u16_decimal(preview_mapping),
                format_optional_u32(preview_mapped),
                format_optional_u8(flags)
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn format_law_variant699_metadata_source_diff() -> String {
    let file_raw = law_linkdata_variant699_metadata_raw();
    let ram_raw = read_variant_metadata_raw(LAW_CUSTOM_LAYOUT_ID);
    let ram_address = static_layouts_address()
        .and_then(|base| costume_variant_metadata_record_address(base, LAW_CUSTOM_LAYOUT_ID));
    let pattern_hits = file_raw
        .map(format_law_variant_metadata_pattern_hits)
        .unwrap_or_else(|| "file_raw_missing".to_string());
    let ram_candidates = static_layouts_address()
        .map(find_law_variant_metadata_ram_candidates)
        .map(|candidates| format_law_variant_metadata_ram_candidates(&candidates))
        .unwrap_or_else(|| "static_layouts_missing".to_string());

    format!(
        "file_raw={} ram_addr={} ram_raw={} match={} pattern_hits={} ram_candidates=[{}]",
        file_raw.map(format_bytes).unwrap_or_else(|| "missing".to_string()),
        format_optional_address(ram_address),
        ram_raw
            .as_ref()
            .map(|bytes| format_bytes(bytes))
            .unwrap_or_else(|| "read_failed".to_string()),
        file_raw
            .zip(ram_raw.as_ref().map(|bytes| bytes.as_slice()))
            .is_some_and(|(file, ram)| file == ram),
        pattern_hits,
        ram_candidates
    )
}

fn format_law_variant_metadata_pattern_hits(pattern: &[u8]) -> String {
    if pattern.len() != COSTUME_VARIANT_METADATA_COPY_SIZE {
        return format!("bad_pattern_len={}", pattern.len());
    }

    let Some(static_layouts) = static_layouts_address() else {
        return "static_layouts_missing".to_string();
    };
    let start = static_layouts + COSTUME_VARIANT_METADATA_BASE_OFFSET;
    let byte_count =
        usize::from(LAW_EXTRA_SLOT_SELECTABLE_VARIANT_LIMIT_EXCLUSIVE) * COSTUME_VARIANT_METADATA_STRIDE;
    let mut bytes = vec![0u8; byte_count];
    if !read_exact_process_memory(start, &mut bytes) {
        return format!("read_failed start=0x{start:x} size=0x{byte_count:x}");
    }

    let hits = bytes
        .windows(pattern.len())
        .enumerate()
        .filter(|(_, window)| *window == pattern)
        .take(4)
        .map(|(offset, _)| {
            let variant_guess = (offset % COSTUME_VARIANT_METADATA_STRIDE == 0)
                .then_some(offset / COSTUME_VARIANT_METADATA_STRIDE);
            format!(
                "addr=0x{:x}/variant={}",
                start + offset,
                variant_guess
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unaligned".to_string())
            )
        })
        .collect::<Vec<_>>();

    if hits.is_empty() {
        "none".to_string()
    } else {
        hits.join("|")
    }
}

fn read_game_global_pointer(rva: usize) -> Option<usize> {
    let main_module = win::main_module() as usize;
    if main_module == 0 {
        return None;
    }
    read_usize_absolute(main_module.checked_add(rva)?).filter(|value| *value != 0)
}

fn is_interesting_costume_scene_trace(trace: CostumeSceneTrace) -> bool {
    trace.layout.is_some_and(is_law_menu_value)
        || trace.row == Some(LAW_MENU_ROW_ID as u32)
        || trace
            .selected_variant
            .is_some_and(|value| is_law_menu_value(value as u32))
        || trace.selected_slot.is_some_and(|value| value >= 3)
        || trace.object_layout.is_some_and(is_law_menu_value)
        || trace.object_variant.is_some_and(is_law_menu_value)
        || trace.object_slot.is_some_and(|value| value >= 3)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CostumeSceneTraceLogScope {
    Regular,
    Important,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewModelUpdateTraceScope {
    Slot5,
    Global,
    Custom,
}

fn costume_scene_trace_has_current_custom(trace: CostumeSceneTrace) -> bool {
    let Some(variant) = current_law_extra_slot_probe_variant_id() else {
        return false;
    };
    let variant_u32 = u32::from(variant);
    let custom_slot = LAW_DUPLICATE_VARIANT_SLOT_INDEX as u32;
    trace.selected_variant == Some(variant)
        || trace.object_variant == Some(variant_u32)
        || trace.selected_slot == Some(custom_slot)
        || trace.object_slot == Some(custom_slot)
}

fn law_custom_scene_available_check_candidate(
    trace: CostumeSceneTrace,
    custom_variant: Option<u16>,
) -> bool {
    let Some(custom_variant) = custom_variant else {
        return false;
    };
    if trace.layout != Some(u32::from(LAW_MASTER_LAYOUT_ID)) {
        return false;
    }

    trace.selected_variant == Some(custom_variant)
        || trace.object_variant == Some(u32::from(custom_variant))
}

fn law_custom_scene_locked_flag_patch_candidate(
    trace: CostumeSceneTrace,
    custom_variant: Option<u16>,
) -> bool {
    if !trace.locked_flag.is_some_and(|flag| flag != 0) {
        return false;
    }

    law_custom_scene_available_check_candidate(trace, custom_variant)
}

fn log_law_custom_scene_locked_flag_patch(
    label: &str,
    state: usize,
    trace: CostumeSceneTrace,
    detail: String,
) {
    let index = COSTUME_SCENE_LOCKED_FLAG_PATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_SCENE_LOCKED_FLAG_PATCH_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Law custom scene locked-flag patch phase={label} state=0x{state:x} trace=[{}] {detail} frames={}",
        format_costume_scene_trace(trace),
        format_stack_frames(&frames, 6)
    ));
}

fn patch_law_custom_scene_locked_flag(
    label: &str,
    state: usize,
    trace: Option<CostumeSceneTrace>,
) -> Option<CostumeSceneTrace> {
    if !LAW_EXTRA_SLOT_CLEAR_SCENE_LOCKED_DIAGNOSTIC_ENABLED || state == 0 {
        return trace;
    }
    let Some(trace) = trace else {
        return None;
    };
    if !law_custom_scene_locked_flag_patch_candidate(
        trace,
        current_law_extra_slot_probe_variant_id(),
    ) {
        return Some(trace);
    }

    let Some(flag_address) = state.checked_add(COSTUME_SCENE_FLAG_LOCKED_OFFSET) else {
        log_law_custom_scene_locked_flag_patch(
            label,
            state,
            trace,
            "reason=flag_address_overflow".to_string(),
        );
        return Some(trace);
    };

    let before = trace.locked_flag;
    let write_result = win::write_process_memory(flag_address, &[0]);
    let written = write_result.unwrap_or(0);
    let write_ok = write_result == Some(1);
    let after = read_costume_scene_trace(state);
    let after_flag = after.and_then(|trace| trace.locked_flag);
    log_law_custom_scene_locked_flag_patch(
        label,
        state,
        trace,
        format!(
            "flag_address=0x{flag_address:x} before={} after={} patched={} write_ok={write_ok} written={written}",
            format_optional_u8(before),
            format_optional_u8(after_flag),
            before.is_some_and(|flag| flag != 0) && after_flag == Some(0)
        ),
    );
    after.or(Some(trace))
}

fn law_custom_scene_available_check_result(
    result: u64,
    before: Option<CostumeSceneTrace>,
    after: Option<CostumeSceneTrace>,
    custom_variant: Option<u16>,
) -> (u64, bool) {
    if !LAW_EXTRA_SLOT_FORCE_SCENE_AVAILABLE_DIAGNOSTIC_ENABLED || result != 0 {
        return (result, false);
    }

    let should_force = before
        .is_some_and(|trace| law_custom_scene_available_check_candidate(trace, custom_variant))
        || after
            .is_some_and(|trace| law_custom_scene_available_check_candidate(trace, custom_variant));
    if should_force {
        (1, true)
    } else {
        (result, false)
    }
}

fn costume_scene_trace_uses_important_budget(
    force_interesting: bool,
    before: Option<CostumeSceneTrace>,
    after: Option<CostumeSceneTrace>,
) -> bool {
    force_interesting
        || law_custom_slot_active()
        || before.is_some_and(costume_scene_trace_has_current_custom)
        || after.is_some_and(costume_scene_trace_has_current_custom)
}

fn costume_scene_trace_log_scope(
    interesting: bool,
    important_after_regular_cap: bool,
    regular_logged: usize,
    important_logged: usize,
) -> Option<CostumeSceneTraceLogScope> {
    if regular_logged < MAX_COSTUME_SCENE_TRACE_LOGS {
        if !interesting
            && !important_after_regular_cap
            && regular_logged >= COSTUME_SCENE_UNINTERESTING_TRACE_LOGS
        {
            return None;
        }
        return Some(CostumeSceneTraceLogScope::Regular);
    }

    if important_after_regular_cap && important_logged < MAX_COSTUME_SCENE_IMPORTANT_TRACE_LOGS {
        return Some(CostumeSceneTraceLogScope::Important);
    }

    None
}

fn costume_scene_list_log_scope(
    important_after_regular_cap: bool,
    regular_logged: usize,
    important_logged: usize,
) -> Option<CostumeSceneTraceLogScope> {
    if regular_logged < MAX_COSTUME_SCENE_LIST_TRACE_LOGS {
        return Some(CostumeSceneTraceLogScope::Regular);
    }

    if important_after_regular_cap && important_logged < MAX_COSTUME_SCENE_LIST_IMPORTANT_TRACE_LOGS
    {
        return Some(CostumeSceneTraceLogScope::Important);
    }

    None
}

fn scene_list_flag_slot_for_preview(
    index: u32,
    layout: u32,
    layouts: &[u32; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT],
) -> Option<usize> {
    let index = usize::try_from(index).ok()?;
    if index < COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT {
        return Some(index);
    }

    layouts
        .iter()
        .position(|entry_layout| *entry_layout == layout)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CostumePreviewVisibleDecision {
    direct_slot: Option<usize>,
    layout_slot: Option<usize>,
    decision_slot: Option<usize>,
    decision_flag: Option<u8>,
}

impl CostumePreviewVisibleDecision {
    fn decision_rule(self) -> &'static str {
        if self.direct_slot.is_some() {
            "direct-index"
        } else {
            "layout-scan"
        }
    }

    fn flag_gate(self) -> &'static str {
        match self.decision_flag {
            Some(0) => "flag-zero",
            Some(_) => "flag-open",
            None => "missing-slot",
        }
    }

    fn would_call_busy_check(self) -> bool {
        self.decision_flag.is_some_and(|flag| flag != 0)
    }
}

fn costume_preview_visible_decision(
    index: u32,
    layout: u32,
    layouts: &[u32; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT],
    flags: &[u8; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT],
) -> CostumePreviewVisibleDecision {
    let direct_slot = usize::try_from(index)
        .ok()
        .filter(|slot| *slot < COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT);
    let layout_slot = layouts
        .iter()
        .position(|entry_layout| *entry_layout == layout);
    let decision_slot = direct_slot.or(layout_slot);
    let decision_flag = decision_slot.and_then(|slot| flags.get(slot).copied());

    CostumePreviewVisibleDecision {
        direct_slot,
        layout_slot,
        decision_slot,
        decision_flag,
    }
}

fn law_custom_scene_list_flag_patch_candidate(
    trace: CostumeSceneTrace,
    custom_variant: Option<u16>,
) -> bool {
    let Some(custom_variant) = custom_variant else {
        return false;
    };
    if trace.layout != Some(u32::from(LAW_MASTER_LAYOUT_ID)) {
        return false;
    }

    trace.selected_variant == Some(custom_variant)
        || trace.object_variant == Some(u32::from(custom_variant))
}

fn log_law_custom_scene_list_flag_patch(
    label: &str,
    state: usize,
    trace: CostumeSceneTrace,
    detail: String,
) {
    let index = COSTUME_SCENE_LIST_FLAG_PATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_SCENE_LIST_FLAG_PATCH_LOGS {
        return;
    }
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Law custom scene-list flag diagnostic phase={label} state=0x{state:x} trace=[{}] {detail} frames={}",
        format_costume_scene_trace(trace),
        format_stack_frames(&frames, 6)
    ));
}

fn patch_law_custom_scene_list_preview_flag(
    label: &str,
    state: usize,
    trace: Option<CostumeSceneTrace>,
) {
    if !LAW_EXTRA_SLOT_SCENE_LIST_FLAG_DIAGNOSTIC_ENABLED || state == 0 {
        return;
    }
    let Some(trace) = trace else {
        return;
    };
    if !law_custom_scene_list_flag_patch_candidate(trace, current_law_extra_slot_probe_variant_id())
    {
        return;
    }

    let Some(layout) = trace.layout else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            "reason=missing_layout".to_string(),
        );
        return;
    };
    let Some(bank) = trace.list_bank else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            "reason=missing_bank".to_string(),
        );
        return;
    };
    let Some(index) = trace.list_index else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            "reason=missing_index".to_string(),
        );
        return;
    };
    if bank as usize >= COSTUME_SCENE_LIST_BANK_COUNT {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            format!("reason=bank_out_of_range bank={bank} index={index}"),
        );
        return;
    }

    let Some(starts) = read_scene_list_bank_starts(state) else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            format!("reason=starts_read_failed bank={bank} index={index}"),
        );
        return;
    };
    let start = starts[bank as usize];
    let Some(entry_offset) = (start as usize)
        .checked_mul(COSTUME_SCENE_LIST_ENTRY_STRIDE)
        .and_then(|offset| COSTUME_SCENE_LIST_BASE_OFFSET.checked_add(offset))
    else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            format!("reason=entry_offset_overflow bank={bank} index={index} start={start}"),
        );
        return;
    };
    let Some(entry) = state.checked_add(entry_offset) else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            format!("reason=entry_address_overflow bank={bank} index={index} start={start}"),
        );
        return;
    };
    let Some(layouts) = read_scene_list_entry_layouts(entry) else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            format!(
                "reason=layouts_read_failed bank={bank} index={index} starts={} entry=0x{entry:x}",
                format_u32_list(&starts)
            ),
        );
        return;
    };
    let Some(flag_slot) = scene_list_flag_slot_for_preview(index, layout, &layouts) else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            format!(
                "reason=flag_slot_not_found bank={bank} index={index} starts={} entry=0x{entry:x} layouts={}",
                format_u32_list(&starts),
                format_u32_list(&layouts)
            ),
        );
        return;
    };
    let Some(flag_offset) = COSTUME_SCENE_LIST_ENTRY_FLAGS_OFFSET.checked_add(flag_slot) else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            format!(
                "reason=flag_offset_overflow bank={bank} index={index} entry=0x{entry:x} flag_slot={flag_slot}"
            ),
        );
        return;
    };
    let Some(flag_address) = entry.checked_add(flag_offset) else {
        log_law_custom_scene_list_flag_patch(
            label,
            state,
            trace,
            format!(
                "reason=flag_address_overflow bank={bank} index={index} entry=0x{entry:x} flag_slot={flag_slot}"
            ),
        );
        return;
    };

    let before = read_u8_field(entry, flag_offset);
    let mut patched = false;
    let mut write_ok = before.is_some();
    let mut written = 0usize;
    if before == Some(0) {
        let write_result = win::write_process_memory(flag_address, &[1]);
        written = write_result.unwrap_or(0);
        write_ok = write_result == Some(1);
        patched = write_ok;
    }
    let after = read_u8_field(entry, flag_offset);
    let slot_layout = layouts.get(flag_slot).copied();
    log_law_custom_scene_list_flag_patch(
        label,
        state,
        trace,
        format!(
            "bank={bank} index={index} starts={} entry=0x{entry:x} layouts={} flag_slot={flag_slot} slot_layout={} flag_address=0x{flag_address:x} before={} after={} patched={patched} write_ok={write_ok} written={written}",
            format_u32_list(&starts),
            format_u32_list(&layouts),
            format_optional_u32(slot_layout),
            format_optional_u8(before),
            format_optional_u8(after),
        ),
    );
}

fn log_costume_scene_state_call(
    label: &str,
    state: usize,
    detail: String,
    before: Option<CostumeSceneTrace>,
    after: Option<CostumeSceneTrace>,
    force_interesting: bool,
) {
    let interesting = force_interesting
        || before.is_some_and(is_interesting_costume_scene_trace)
        || after.is_some_and(is_interesting_costume_scene_trace);
    let important_after_regular_cap =
        costume_scene_trace_uses_important_budget(force_interesting, before, after);
    let Some(scope) = costume_scene_trace_log_scope(
        interesting,
        important_after_regular_cap,
        COSTUME_SCENE_TRACE_LOGS.load(Ordering::Relaxed),
        COSTUME_SCENE_IMPORTANT_TRACE_LOGS.load(Ordering::Relaxed),
    ) else {
        return;
    };
    match scope {
        CostumeSceneTraceLogScope::Regular => {
            let index = COSTUME_SCENE_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
            if index >= MAX_COSTUME_SCENE_TRACE_LOGS {
                return;
            }
        }
        CostumeSceneTraceLogScope::Important => {
            let index = COSTUME_SCENE_IMPORTANT_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
            if index >= MAX_COSTUME_SCENE_IMPORTANT_TRACE_LOGS {
                return;
            }
        }
    }

    let before_text = before
        .map(format_costume_scene_trace)
        .unwrap_or_else(|| "none".to_string());
    let after_text = after
        .map(format_costume_scene_trace)
        .unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    let detail = if detail.is_empty() {
        String::new()
    } else {
        format!(" {detail}")
    };
    let scope_text = if scope == CostumeSceneTraceLogScope::Important {
        " scope=important"
    } else {
        ""
    };
    log::write_line(format!(
        "Costume scene {label}{scope_text} state=0x{state:x}{detail} before=[{before_text}] after=[{after_text}] frames={}",
        format_stack_frames(&frames, 6)
    ));
}

fn log_costume_preview_visible_decision(
    label: &str,
    state: usize,
    param2: i32,
    result: u64,
    trace: Option<CostumeSceneTrace>,
) {
    let Some(trace) = trace else {
        return;
    };
    let interesting = law_custom_slot_active()
        || is_interesting_costume_scene_trace(trace)
        || costume_scene_trace_has_current_custom(trace);
    if state == 0 || !interesting {
        return;
    }

    let index = COSTUME_PREVIEW_VISIBLE_DECISION_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_PREVIEW_VISIBLE_DECISION_TRACE_LOGS {
        return;
    }

    let detail = format_costume_preview_visible_decision_detail(state, trace);
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume preview visible-decision reason={label} state=0x{state:x} p2={param2} result={result} trace=[{}] {detail} frames={}",
        format_costume_scene_trace(trace),
        format_stack_frames(&frames, 6)
    ));
}

fn format_costume_preview_visible_decision_detail(
    state: usize,
    trace: CostumeSceneTrace,
) -> String {
    let Some(layout) = trace.layout else {
        return "reason=missing_layout".to_string();
    };
    let Some(bank) = trace.list_bank else {
        return format!("reason=missing_bank layout={layout}");
    };
    let Some(index) = trace.list_index else {
        return format!("reason=missing_index layout={layout} bank={bank}");
    };
    if bank as usize >= COSTUME_SCENE_LIST_BANK_COUNT {
        return format!("reason=bank_out_of_range layout={layout} bank={bank} index={index}");
    }

    let Some(starts) = read_scene_list_bank_starts(state) else {
        return format!("reason=starts_read_failed layout={layout} bank={bank} index={index}");
    };
    let start = starts[bank as usize];
    let Some(entry_offset) = (start as usize)
        .checked_mul(COSTUME_SCENE_LIST_ENTRY_STRIDE)
        .and_then(|offset| COSTUME_SCENE_LIST_BASE_OFFSET.checked_add(offset))
    else {
        return format!(
            "reason=entry_offset_overflow layout={layout} bank={bank} index={index} start={start}"
        );
    };
    let Some(entry) = state.checked_add(entry_offset) else {
        return format!(
            "reason=entry_address_overflow layout={layout} bank={bank} index={index} start={start}"
        );
    };
    let Some(layouts) = read_scene_list_entry_layouts(entry) else {
        return format!(
            "reason=layouts_read_failed layout={layout} bank={bank} index={index} starts={} entry=0x{entry:x}",
            format_u32_list(&starts)
        );
    };
    let Some(flags) = read_scene_list_entry_flags(entry) else {
        return format!(
            "reason=flags_read_failed layout={layout} bank={bank} index={index} starts={} entry=0x{entry:x} layouts={}",
            format_u32_list(&starts),
            format_u32_list(&layouts)
        );
    };

    let decision = costume_preview_visible_decision(index, layout, &layouts, &flags);
    let direct_flag = decision
        .direct_slot
        .and_then(|slot| flags.get(slot).copied());
    let layout_flag = decision
        .layout_slot
        .and_then(|slot| flags.get(slot).copied());
    format!(
        "layout={layout} bank={bank} index={index} starts={} entry=0x{entry:x} layouts={} flags={} rule={} direct_slot={} direct_flag={} layout_slot={} layout_flag={} decision_slot={} decision_flag={} flag_gate={} would_call_492e20={}",
        format_u32_list(&starts),
        format_u32_list(&layouts),
        format_bytes(&flags),
        decision.decision_rule(),
        format_optional_usize_decimal(decision.direct_slot),
        format_optional_u8(direct_flag),
        format_optional_usize_decimal(decision.layout_slot),
        format_optional_u8(layout_flag),
        format_optional_usize_decimal(decision.decision_slot),
        format_optional_u8(decision.decision_flag),
        decision.flag_gate(),
        decision.would_call_busy_check()
    )
}

#[allow(clippy::too_many_arguments)]
fn log_costume_scene_list_build_detail(
    state: usize,
    list_base: usize,
    param2: usize,
    params: [u32; 2],
    before_trace: Option<CostumeSceneTrace>,
    after_trace: Option<CostumeSceneTrace>,
    before: Option<CostumeSceneListRebuildSnapshot>,
    after: Option<CostumeSceneListRebuildSnapshot>,
    before_source: &str,
    after_source: &str,
    banks: &str,
) {
    let interesting = law_custom_slot_active()
        || before_trace.is_some_and(is_interesting_costume_scene_trace)
        || after_trace.is_some_and(is_interesting_costume_scene_trace)
        || before_trace.is_some_and(costume_scene_trace_has_current_custom)
        || after_trace.is_some_and(costume_scene_trace_has_current_custom)
        || costume_scene_list_rebuild_snapshot_is_interesting(before)
        || costume_scene_list_rebuild_snapshot_is_interesting(after);
    if !interesting {
        return;
    }

    let index = COSTUME_SCENE_LIST_BUILD_DETAIL_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_SCENE_LIST_BUILD_DETAIL_TRACE_LOGS {
        return;
    }

    let before_decision = before_trace
        .map(|trace| format_costume_preview_visible_decision_detail(state, trace))
        .unwrap_or_else(|| "none".to_string());
    let after_decision = after_trace
        .map(|trace| format_costume_preview_visible_decision_detail(state, trace))
        .unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume scene list-build detail state=0x{state:x} list_base=0x{list_base:x} p2=0x{param2:x} p3={} p4={} before_trace=[{}] after_trace=[{}] before_list=[{}] after_list=[{}] before_source=[{}] after_source=[{}] banks=[{}] before_decision=[{}] after_decision=[{}] frames={}",
        params[0],
        params[1],
        before_trace
            .map(format_costume_scene_trace)
            .unwrap_or_else(|| "none".to_string()),
        after_trace
            .map(format_costume_scene_trace)
            .unwrap_or_else(|| "none".to_string()),
        format_costume_scene_list_rebuild_snapshot(before),
        format_costume_scene_list_rebuild_snapshot(after),
        before_source,
        after_source,
        banks,
        before_decision,
        after_decision,
        format_stack_frames(&frames, 8)
    ));
}

fn log_costume_scene_list_rebuild_detail(
    state: usize,
    list_base: usize,
    param2: usize,
    params: [u32; 4],
    before_trace: Option<CostumeSceneTrace>,
    after_trace: Option<CostumeSceneTrace>,
    before: Option<CostumeSceneListRebuildSnapshot>,
    after: Option<CostumeSceneListRebuildSnapshot>,
) {
    let interesting = law_custom_slot_active()
        || before_trace.is_some_and(is_interesting_costume_scene_trace)
        || after_trace.is_some_and(is_interesting_costume_scene_trace)
        || before_trace.is_some_and(costume_scene_trace_has_current_custom)
        || after_trace.is_some_and(costume_scene_trace_has_current_custom)
        || costume_scene_list_rebuild_snapshot_is_interesting(before)
        || costume_scene_list_rebuild_snapshot_is_interesting(after);
    if !interesting {
        return;
    }

    let index = COSTUME_SCENE_LIST_REBUILD_DETAIL_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_SCENE_LIST_REBUILD_DETAIL_TRACE_LOGS {
        return;
    }

    let before_detail = format_costume_scene_list_rebuild_snapshot(before);
    let after_detail = format_costume_scene_list_rebuild_snapshot(after);
    let before_decision = before_trace
        .map(|trace| format_costume_preview_visible_decision_detail(state, trace))
        .unwrap_or_else(|| "none".to_string());
    let after_decision = after_trace
        .map(|trace| format_costume_preview_visible_decision_detail(state, trace))
        .unwrap_or_else(|| "none".to_string());
    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume scene list-rebuild detail state=0x{state:x} list_base=0x{list_base:x} p2=0x{param2:x} p3={} p4={} p5={} p6={} before_trace=[{}] after_trace=[{}] before_list=[{}] after_list=[{}] before_decision=[{}] after_decision=[{}] frames={}",
        params[0],
        params[1],
        params[2],
        params[3],
        before_trace
            .map(format_costume_scene_trace)
            .unwrap_or_else(|| "none".to_string()),
        after_trace
            .map(format_costume_scene_trace)
            .unwrap_or_else(|| "none".to_string()),
        before_detail,
        after_detail,
        before_decision,
        after_decision,
        format_stack_frames(&frames, 8)
    ));
}

fn costume_scene_list_rebuild_snapshot_is_interesting(
    snapshot: Option<CostumeSceneListRebuildSnapshot>,
) -> bool {
    let Some(snapshot) = snapshot else {
        return false;
    };
    snapshot.selected_layout.is_some_and(is_law_menu_value)
        || snapshot.row == Some(LAW_MENU_ROW_ID as u32)
        || snapshot
            .active_layouts
            .is_some_and(|layouts| layouts.iter().copied().any(is_law_menu_value))
}

fn log_costume_scene_list_probe(label: &str, state: usize, trace: Option<CostumeSceneTrace>) {
    let Some(trace) = trace else {
        return;
    };
    let important_after_regular_cap =
        law_custom_slot_active() || costume_scene_trace_has_current_custom(trace);
    if state == 0 || (!is_interesting_costume_scene_trace(trace) && !important_after_regular_cap) {
        return;
    }

    let Some(bank) = trace.list_bank else {
        return;
    };
    let Some(index) = trace.list_index else {
        return;
    };
    if bank as usize >= COSTUME_SCENE_LIST_BANK_COUNT {
        return;
    }

    let Some(starts) = read_scene_list_bank_starts(state) else {
        return;
    };
    let start = starts[bank as usize];
    let entry =
        state + COSTUME_SCENE_LIST_BASE_OFFSET + start as usize * COSTUME_SCENE_LIST_ENTRY_STRIDE;
    let Some(layouts) = read_scene_list_entry_layouts(entry) else {
        return;
    };
    let Some(flags) = read_scene_list_entry_flags(entry) else {
        return;
    };

    let entry_is_interesting = layouts.iter().copied().any(is_law_menu_value)
        || trace.layout.is_some_and(is_law_menu_value)
        || trace
            .selected_variant
            .is_some_and(|value| is_law_menu_value(value as u32));
    if !entry_is_interesting && !important_after_regular_cap {
        return;
    }

    let Some(scope) = costume_scene_list_log_scope(
        important_after_regular_cap,
        COSTUME_SCENE_LIST_TRACE_LOGS.load(Ordering::Relaxed),
        COSTUME_SCENE_LIST_IMPORTANT_TRACE_LOGS.load(Ordering::Relaxed),
    ) else {
        return;
    };
    match scope {
        CostumeSceneTraceLogScope::Regular => {
            let index = COSTUME_SCENE_LIST_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
            if index >= MAX_COSTUME_SCENE_LIST_TRACE_LOGS {
                return;
            }
        }
        CostumeSceneTraceLogScope::Important => {
            let index = COSTUME_SCENE_LIST_IMPORTANT_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
            if index >= MAX_COSTUME_SCENE_LIST_IMPORTANT_TRACE_LOGS {
                return;
            }
        }
    }
    let scope_text = if scope == CostumeSceneTraceLogScope::Important {
        " scope=important"
    } else {
        ""
    };

    log::write_line(format!(
        "Costume scene list probe reason={}{} state=0x{:x} bank={} index={} starts={} entry=0x{:x} layouts={} flags={} selected_layout={} selected_variant={} selected_slot={}",
        label,
        scope_text,
        state,
        bank,
        index,
        format_u32_list(&starts),
        entry,
        format_u32_list(&layouts),
        format_bytes(&flags),
        format_optional_u32(trace.layout),
        format_optional_u16_decimal(trace.selected_variant),
        format_optional_u32(trace.selected_slot)
    ));
}

fn read_scene_list_bank_starts(state: usize) -> Option<[u32; COSTUME_SCENE_LIST_BANK_COUNT]> {
    let mut starts = [0u32; COSTUME_SCENE_LIST_BANK_COUNT];
    for (bank, start) in starts.iter_mut().enumerate() {
        *start = read_u32_field(state, COSTUME_SCENE_LIST_BANK_STARTS_OFFSET + bank * 4)?;
    }
    Some(starts)
}

fn read_scene_list_bank_starts_from_list_base(
    list_base: usize,
) -> Option<[u32; COSTUME_SCENE_LIST_BANK_COUNT]> {
    let mut starts = [0u32; COSTUME_SCENE_LIST_BANK_COUNT];
    for (bank, start) in starts.iter_mut().enumerate() {
        *start = read_u32_field(
            list_base,
            COSTUME_SCENE_LIST_REBUILD_BANK_STARTS_OFFSET + bank * 4,
        )?;
    }
    Some(starts)
}

fn read_scene_list_entry_layouts(
    entry: usize,
) -> Option<[u32; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT]> {
    let mut layouts = [0u32; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT];
    for (slot, layout) in layouts.iter_mut().enumerate() {
        *layout = read_u32_field(entry, slot * 4)?;
    }
    Some(layouts)
}

fn read_scene_list_entry_flags(
    entry: usize,
) -> Option<[u8; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT]> {
    let mut flags = [0u8; COSTUME_SCENE_LIST_ENTRY_LAYOUT_COUNT];
    for (slot, flag) in flags.iter_mut().enumerate() {
        *flag = read_u8_field(entry, COSTUME_SCENE_LIST_ENTRY_FLAGS_OFFSET + slot)?;
    }
    Some(flags)
}

unsafe extern "system" fn hooked_costume_validator(layout_id: u32, row_id: i32, mode: i32) -> u64 {
    let original_address = COSTUME_VALIDATOR_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let original: CostumeValidatorFn = std::mem::transmute(original_address);
    let result = original(layout_id, row_id, mode);
    log_costume_validator_call(layout_id, row_id, mode, result);
    result
}

fn log_costume_validator_call(layout_id: u32, row_id: i32, mode: i32, result: u64) {
    let interesting =
        is_law_menu_value(layout_id) || is_law_menu_i32(row_id) || is_law_menu_value(result as u32);
    let logged = COSTUME_VALIDATOR_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= COSTUME_VALIDATOR_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = COSTUME_VALIDATOR_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_VALIDATOR_TRACE_LOGS {
        return;
    }

    let frames = capture_stack_trace();
    log::write_line(format!(
        "Costume validator layout={} row={} mode={} result={} frames={}",
        layout_id,
        row_id,
        mode,
        result,
        format_stack_frames(&frames, 6)
    ));
}

unsafe extern "system" fn hooked_costume_resolver(
    layout_id: u32,
    row_id: u32,
    mode: u32,
    output: *mut u32,
) -> u64 {
    let original_address = COSTUME_RESOLVER_ORIGINAL.load(Ordering::Acquire);
    if original_address == 0 {
        return 0;
    }
    let before = read_u32_pointer_values(output, 8);
    let before_states = read_tracked_preview_resource_states();
    let original: CostumeResolverFn = std::mem::transmute(original_address);
    let result = original(layout_id, row_id, mode, output);
    let after = read_u32_pointer_values(output, 8);
    let after_states = read_tracked_preview_resource_states();
    let frames = capture_stack_trace();
    log_costume_resolver_call(
        layout_id,
        row_id,
        mode,
        output,
        &before,
        result,
        &after,
        Some(before_states),
        Some(after_states),
        &frames,
    );
    result
}

fn log_costume_resolver_call(
    layout_id: u32,
    row_id: u32,
    mode: u32,
    output: *mut u32,
    before: &[u32],
    result: u64,
    after: &[u32],
    before_states: Option<TrackedPreviewResourceStates>,
    after_states: Option<TrackedPreviewResourceStates>,
    frames: &[usize],
) {
    let caller_rva = format_preview_resource_caller_rva(frames);
    let trace_context = last_law_ready_timeline_trace_label();
    let slot5_selected = trace_context.is_some_and(|(label, trace)| {
        label == "slot5-custom"
            && trace.selected_variant == current_law_extra_slot_probe_variant_id()
    });
    let known_preview_caller = matches!(
        caller_rva.as_str(),
        "game+0x14928f3" | "game+0x1490b95" | "game+0x1491c5f"
    );
    let interesting = is_law_menu_value(layout_id)
        || is_law_menu_value(row_id)
        || is_law_menu_value(mode)
        || is_law_menu_value(result as u32)
        || before.iter().copied().any(is_law_menu_value)
        || after.iter().copied().any(is_law_menu_value)
        || slot5_selected
        || known_preview_caller;
    let logged = COSTUME_RESOLVER_TRACE_LOGS.load(Ordering::Relaxed);
    if !interesting && logged >= COSTUME_RESOLVER_UNINTERESTING_TRACE_LOGS {
        return;
    }
    let index = COSTUME_RESOLVER_TRACE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_COSTUME_RESOLVER_TRACE_LOGS {
        return;
    }

    log_scene_preview_fallback_source(
        layout_id,
        row_id,
        mode,
        result,
        after,
        before_states,
        after_states,
        &caller_rva,
        trace_context,
        frames,
    );
    log::write_line(format!(
        "Costume resolver layout={} row={} mode={} out=0x{:x} before={} result={} caller_rva={} callsite={} after={} resource_before=[{}] resource_after=[{}] context=[{}] frames={}",
        layout_id,
        row_id,
        mode,
        output as usize,
        format_u32_list(before),
        result,
        caller_rva,
        preview_resource_callsite_label(&caller_rva),
        format_u32_list(after),
        format_tracked_preview_resource_states(before_states),
        format_tracked_preview_resource_states(after_states),
        preview_resource_trace_context(trace_context.map(|(_, trace)| trace)),
        format_stack_frames(&frames, 6)
    ));
}

fn log_scene_preview_fallback_source(
    layout_id: u32,
    row_id: u32,
    mode: u32,
    result: u64,
    after: &[u32],
    before_states: Option<TrackedPreviewResourceStates>,
    after_states: Option<TrackedPreviewResourceStates>,
    caller_rva: &str,
    trace_context: Option<(&'static str, CostumeObjectUpdateTrace)>,
    frames: &[usize],
) {
    let Some((label, trace)) = trace_context else {
        return;
    };
    if label != "slot5-custom" {
        return;
    }
    let resolved_preview_arg = u32::try_from(result).ok();
    let mapped_resource = resolved_preview_arg
        .and_then(|value| u16::try_from(value).ok())
        .map(preview_model_mapped_id_from_metadata_value);
    let preview_resource_interesting = mapped_resource.is_some_and(|resource| {
        resource == LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID
            || resource == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
    }) || resolved_preview_arg.is_some_and(|resource| {
        resource == LAW_EXTRA_SLOT_BASE_PREVIEW_MAPPED_RESOURCE_ID
            || resource == LAW_EXTRA_SLOT_ONI_PREVIEW_MAPPED_RESOURCE_ID
    });
    let known_callers = format_known_preview_resource_callers(frames);
    let known_preview_caller = known_preview_resource_callers(frames)
        .iter()
        .any(|rva| matches!(*rva, 0x14928f3 | 0x1490b95 | 0x1491c5f));
    if layout_id != u32::from(LAW_MASTER_LAYOUT_ID)
        && !preview_resource_interesting
        && !known_preview_caller
    {
        return;
    }
    log::write_line(format!(
        "scene-preview-fallback-source selected_variant={} layout={} row={} mode={} caller_rva={} callsite={} known_callers={} resolved_preview_arg={} mapped_resource={} output={} resource_before=[{}] resource_after=[{}] context=[{}] frames={}",
        format_optional_u16_decimal(trace.selected_variant),
        layout_id,
        row_id,
        mode,
        caller_rva,
        preview_resource_callsite_label(caller_rva),
        known_callers,
        format_optional_u32(resolved_preview_arg),
        format_optional_u32(mapped_resource),
        format_u32_list(after),
        format_tracked_preview_resource_states(before_states),
        format_tracked_preview_resource_states(after_states),
        format_costume_object_update_trace(trace),
        format_stack_frames(frames, 7)
    ));
}
