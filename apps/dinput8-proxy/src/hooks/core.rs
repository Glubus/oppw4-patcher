fn log_law_menu_row_slot_probe(dump: &CostumeTableDump) {
    if LAW_SLOT_SOURCE_PROBE_LOGS.fetch_add(1, Ordering::Relaxed) >= 4 {
        return;
    }

    let address =
        dump.static_rows + LAW_MENU_ROW_ID * COSTUME_PRIMARY_ROW_STRIDE + LAW_MENU_ROW_SLOT_OFFSET;
    let mut bytes = [0u8; LAW_MENU_ROW_SLOT_COUNT * size_of::<u16>()];
    if !read_exact_process_memory(address, &mut bytes) {
        log::write_line(format!(
            "Law menu row slot probe failed row={} address=0x{:x} error=read_failed",
            LAW_MENU_ROW_ID, address
        ));
        return;
    }

    let slots = bytes
        .chunks_exact(size_of::<u16>())
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    log::write_line(format!(
        "Law menu row slot probe row={} address=0x{:x} offset=0x{:x} u16={} raw={}",
        LAW_MENU_ROW_ID,
        address,
        LAW_MENU_ROW_SLOT_OFFSET,
        format_u16_list(&slots),
        format_bytes(&bytes)
    ));
}

fn log_law_ram_slot_source(dump: &CostumeTableDump, layout_dump: &CostumeLayoutTableDump) {
    let menu_address =
        dump.static_rows + LAW_MENU_ROW_ID * COSTUME_PRIMARY_ROW_STRIDE + LAW_MENU_ROW_SLOT_OFFSET;
    let mut menu_bytes = [0u8; LAW_MENU_ROW_SLOT_COUNT * size_of::<u16>()];
    let menu_values = if read_exact_process_memory(menu_address, &mut menu_bytes) {
        menu_bytes
            .chunks_exact(size_of::<u16>())
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let layout_address =
        layout_dump.static_layouts + LAW_MASTER_LAYOUT_ID as usize * COSTUME_LAYOUT_ROW_STRIDE;
    let mut layout_raw = [0u8; COSTUME_LAYOUT_ROW_COPY_SIZE];
    let layout_raw_ok = read_exact_process_memory(layout_address, &mut layout_raw);
    let layout_variants = if layout_raw_ok {
        (0..COSTUME_LAYOUT_VARIANT_COUNT as usize)
            .filter_map(|index| {
                let offset = COSTUME_LAYOUT_VARIANTS_OFFSET + index * size_of::<u16>();
                layout_raw
                    .get(offset..offset + size_of::<u16>())
                    .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let active_count = layout_raw
        .get(COSTUME_LAYOUT_AVAILABILITY_FLAG_OFFSET - 2)
        .copied();
    let custom_slot = layout_variants
        .get(LAW_DUPLICATE_VARIANT_SLOT_INDEX)
        .copied();

    log::write_line(format!(
        "law-ram-slot-source static_rows=0x{:x} static_layouts=0x{:x} row={} row_slot_address=0x{:x} row_slot_values={} row_slot_raw={} layout={} layout_address=0x{:x} layout_active_count={} layout_custom_slot={} layout_variants={} layout_raw={}",
        dump.static_rows,
        layout_dump.static_layouts,
        LAW_MENU_ROW_ID,
        menu_address,
        if menu_values.is_empty() { "read_failed".to_string() } else { format_u16_list(&menu_values) },
        format_bytes(&menu_bytes),
        LAW_MASTER_LAYOUT_ID,
        layout_address,
        active_count.map(|value| value.to_string()).unwrap_or_else(|| "none".to_string()),
        format_optional_u16_decimal(custom_slot),
        if layout_variants.is_empty() { "read_failed".to_string() } else { format_u16_list(&layout_variants) },
        if layout_raw_ok { format_bytes(&layout_raw) } else { "read_failed".to_string() }
    ));

    if layout_raw_ok {
        if let Some(candidate_raw) = law_linkdata_layout26_candidate_raw() {
            let candidate_prefix = candidate_raw.get(..layout_raw.len());
            let matches = candidate_prefix == Some(layout_raw.as_slice());
            let candidate_active_count = candidate_raw
                .get(COSTUME_LAYOUT_AVAILABILITY_FLAG_OFFSET - 2)
                .copied();
            let candidate_slot4 = read_u16_from_bytes(
                candidate_raw,
                COSTUME_LAYOUT_VARIANTS_OFFSET
                    + LAW_DUPLICATE_VARIANT_SLOT_INDEX as usize * size_of::<u16>(),
            );
            log::write_line(format!(
                "law-linkdata-vs-ram-layout26 match={} offset_delta=unknown slot4_linkdata={} slot4_ram={} active_count_linkdata={} active_count_ram={} entry3_candidate_raw={} ram_layout26_raw={}",
                matches,
                format_optional_u16_decimal(candidate_slot4),
                format_optional_u16_decimal(custom_slot),
                candidate_active_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                active_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                format_bytes(candidate_raw),
                format_bytes(&layout_raw)
            ));
        } else {
            log::write_line(format!(
                "law-linkdata-vs-ram-layout26 match=false reason=no_entry3_candidate ram_layout26_raw={}",
                format_bytes(&layout_raw)
            ));
        }
    }
}

fn log_costume_table_dump(phase: &str, trigger: &str, dump: &CostumeTableDump) {
    log::write_line(format!(
        "Global costume table dump phase={phase} trigger={trigger} static_db=0x{:x} static_root=0x{:x} static_rows=0x{:x} runtime_db={} runtime_root={} runtime_rows={} rows={} enabled={} hidden={} sort26_candidates={} clone_candidates={} groups={}",
        dump.static_database,
        dump.static_root,
        dump.static_rows,
        format_optional_address(dump.runtime_database),
        format_optional_address(dump.runtime_root),
        format_optional_address(dump.runtime_rows),
        dump.rows.len(),
        dump.enabled_rows(),
        dump.hidden_rows(),
        dump.sort26_candidate_rows(),
        dump.clone_candidate_rows(),
        format_group_counts(&dump.group_counts())
    ));
    log_costume_group_lists(dump);
    for row in dump
        .rows
        .iter()
        .copied()
        .filter(|row| row.should_log_default())
    {
        log_costume_table_row(row);
        if row.should_log_detail() {
            log_costume_table_row_detail(row);
        }
    }
}

fn log_costume_layout_table_dump(dump: &CostumeLayoutTableDump) {
    log::write_line(format!(
        "Global costume layout dump static_db=0x{:x} static_root=0x{:x} static_layouts=0x{:x} static_rows=0x{:x} runtime_db={} runtime_root={} logged_layouts={} logged_law_category_layouts={} logged_newgate_category_layouts={} matrix_probes={} variant_owners={} free_layout_candidates={}",
        dump.static_database,
        dump.static_root,
        dump.static_layouts,
        dump.static_rows,
        format_optional_address(dump.runtime_database),
        format_optional_address(dump.runtime_root),
        dump.layouts.len(),
        dump.logged_law_category_layouts(),
        dump.logged_newgate_category_layouts(),
        dump.matrix_probes.len(),
        dump.variant_owners.len(),
        dump.free_layout_candidates.len()
    ));
    log_law_extra_slot_probe_id_usage(dump);
    for layout in dump.layouts.iter().copied() {
        log_costume_layout(layout);
    }
    for probe in dump.matrix_probes.iter().copied() {
        log_costume_layout_matrix_probe(probe);
    }
}

fn log_law_extra_slot_probe_id_usage(dump: &CostumeLayoutTableDump) {
    let owners = dump.owners_of_variant(LAW_EXTRA_SLOT_PROBE_FALLBACK_VARIANT_ID);
    let free_preview = dump
        .free_layout_candidates
        .iter()
        .copied()
        .take(48)
        .collect::<Vec<_>>();
    log::write_line(format!(
        "Global costume variant id probe variant={} owners={} free_layout_candidates_first={}",
        LAW_EXTRA_SLOT_PROBE_FALLBACK_VARIANT_ID,
        format_costume_variant_owners(&owners),
        format_u16_list(&free_preview)
    ));
}

fn current_law_extra_slot_probe_variant_id() -> Option<u16> {
    let variant = LAW_EXTRA_SLOT_PROBE_VARIANT_ID.load(Ordering::Relaxed);
    if variant == 0 {
        law_slot5_id699_preflight_ok().then_some(LAW_CUSTOM_LAYOUT_ID)
    } else {
        Some(variant as u16)
    }
}

fn set_law_extra_slot_probe_variant_id(variant_id: u16) {
    LAW_EXTRA_SLOT_PROBE_VARIANT_ID.store(usize::from(variant_id), Ordering::Relaxed);
}

fn choose_law_extra_slot_allocation(
    dump: &CostumeLayoutTableDump,
) -> Option<CustomVariantAllocation> {
    let allocated_variant_id = dump
        .free_layout_candidates
        .iter()
        .copied()
        .filter(|candidate| usize::from(*candidate) < costume_table::LAYOUT_ROW_COUNT)
        .filter(|candidate| *candidate < LAW_EXTRA_SLOT_SELECTABLE_VARIANT_LIMIT_EXCLUSIVE)
        .filter(|candidate| *candidate != LAW_MASTER_LAYOUT_ID)
        .filter(|candidate| *candidate != costume_table::LAW_DUPLICATE_VARIANT_ID)
        .filter(|candidate| dump.owners_of_variant(*candidate).is_empty())
        .max()?;

    Some(CustomVariantAllocation {
        character: "Law",
        source_variant_id: costume_table::LAW_DUPLICATE_VARIANT_ID,
        metadata_source_layout_id: LAW_EXTRA_SLOT_METADATA_SOURCE_LAYOUT_ID,
        metadata_source_variant_id: LAW_EXTRA_SLOT_METADATA_SOURCE_VARIANT_ID,
        allocated_variant_id,
        slot_index: LAW_DUPLICATE_VARIANT_SLOT_INDEX,
        mode: "auto-free-layout",
    })
}

fn format_custom_variant_allocations(allocations: &[CustomVariantAllocation]) -> String {
    if allocations.is_empty() {
        return "none".to_string();
    }
    allocations
        .iter()
        .map(|allocation| {
            format!(
                "{} slot={} source={} metadata_layout={} metadata_variant={} allocated={} mode={}",
                allocation.character,
                allocation.slot_index,
                allocation.source_variant_id,
                allocation.metadata_source_layout_id,
                allocation.metadata_source_variant_id,
                allocation.allocated_variant_id,
                allocation.mode
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn law_extra_slot_metadata_clone_for_allocation(
    dump: &CostumeLayoutTableDump,
    allocation: CustomVariantAllocation,
) -> Option<CustomVariantMetadataClone> {
    Some(CustomVariantMetadataClone {
        source_layout_id: allocation.metadata_source_layout_id,
        target_layout_id: allocation.allocated_variant_id,
        source_address: costume_layout_row_address(
            dump.static_layouts,
            allocation.metadata_source_layout_id,
        )?,
        target_address: costume_layout_row_address(
            dump.static_layouts,
            allocation.allocated_variant_id,
        )?,
        byte_count: COSTUME_LAYOUT_ROW_COPY_SIZE,
    })
}

fn costume_layout_row_address(static_layouts: usize, layout_id: u16) -> Option<usize> {
    if u32::from(layout_id) >= COSTUME_LAYOUT_ROW_COUNT {
        return None;
    }
    Some(static_layouts + usize::from(layout_id) * COSTUME_LAYOUT_ROW_STRIDE)
}

fn costume_variant_metadata_record_address(
    static_layouts: usize,
    variant_id: u16,
) -> Option<usize> {
    if variant_id >= LAW_EXTRA_SLOT_SELECTABLE_VARIANT_LIMIT_EXCLUSIVE {
        return None;
    }
    if let Some(base) = law_variant_metadata_ram_base(static_layouts) {
        return base.checked_add(usize::from(variant_id) * COSTUME_VARIANT_METADATA_STRIDE);
    }
    legacy_costume_variant_metadata_record_address(static_layouts, variant_id)
}

fn legacy_costume_variant_metadata_record_address(
    static_layouts: usize,
    variant_id: u16,
) -> Option<usize> {
    if variant_id >= LAW_EXTRA_SLOT_SELECTABLE_VARIANT_LIMIT_EXCLUSIVE {
        return None;
    }
    Some(
        static_layouts
            + COSTUME_VARIANT_METADATA_BASE_OFFSET
            + usize::from(variant_id) * COSTUME_VARIANT_METADATA_STRIDE,
    )
}

#[derive(Clone)]
struct LawVariantMetadataRamCandidate {
    base: usize,
    matched_variants: Vec<u16>,
    target_address: usize,
    target_raw: [u8; COSTUME_VARIANT_METADATA_COPY_SIZE],
    source: &'static str,
}

fn law_variant_metadata_ram_base(static_layouts: usize) -> Option<usize> {
    let cached = LAW_VARIANT_METADATA_RAM_SELECTED_BASE.load(Ordering::Relaxed);
    if cached != 0 {
        return Some(cached);
    }
    let candidate = find_law_variant_metadata_ram_candidates(static_layouts)
        .into_iter()
        .find(|candidate| candidate.matched_variants.len() >= LAW_OFFICIAL_LAYOUT_VARIANTS.len())?;
    LAW_VARIANT_METADATA_RAM_SELECTED_BASE.store(candidate.base, Ordering::Relaxed);
    Some(candidate.base)
}

fn find_law_variant_metadata_ram_candidates(
    static_layouts: usize,
) -> Vec<LawVariantMetadataRamCandidate> {
    let Some(records) = law_linkdata_variant_metadata_records() else {
        return Vec::new();
    };
    let official_records = records
        .iter()
        .filter(|(variant, raw)| {
            LAW_OFFICIAL_LAYOUT_VARIANTS.contains(variant)
                && raw.len() == COSTUME_VARIANT_METADATA_COPY_SIZE
        })
        .collect::<Vec<_>>();
    if official_records.len() < LAW_OFFICIAL_LAYOUT_VARIANTS.len() {
        return Vec::new();
    }

    let start = static_layouts;
    let byte_count = COSTUME_VARIANT_METADATA_BASE_OFFSET
        + usize::from(LAW_EXTRA_SLOT_SELECTABLE_VARIANT_LIMIT_EXCLUSIVE)
            * COSTUME_VARIANT_METADATA_STRIDE
        + 0x4000;
    let mut memory = vec![0u8; byte_count];
    if !read_exact_process_memory(start, &mut memory) {
        return Vec::new();
    }

    let mut bases = HashSet::new();
    for (variant, raw) in &official_records {
        for (offset, _) in memory
            .windows(COSTUME_VARIANT_METADATA_COPY_SIZE)
            .enumerate()
            .filter(|(_, window)| *window == raw.as_slice())
        {
            let variant_offset = usize::from(*variant) * COSTUME_VARIANT_METADATA_STRIDE;
            if let Some(base) = start.checked_add(offset).and_then(|address| address.checked_sub(variant_offset)) {
                bases.insert(base);
            }
        }
    }

    let mut candidates = bases
        .into_iter()
        .filter_map(|base| {
            let mut matched_variants = Vec::new();
            for (variant, raw) in &official_records {
                let address =
                    base.checked_add(usize::from(*variant) * COSTUME_VARIANT_METADATA_STRIDE)?;
                let mut bytes = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
                if read_exact_process_memory(address, &mut bytes) && bytes.as_slice() == raw.as_slice() {
                    matched_variants.push(*variant);
                }
            }
            let target_address =
                base.checked_add(usize::from(LAW_CUSTOM_LAYOUT_ID) * COSTUME_VARIANT_METADATA_STRIDE)?;
            let mut target_raw = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
            if !read_exact_process_memory(target_address, &mut target_raw) {
                return None;
            }
            Some(LawVariantMetadataRamCandidate {
                base,
                matched_variants,
                target_address,
                target_raw,
                source: "raw_scan",
            })
        })
        .collect::<Vec<_>>();
    if let Some(legacy_candidate) =
        law_variant_metadata_legacy_semantic_candidate(static_layouts, &official_records)
    {
        candidates.push(legacy_candidate);
    }
    candidates.sort_by_key(|candidate| {
        (
            std::cmp::Reverse(candidate.matched_variants.len()),
            candidate.source != "legacy_semantic",
            candidate.base,
        )
    });
    candidates.truncate(6);
    candidates
}

fn law_variant_metadata_legacy_semantic_candidate(
    static_layouts: usize,
    official_records: &[&(u16, Vec<u8>)],
) -> Option<LawVariantMetadataRamCandidate> {
    let base = static_layouts + COSTUME_VARIANT_METADATA_BASE_OFFSET;
    let mut matched_variants = Vec::new();
    for (variant, expected_model, expected_preview) in LAW_OFFICIAL_VARIANT_METADATA_EXPECTATIONS {
        let address = legacy_costume_variant_metadata_record_address(static_layouts, variant)?;
        let mut actual_raw = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
        if !read_exact_process_memory(address, &mut actual_raw) {
            continue;
        }
        if read_u16_from_bytes(&actual_raw, COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET)
            == Some(expected_model)
            && read_u16_from_bytes(&actual_raw, COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET)
                == Some(expected_preview)
        {
            matched_variants.push(variant);
        }
    }
    if matched_variants.len() < LAW_OFFICIAL_LAYOUT_VARIANTS.len() {
        for record in official_records {
            let (variant, expected_raw) = *record;
            if matched_variants.contains(variant) {
                continue;
            }
            let address = legacy_costume_variant_metadata_record_address(static_layouts, *variant)?;
            let mut actual_raw = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
            if !read_exact_process_memory(address, &mut actual_raw) {
                continue;
            }
            if variant_metadata_semantic_match(&actual_raw, expected_raw) {
                matched_variants.push(*variant);
            }
        }
    }
    let target_address =
        legacy_costume_variant_metadata_record_address(static_layouts, LAW_CUSTOM_LAYOUT_ID)?;
    let mut target_raw = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
    if !read_exact_process_memory(target_address, &mut target_raw) {
        return None;
    }
    Some(LawVariantMetadataRamCandidate {
        base,
        matched_variants,
        target_address,
        target_raw,
        source: "legacy_semantic",
    })
}

fn variant_metadata_semantic_match(actual: &[u8], expected: &[u8]) -> bool {
    read_u16_from_bytes(actual, COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET)
        == read_u16_from_bytes(expected, COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET)
        && read_u16_from_bytes(actual, COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET)
            == read_u16_from_bytes(expected, COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET)
}

fn format_law_variant_metadata_ram_candidates(
    candidates: &[LawVariantMetadataRamCandidate],
) -> String {
    if candidates.is_empty() {
        return "none".to_string();
    }
    candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let target_model = read_u16_from_bytes(
                &candidate.target_raw,
                COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET,
            );
            let target_preview = read_u16_from_bytes(
                &candidate.target_raw,
                COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET,
            );
            let target_flags = candidate
                .target_raw
                .get(COSTUME_VARIANT_METADATA_FLAGS_OFFSET)
                .copied();
            format!(
                "{}:selected={} source={} base=0x{:x} matches={}/{} variants={} target_addr=0x{:x} target_model={} target_preview={} target_flags={} target_raw={}",
                index,
                index == 0 && candidate.matched_variants.len() >= LAW_OFFICIAL_LAYOUT_VARIANTS.len(),
                candidate.source,
                candidate.base,
                candidate.matched_variants.len(),
                LAW_OFFICIAL_LAYOUT_VARIANTS.len(),
                format_u16_list(&candidate.matched_variants),
                candidate.target_address,
                format_optional_u16_decimal(target_model),
                format_optional_u16_decimal(target_preview),
                format_optional_u8(target_flags),
                format_bytes(&candidate.target_raw)
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn runtime_category_slot_pointer_address(runtime_root: usize, category: u16) -> Option<usize> {
    if category as usize >= COSTUME_CATEGORY_COUNT {
        return None;
    }
    Some(
        runtime_root
            + (usize::from(category) + COSTUME_RUNTIME_CATEGORY_SLOT_BASE) * size_of::<usize>(),
    )
}

fn runtime_category_slot_flag_address(slot_base: usize, slot_index: usize) -> Option<usize> {
    if slot_index >= COSTUME_LAYOUT_VARIANT_COUNT as usize {
        return None;
    }
    Some(slot_base + COSTUME_RUNTIME_UNLOCK_SLOT_FLAGS_OFFSET + slot_index)
}

fn runtime_category_slot_flag_address_for_dump(
    dump: &CostumeLayoutTableDump,
    category: u16,
    slot_index: usize,
) -> Option<usize> {
    let runtime_root = dump.runtime_root?;
    let pointer_address = runtime_category_slot_pointer_address(runtime_root, category)?;
    let slot_base = read_usize_absolute(pointer_address)?;
    if slot_base == 0 {
        return None;
    }
    runtime_category_slot_flag_address(slot_base, slot_index)
}

fn log_costume_layout(layout: CostumeLayoutSnapshot) {
    log::write_line(format!(
        "Global costume layout id={} family={} category={} flags_4a=0x{:02x} active_variants={} priority={} category_flags={} category_gates={} variants={} raw={}",
        layout.layout_id,
        layout.family,
        layout.category,
        layout.flags_4a,
        layout.active_variant_count,
        format_optional_u16(layout.priority),
        format_optional_u16(layout.category_flags),
        layout
            .category_gates
            .as_ref()
            .map(|gates| format_bytes(gates))
            .unwrap_or_else(|| "none".to_string()),
        format_u16_list(&layout.variants),
        format_bytes(&layout.raw)
    ));
}

fn log_costume_layout_matrix_probe(probe: CostumeLayoutMatrixProbe) {
    log::write_line(format!(
        "Global costume layout matrix row={} layout={} byte=0x{:02x}",
        probe.row, probe.layout_id, probe.byte
    ));
}

fn maybe_patch_law_duplicate_variant_slot(dump: &CostumeLayoutTableDump) {
    if !DUPLICATE_LAW_VARIANT_SLOT_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if DUPLICATE_LAW_VARIANT_SLOT_DONE.swap(true, Ordering::Relaxed) {
        return;
    }

    let Some(allocation) = choose_law_extra_slot_allocation(dump) else {
        log::write_line("Custom variant allocation table entries=0 rows=none");
        log::write_line("Law extra slot variant patch skipped: no free custom variant id");
        return;
    };
    set_law_extra_slot_probe_variant_id(allocation.allocated_variant_id);
    log::write_line(format!(
        "Custom variant allocation table entries=1 rows={}",
        format_custom_variant_allocations(&[allocation])
    ));
    if !patch_law_extra_slot_metadata_clone(dump, allocation) {
        return;
    }
    if !patch_law_extra_slot_variant_metadata_clone(dump, allocation) {
        return;
    }
    let private_model_ready =
        LAW_EXTRA_SLOT_PRIVATE_MODEL_RAM_CLONE_ENABLED && patch_law_private_model_ram_clone();
    if LAW_EXTRA_SLOT_CUSTOM_MODEL_RESOURCE_PATCH_ENABLED {
        let manager_alias_ready = law_private_model_manager_alias_ready();
        let model_resource_id =
            law_extra_slot_model_resource_for_patch(private_model_ready, manager_alias_ready);
        if !private_model_ready && manager_alias_ready {
            log::write_line(format!(
                "Law private model RAM clone unavailable target_variant={} manager_alias_resource={} alias_source_resource={}",
                allocation.allocated_variant_id,
                model_resource_id,
                LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID
            ));
        } else if !private_model_ready {
            log::write_line(format!(
                "Law private model RAM clone unavailable target_variant={} linkdata_model_resource={model_resource_id}",
                allocation.allocated_variant_id
            ));
        }
        if !patch_law_extra_slot_variant_model_resource(dump, allocation, model_resource_id) {
            return;
        }
    } else {
        log::write_line(format!(
            "Law extra slot model resource patch disabled target_variant={} reason=requires_private_deep_copy",
            allocation.allocated_variant_id
        ));
    }
    if LAW_EXTRA_SLOT_PREVIEW_MAPPING_DIAGNOSTIC_ENABLED {
        patch_law_extra_slot_variant_preview_mapping(dump, allocation);
    } else {
        log::write_line(format!(
            "Law extra slot preview mapping diagnostic disabled target_variant={}",
            allocation.allocated_variant_id
        ));
    }
    if LAW_EXTRA_SLOT_COLOR_VARIATION_PATCH_ENABLED {
        patch_law_extra_slot_variant_color_variations(dump, allocation);
    } else {
        log::write_line(format!(
            "Law extra slot color variation patch disabled target_variant={}",
            allocation.allocated_variant_id
        ));
    }
    if LAW_EXTRA_SLOT_CUSTOM_NON_DLC_ADMISSION_ENABLED {
        if !patch_law_extra_slot_variant_metadata_flags(dump, allocation) {
            return;
        }
    } else {
        log::write_line(format!(
            "Law extra slot custom non-DLC admission disabled target_variant={} reason=post_unlock_crash_guard",
            allocation.allocated_variant_id
        ));
    }
    patch_law_extra_slot_runtime_unlock_slot(dump, allocation);

    let Some(variant_address) = dump.law_duplicate_variant_address() else {
        log::write_line("Law extra slot variant patch skipped: address unavailable");
        return;
    };
    if !patch_law_duplicate_variant_slot(variant_address) {
        return;
    }
    patch_law_duplicate_variant_count(dump);
}

fn maybe_patch_law_linkdata_only_ram_variant699_metadata(dump: &CostumeLayoutTableDump) {
    if !LAW_SLOT5_PATCH_VARIANT699_METADATA_RAM_ENABLED {
        return;
    }

    if !law_slot5_id699_preflight_ok() {
        log::write_line("law-slot5-variant699-metadata-ram-patch skipped reason=id699_preflight_blocked_or_missing");
        return;
    }

    let Some(layout) = dump
        .layouts
        .iter()
        .find(|layout| layout.layout_id == LAW_MASTER_LAYOUT_ID)
    else {
        log::write_line("law-linkdata-only-ram-variant699-metadata-patch skipped reason=layout26_missing");
        return;
    };
    let slot4 = layout
        .variants
        .get(LAW_DUPLICATE_VARIANT_SLOT_INDEX)
        .copied();
    if layout.active_variant_count != 5 || slot4 != Some(LAW_CUSTOM_LAYOUT_ID) {
        log::write_line(format!(
            "law-linkdata-only-ram-variant699-metadata-patch skipped reason=layout_not_custom active_count={} slot4={}",
            layout.active_variant_count,
            format_optional_u16_decimal(slot4)
        ));
        return;
    }

    maybe_patch_law_linkdata_only_ram_variant699_metadata_at_static_layouts(
        dump.static_layouts,
        "layout_dump",
    );
    maybe_patch_law_linkdata_only_runtime_unlock_slot_with_dump(dump, "layout_dump");
}

fn maybe_patch_law_linkdata_only_ram_variant699_metadata_at_static_layouts(
    static_layouts: usize,
    trigger: &str,
) {
    if !LAW_SLOT5_PATCH_VARIANT699_METADATA_RAM_ENABLED {
        return;
    }

    if !law_slot5_id699_preflight_ok() {
        log::write_line(format!(
            "law-slot5-variant699-metadata-ram-patch skipped trigger={trigger} reason=id699_preflight_blocked_or_missing"
        ));
        return;
    }

    let candidates = find_law_variant_metadata_ram_candidates(static_layouts);
    log::write_line(format!(
        "law-variant-metadata-ram-candidates trigger={trigger} selected={} candidates=[{}]",
        candidates
            .first()
            .is_some_and(|candidate| candidate.matched_variants.len() >= LAW_OFFICIAL_LAYOUT_VARIANTS.len()),
        format_law_variant_metadata_ram_candidates(&candidates)
    ));

    let Some(candidate) = candidates
        .first()
        .filter(|candidate| candidate.matched_variants.len() >= LAW_OFFICIAL_LAYOUT_VARIANTS.len())
    else {
        log::write_line(format!(
            "law-slot5-variant699-metadata-ram-patch skipped trigger={trigger} reason=no_valid_metadata_table"
        ));
        return;
    };
    if LAW_LINKDATA_ONLY_RAM_VARIANT699_METADATA_PATCH_DONE.swap(true, Ordering::Relaxed) {
        return;
    }
    LAW_VARIANT_METADATA_RAM_SELECTED_BASE.store(candidate.base, Ordering::Relaxed);
    let address = candidate.target_address;

    let mut before = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
    if !read_exact_process_memory(address, &mut before) {
        log::write_line(format!(
            "law-slot5-variant699-metadata-ram-patch skipped address=0x{address:x} reason=read_before_failed"
        ));
        return;
    }

    let Some(file_raw) = law_linkdata_variant699_metadata_raw() else {
        log::write_line(format!(
            "law-slot5-variant699-metadata-ram-patch skipped trigger={trigger} address=0x{address:x} reason=file_raw_missing before={}",
            format_bytes(&before)
        ));
        return;
    };
    if file_raw.len() != COSTUME_VARIANT_METADATA_COPY_SIZE {
        log::write_line(format!(
            "law-slot5-variant699-metadata-ram-patch skipped trigger={trigger} address=0x{address:x} reason=file_raw_bad_len len={} before={}",
            file_raw.len(),
            format_bytes(&before)
        ));
        return;
    }

    let before_model = u16::from_le_bytes([
        before[COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET],
        before[COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET + 1],
    ]);
    let before_preview = u16::from_le_bytes([
        before[COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET],
        before[COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET + 1],
    ]);
    let before_flags = before[COSTUME_VARIANT_METADATA_FLAGS_OFFSET];
    if before_model == LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID
        && before_preview == LAW_EXTRA_SLOT_PREVIEW_TABLE_MAPPING_SOURCE_ID as u16
        && before.as_slice() == file_raw
    {
        log::write_line(format!(
            "law-slot5-variant699-metadata-ram-patch already_present trigger={trigger} address=0x{address:x} data={}",
            format_bytes(&before)
        ));
        return;
    }
    if before_model != u16::MAX || before_preview != u16::MAX {
        log::write_line(format!(
            "law-slot5-variant699-metadata-ram-patch skipped trigger={trigger} address=0x{address:x} reason=target_not_empty before_model={} before_preview={} before_flags={} before={}",
            before_model,
            before_preview,
            format_optional_u8(Some(before_flags)),
            format_bytes(&before)
        ));
        return;
    }

    let wrote_full =
        matches!(win::write_process_memory(address, file_raw), Some(written) if written == file_raw.len());

    let mut after = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
    let after_read = read_exact_process_memory(address, &mut after);
    let after_matches = after_read && after.as_slice() == file_raw;
    let after_model = after_read.then(|| {
        u16::from_le_bytes([
            after[COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET],
            after[COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET + 1],
        ])
    });
    let after_preview = after_read.then(|| {
        u16::from_le_bytes([
            after[COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET],
            after[COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET + 1],
        ])
    });

    log::write_line(format!(
        "law-slot5-variant699-metadata-ram-patch trigger={trigger} address=0x{address:x} before=[model={before_model} preview={before_preview} flags={}] after=[model={} preview={} flags={}] before_raw={} source={} after_raw={} wrote_full={} match={} reason=copy_full_linkdata_variant699_metadata",
        format_optional_u8(Some(before_flags)),
        format_optional_u16_decimal(after_model),
        format_optional_u16_decimal(after_preview),
        if after_read {
            format_optional_u8(Some(after[COSTUME_VARIANT_METADATA_FLAGS_OFFSET]))
        } else {
            "read_failed".to_string()
        },
        format_bytes(&before),
        format_bytes(file_raw),
        if after_read {
            format_bytes(&after)
        } else {
            "read_failed".to_string()
        },
        wrote_full,
        after_matches
    ));
}

fn maybe_patch_law_linkdata_only_runtime_unlock_slot(trigger: &str) {
    if !LAW_SLOT5_PATCH_RUNTIME_UNLOCK_SLOT_LINKDATA_ENABLED {
        return;
    }
    if LAW_LINKDATA_ONLY_RUNTIME_UNLOCK_SLOT_PATCH_DONE.load(Ordering::Relaxed) {
        return;
    }

    let attempt = LAW_LINKDATA_ONLY_RUNTIME_UNLOCK_SLOT_PATCH_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
    if attempt >= 16 {
        if attempt == 16 {
            log::write_line(format!(
                "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} reason=attempts_exhausted"
            ));
        }
        return;
    }

    let mut memory = ProcessMemoryReader;
    match costume_table::dump_costume_layout_table(win::main_module() as usize, &mut memory) {
        Ok(dump) => maybe_patch_law_linkdata_only_runtime_unlock_slot_with_dump(&dump, trigger),
        Err(error) => {
            if attempt < 4 {
                log::write_line(format!(
                    "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} reason=layout_dump_failed error={error:?}"
                ));
            }
        }
    }
}

fn maybe_patch_law_linkdata_only_runtime_unlock_slot_with_dump(
    dump: &CostumeLayoutTableDump,
    trigger: &str,
) {
    if !LAW_SLOT5_PATCH_RUNTIME_UNLOCK_SLOT_LINKDATA_ENABLED {
        return;
    }
    if LAW_LINKDATA_ONLY_RUNTIME_UNLOCK_SLOT_PATCH_DONE.load(Ordering::Relaxed) {
        return;
    }
    if !law_slot5_id699_preflight_ok() {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} reason=id699_preflight_blocked_or_missing"
        ));
        return;
    }

    let Some(layout) = dump
        .layouts
        .iter()
        .find(|layout| layout.layout_id == LAW_MASTER_LAYOUT_ID)
    else {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} reason=layout26_missing"
        ));
        return;
    };
    let slot4 = layout
        .variants
        .get(LAW_DUPLICATE_VARIANT_SLOT_INDEX)
        .copied();
    if layout.active_variant_count != 5 || slot4 != Some(LAW_CUSTOM_LAYOUT_ID) {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} reason=layout_not_custom active_count={} slot4={}",
            layout.active_variant_count,
            format_optional_u16_decimal(slot4)
        ));
        return;
    }

    let metadata699_matches_linkdata =
        read_variant_metadata_raw(LAW_CUSTOM_LAYOUT_ID).is_some_and(|raw| {
            law_linkdata_variant699_metadata_raw().is_some_and(|file_raw| raw.as_slice() == file_raw)
        });
    if !metadata699_matches_linkdata {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} reason=metadata699_not_linkdata model={} preview={} flags={} raw={} file_raw={}",
            format_optional_u16_decimal(read_variant_model_resource(LAW_CUSTOM_LAYOUT_ID)),
            format_optional_u16_decimal(read_variant_preview_mapping(LAW_CUSTOM_LAYOUT_ID)),
            format_optional_u8(read_variant_flags(LAW_CUSTOM_LAYOUT_ID)),
            read_variant_metadata_raw(LAW_CUSTOM_LAYOUT_ID)
                .map(|raw| format_bytes(&raw))
                .unwrap_or_else(|| "read_failed".to_string()),
            law_linkdata_variant699_metadata_raw()
                .map(format_bytes)
                .unwrap_or_else(|| "missing".to_string())
        ));
        return;
    }

    if patch_law_linkdata_only_runtime_unlock_slot(dump, trigger) {
        LAW_LINKDATA_ONLY_RUNTIME_UNLOCK_SLOT_PATCH_DONE.store(true, Ordering::Relaxed);
    }
}

fn patch_law_linkdata_only_runtime_unlock_slot(
    dump: &CostumeLayoutTableDump,
    trigger: &str,
) -> bool {
    let source_slot = LAW_EXTRA_SLOT_SOURCE_SLOT_INDEX;
    let target_slot = LAW_DUPLICATE_VARIANT_SLOT_INDEX;
    let Some(source_address) = runtime_category_slot_flag_address_for_dump(
        dump,
        LAW_MASTER_CATEGORY_ID as u16,
        source_slot,
    ) else {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} category={} source_slot={} target_slot={} reason=source_address_unavailable",
            LAW_MASTER_CATEGORY_ID, source_slot, target_slot
        ));
        return false;
    };
    let Some(target_address) = runtime_category_slot_flag_address_for_dump(
        dump,
        LAW_MASTER_CATEGORY_ID as u16,
        target_slot,
    ) else {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} category={} source_slot={} target_slot={} reason=target_address_unavailable",
            LAW_MASTER_CATEGORY_ID, source_slot, target_slot
        ));
        return false;
    };

    let mut source_bytes = [0u8; 1];
    if !read_exact_process_memory(source_address, &mut source_bytes) {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} reason=read_source_failed",
            LAW_MASTER_CATEGORY_ID, source_slot, target_slot, source_address, target_address
        ));
        return false;
    }

    let mut before_bytes = [0u8; 1];
    if !read_exact_process_memory(target_address, &mut before_bytes) {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch skipped trigger={trigger} category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} reason=read_target_failed",
            LAW_MASTER_CATEGORY_ID, source_slot, target_slot, source_address, target_address
        ));
        return false;
    }

    let target_value = custom_runtime_unlock_slot_flags(
        source_bytes[0],
        LAW_EXTRA_SLOT_CUSTOM_NON_DLC_ADMISSION_ENABLED,
    );
    if before_bytes[0] == target_value {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch already_present trigger={trigger} category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} source_value=0x{:02x} value=0x{:02x}",
            LAW_MASTER_CATEGORY_ID,
            source_slot,
            target_slot,
            source_address,
            target_address,
            source_bytes[0],
            before_bytes[0]
        ));
        return true;
    }

    let patch_bytes = [target_value];
    if !matches!(win::write_process_memory(target_address, &patch_bytes), Some(written) if written == patch_bytes.len())
    {
        log::write_line(format!(
            "law-slot5-runtime-unlock-slot-patch failed trigger={trigger} category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} before=0x{:02x} source_value=0x{:02x} target_value=0x{:02x} reason=write_failed",
            LAW_MASTER_CATEGORY_ID,
            source_slot,
            target_slot,
            source_address,
            target_address,
            before_bytes[0],
            source_bytes[0],
            target_value
        ));
        return false;
    }

    let mut after_bytes = [0u8; 1];
    let after_matches = read_exact_process_memory(target_address, &mut after_bytes)
        && after_bytes[0] == target_value;
    log::write_line(format!(
        "law-slot5-runtime-unlock-slot-patch trigger={trigger} category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} before=0x{:02x} source_value=0x{:02x} target_value=0x{:02x} after=0x{:02x} match={}",
        LAW_MASTER_CATEGORY_ID,
        source_slot,
        target_slot,
        source_address,
        target_address,
        before_bytes[0],
        source_bytes[0],
        target_value,
        after_bytes[0],
        after_matches
    ));
    after_matches
}

fn patch_law_private_model_ram_clone() -> bool {
    if LAW_PRIVATE_MODEL_RAM_CLONE_DONE.load(Ordering::Relaxed) {
        return true;
    }
    let Some(plan) = law_private_model_ram_plan() else {
        log::write_line(format!(
            "Law private model RAM clone skipped source_row={} target_row={} reason=plan_unavailable",
            LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID,
            LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID
        ));
        return false;
    };
    if plan.source_row_id != LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID
        || plan.target_row_id != LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID
    {
        log::write_line(format!(
            "Law private model RAM clone skipped source_row={} target_row={} reason=plan_mismatch expected_source={} expected_target={}",
            plan.source_row_id,
            plan.target_row_id,
            LAW_EXTRA_SLOT_PRIVATE_MODEL_SOURCE_RESOURCE_ID,
            LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID
        ));
        return false;
    }
    if !patch_law_private_model_entry35_row(&plan) {
        return false;
    }
    if !patch_law_private_model_entry32_name(&plan) {
        return false;
    }
    LAW_PRIVATE_MODEL_RAM_CLONE_DONE.store(true, Ordering::Relaxed);
    true
}

fn law_extra_slot_model_resource_for_patch(
    private_model_ready: bool,
    manager_alias_ready: bool,
) -> u16 {
    let _ = (private_model_ready, manager_alias_ready);
    LAW_EXTRA_SLOT_PRIVATE_MODEL_TARGET_RESOURCE_ID
}

fn law_private_model_ram_plan() -> Option<LawPrivateModelRamPlan> {
    let plan = LAW_PRIVATE_MODEL_RAM_PLAN.get()?;
    let guard = plan.lock().ok()?;
    guard.clone()
}

fn patch_law_private_model_entry35_row(plan: &LawPrivateModelRamPlan) -> bool {
    if plan.source_row.len() != LINKDATA_ENTRY35_MODEL_ROW_STRIDE {
        log::write_line(format!(
            "Law private model row clone skipped source_row={} target_row={} reason=bad_source_len len=0x{:x}",
            plan.source_row_id,
            plan.target_row_id,
            plan.source_row.len()
        ));
        return false;
    }
    let Some(base) = find_entry35_model_table_base(plan) else {
        log::write_line(format!(
            "Law private model row clone skipped source_row={} target_row={} reason=entry35_base_not_found",
            plan.source_row_id, plan.target_row_id
        ));
        return false;
    };
    let Some(target_address) =
        entry35_row_start(plan.target_row_id).and_then(|offset| base.checked_add(offset))
    else {
        log::write_line(format!(
            "Law private model row clone skipped source_row={} target_row={} reason=target_address_overflow",
            plan.source_row_id, plan.target_row_id
        ));
        return false;
    };

    let mut before = vec![0u8; LINKDATA_ENTRY35_MODEL_ROW_STRIDE];
    if !read_exact_process_memory(target_address, &mut before) {
        log::write_line(format!(
            "Law private model row clone failed source_row={} target_row={} target=0x{target_address:x} error=read_target_failed",
            plan.source_row_id, plan.target_row_id
        ));
        return false;
    }
    if before == plan.source_row {
        log::write_line(format!(
            "Law private model row clone already present source_row={} target_row={} entry35_base=0x{base:x} target=0x{target_address:x}",
            plan.source_row_id, plan.target_row_id
        ));
        return true;
    }
    if !matches!(
        win::write_process_memory(target_address, &plan.source_row),
        Some(written) if written == plan.source_row.len()
    ) {
        log::write_line(format!(
            "Law private model row clone failed source_row={} target_row={} target=0x{target_address:x} error=write_failed",
            plan.source_row_id, plan.target_row_id
        ));
        return false;
    }
    let mut after = vec![0u8; LINKDATA_ENTRY35_MODEL_ROW_STRIDE];
    let after_matches =
        read_exact_process_memory(target_address, &mut after) && after == plan.source_row;
    log::write_line(format!(
        "Law private model row clone source_row={} target_row={} entry35_base=0x{base:x} target=0x{target_address:x} before={} after={} match={after_matches}",
        plan.source_row_id,
        plan.target_row_id,
        format_bytes(&before),
        format_bytes(&after)
    ));
    after_matches
}

fn patch_law_private_model_entry32_name(plan: &LawPrivateModelRamPlan) -> bool {
    let Some(patch) = plan.entry32_name_patch.as_ref() else {
        log::write_line(format!(
            "Law private model name patch skipped target_row={} reason=no_entry32_patch",
            plan.target_row_id
        ));
        return true;
    };
    if patch.from.len() != patch.to.len() || patch.from.is_empty() {
        log::write_line(format!(
            "Law private model name patch skipped target_row={} reason=bad_patch_len from=0x{:x} to=0x{:x}",
            plan.target_row_id,
            patch.from.len(),
            patch.to.len()
        ));
        return false;
    }
    if let Some(address) = find_entry32_name_patch_address(&patch.to, patch) {
        log::write_line(format!(
            "Law private model name patch already present target_row={} address=0x{address:x}",
            plan.target_row_id
        ));
        return true;
    }
    let Some(address) = find_entry32_name_patch_address(&patch.from, patch) else {
        log::write_line(format!(
            "Law private model name patch skipped target_row={} reason=entry32_target_not_found",
            plan.target_row_id
        ));
        return false;
    };
    if !matches!(
        win::write_process_memory(address, &patch.to),
        Some(written) if written == patch.to.len()
    ) {
        log::write_line(format!(
            "Law private model name patch failed target_row={} address=0x{address:x} error=write_failed",
            plan.target_row_id
        ));
        return false;
    }
    let mut after = vec![0u8; patch.to.len()];
    let after_matches = read_exact_process_memory(address, &mut after) && after == patch.to;
    log::write_line(format!(
        "Law private model name patch target_row={} address=0x{address:x} from={} to={} after={} match={after_matches}",
        plan.target_row_id,
        format_bytes(&patch.from),
        format_bytes(&patch.to),
        format_bytes(&after)
    ));
    after_matches
}

fn find_entry35_model_table_base(plan: &LawPrivateModelRamPlan) -> Option<usize> {
    if plan.source_row.is_empty() {
        return None;
    }
    for region in win::writable_memory_regions() {
        let mut offset = 0usize;
        let overlap = plan.source_row.len().saturating_sub(1);
        while offset < region.size {
            let read_len = LINKDATA_RAM_SCAN_CHUNK_SIZE.min(region.size - offset);
            let mut buffer = vec![0u8; read_len];
            let read = win::read_process_memory(region.base + offset, &mut buffer).unwrap_or(0);
            if read == 0 {
                break;
            }
            buffer.truncate(read);
            for position in find_all_bytes(&buffer, &plan.source_row) {
                let address = region.base + offset + position;
                let Some(base) = entry35_base_from_source_address(address, plan.source_row_id)
                else {
                    continue;
                };
                if entry35_memory_candidate_matches(base, plan) {
                    return Some(base);
                }
            }
            if offset + read >= region.size {
                break;
            }
            let advance = read.saturating_sub(overlap).max(1);
            offset = offset.saturating_add(advance);
        }
    }
    None
}

fn entry35_memory_candidate_matches(base: usize, plan: &LawPrivateModelRamPlan) -> bool {
    let source_matches = entry35_row_matches(base, plan.source_row_id, &plan.source_row);
    source_matches
        && plan
            .validation_rows
            .iter()
            .all(|row| entry35_row_matches(base, row.row_id, &row.bytes))
}

fn entry35_row_matches(base: usize, row_id: u16, expected: &[u8]) -> bool {
    let Some(address) = entry35_row_start(row_id).and_then(|offset| base.checked_add(offset))
    else {
        return false;
    };
    let mut bytes = vec![0u8; expected.len()];
    read_exact_process_memory(address, &mut bytes) && bytes == expected
}

fn find_entry32_name_patch_address(needle: &[u8], patch: &LawEntry32NamePatch) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    for region in win::writable_memory_regions() {
        let mut offset = 0usize;
        let overlap = needle.len().saturating_sub(1);
        while offset < region.size {
            let read_len = LINKDATA_RAM_SCAN_CHUNK_SIZE.min(region.size - offset);
            let mut buffer = vec![0u8; read_len];
            let read = win::read_process_memory(region.base + offset, &mut buffer).unwrap_or(0);
            if read == 0 {
                break;
            }
            buffer.truncate(read);
            for position in find_all_bytes(&buffer, needle) {
                let address = region.base + offset + position;
                if entry32_name_patch_address_is_coherent(address, patch) {
                    return Some(address);
                }
            }
            if offset + read >= region.size {
                break;
            }
            let advance = read.saturating_sub(overlap).max(1);
            offset = offset.saturating_add(advance);
        }
    }
    None
}

fn entry32_name_patch_address_is_coherent(address: usize, patch: &LawEntry32NamePatch) -> bool {
    let Some(base) = address.checked_sub(patch.offset) else {
        return false;
    };
    patch.anchors.iter().all(|anchor| {
        let Some(anchor_address) = base.checked_add(anchor.offset) else {
            return false;
        };
        let mut bytes = vec![0u8; anchor.bytes.len()];
        read_exact_process_memory(anchor_address, &mut bytes) && bytes == anchor.bytes
    })
}

pub(crate) fn padded_nul_terminated_name_patch(from: &[u8], target: &str) -> Option<Vec<u8>> {
    if from.is_empty() || !from.contains(&0) {
        return None;
    }
    let mut to = target.as_bytes().to_vec();
    to.push(0);
    if to.len() > from.len() {
        return None;
    }
    to.resize(from.len(), 0);
    Some(to)
}

fn entry35_row_start(row_id: u16) -> Option<usize> {
    LINKDATA_ENTRY35_MODEL_ROW_BASE
        .checked_add(usize::from(row_id).checked_mul(LINKDATA_ENTRY35_MODEL_ROW_STRIDE)?)
}

fn entry35_base_from_source_address(source_address: usize, source_row_id: u16) -> Option<usize> {
    source_address.checked_sub(entry35_row_start(source_row_id)?)
}

fn find_all_bytes(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return Vec::new();
    }
    haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index))
        .collect()
}

fn patch_law_extra_slot_metadata_clone(
    dump: &CostumeLayoutTableDump,
    allocation: CustomVariantAllocation,
) -> bool {
    let Some(clone) = law_extra_slot_metadata_clone_for_allocation(dump, allocation) else {
        log::write_line(format!(
            "Law extra slot metadata clone skipped source_layout={} target_layout={} error=address_unavailable",
            allocation.metadata_source_layout_id, allocation.allocated_variant_id
        ));
        return false;
    };

    let mut source_bytes = [0u8; COSTUME_LAYOUT_ROW_COPY_SIZE];
    if !read_exact_process_memory(clone.source_address, &mut source_bytes) {
        log::write_line(format!(
            "Law extra slot metadata clone failed source_layout={} target_layout={} source=0x{:x} target=0x{:x} bytes={} error=read_source_failed",
            clone.source_layout_id,
            clone.target_layout_id,
            clone.source_address,
            clone.target_address,
            clone.byte_count
        ));
        return false;
    }

    let mut before_bytes = [0u8; COSTUME_LAYOUT_ROW_COPY_SIZE];
    if !read_exact_process_memory(clone.target_address, &mut before_bytes) {
        log::write_line(format!(
            "Law extra slot metadata clone failed source_layout={} target_layout={} source=0x{:x} target=0x{:x} bytes={} error=read_target_failed",
            clone.source_layout_id,
            clone.target_layout_id,
            clone.source_address,
            clone.target_address,
            clone.byte_count
        ));
        return false;
    }

    if before_bytes == source_bytes {
        log::write_line(format!(
            "Law extra slot metadata clone already present source_layout={} target_layout={} source=0x{:x} target=0x{:x} bytes={} data={}",
            clone.source_layout_id,
            clone.target_layout_id,
            clone.source_address,
            clone.target_address,
            clone.byte_count,
            format_bytes(&source_bytes)
        ));
        return true;
    }

    if !matches!(win::write_process_memory(clone.target_address, &source_bytes), Some(written) if written == source_bytes.len())
    {
        log::write_line(format!(
            "Law extra slot metadata clone failed source_layout={} target_layout={} source=0x{:x} target=0x{:x} bytes={} error=write_failed",
            clone.source_layout_id,
            clone.target_layout_id,
            clone.source_address,
            clone.target_address,
            clone.byte_count
        ));
        return false;
    }

    let mut after_bytes = [0u8; COSTUME_LAYOUT_ROW_COPY_SIZE];
    let after_matches = read_exact_process_memory(clone.target_address, &mut after_bytes)
        && after_bytes == source_bytes;
    log::write_line(format!(
        "Law extra slot metadata clone source_layout={} target_layout={} source=0x{:x} target=0x{:x} bytes={} before={} source_data={} after={} match={}",
        clone.source_layout_id,
        clone.target_layout_id,
        clone.source_address,
        clone.target_address,
        clone.byte_count,
        format_bytes(&before_bytes),
        format_bytes(&source_bytes),
        format_bytes(&after_bytes),
        after_matches
    ));
    after_matches
}

fn patch_law_extra_slot_variant_metadata_clone(
    dump: &CostumeLayoutTableDump,
    allocation: CustomVariantAllocation,
) -> bool {
    let source_variant_id = allocation.metadata_source_variant_id;
    let Some(source_address) =
        costume_variant_metadata_record_address(dump.static_layouts, source_variant_id)
    else {
        log::write_line(format!(
            "Law extra slot variant metadata clone skipped source_variant={} target_variant={} error=source_address_unavailable",
            source_variant_id, allocation.allocated_variant_id
        ));
        return false;
    };
    let Some(target_address) = costume_variant_metadata_record_address(
        dump.static_layouts,
        allocation.allocated_variant_id,
    ) else {
        log::write_line(format!(
            "Law extra slot variant metadata clone skipped source_variant={} target_variant={} error=target_address_unavailable",
            source_variant_id, allocation.allocated_variant_id
        ));
        return false;
    };

    let mut source_bytes = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
    if !read_exact_process_memory(source_address, &mut source_bytes) {
        log::write_line(format!(
            "Law extra slot variant metadata clone failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} error=read_source_failed",
            source_variant_id,
            allocation.allocated_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_COPY_SIZE
        ));
        return false;
    }

    let mut before_bytes = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
    if !read_exact_process_memory(target_address, &mut before_bytes) {
        log::write_line(format!(
            "Law extra slot variant metadata clone failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} error=read_target_failed",
            source_variant_id,
            allocation.allocated_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_COPY_SIZE
        ));
        return false;
    }

    if before_bytes == source_bytes {
        log::write_line(format!(
            "Law extra slot variant metadata clone already present source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} data={}",
            source_variant_id,
            allocation.allocated_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_COPY_SIZE,
            format_bytes(&source_bytes)
        ));
        return true;
    }

    if !matches!(win::write_process_memory(target_address, &source_bytes), Some(written) if written == source_bytes.len())
    {
        log::write_line(format!(
            "Law extra slot variant metadata clone failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} error=write_failed",
            source_variant_id,
            allocation.allocated_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_COPY_SIZE
        ));
        return false;
    }

    let mut after_bytes = [0u8; COSTUME_VARIANT_METADATA_COPY_SIZE];
    let after_matches =
        read_exact_process_memory(target_address, &mut after_bytes) && after_bytes == source_bytes;
    log::write_line(format!(
        "Law extra slot variant metadata clone source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} before={} source_data={} after={} match={}",
        source_variant_id,
        allocation.allocated_variant_id,
        source_address,
        target_address,
        COSTUME_VARIANT_METADATA_COPY_SIZE,
        format_bytes(&before_bytes),
        format_bytes(&source_bytes),
        format_bytes(&after_bytes),
        after_matches
    ));
    after_matches
}

fn patch_law_extra_slot_variant_metadata_flags(
    dump: &CostumeLayoutTableDump,
    allocation: CustomVariantAllocation,
) -> bool {
    let Some(address) = costume_variant_metadata_record_address(
        dump.static_layouts,
        allocation.allocated_variant_id,
    )
    .and_then(|base| base.checked_add(COSTUME_VARIANT_METADATA_FLAGS_OFFSET)) else {
        log::write_line(format!(
            "Law extra slot variant metadata flags patch skipped target_variant={} error=address_unavailable",
            allocation.allocated_variant_id
        ));
        return false;
    };

    let mut before_bytes = [0u8; 1];
    if !read_exact_process_memory(address, &mut before_bytes) {
        log::write_line(format!(
            "Law extra slot variant metadata flags patch failed target_variant={} address=0x{:x} error=read_before_failed",
            allocation.allocated_variant_id, address
        ));
        return false;
    }

    let target = custom_variant_metadata_flags(before_bytes[0]);
    if target == before_bytes[0] {
        log::write_line(format!(
            "Law extra slot variant metadata flags patch already present target_variant={} address=0x{:x} value=0x{:02x}",
            allocation.allocated_variant_id, address, before_bytes[0]
        ));
        return true;
    }

    let patch_bytes = [target];
    if !matches!(win::write_process_memory(address, &patch_bytes), Some(written) if written == patch_bytes.len())
    {
        log::write_line(format!(
            "Law extra slot variant metadata flags patch failed target_variant={} address=0x{:x} before=0x{:02x} target=0x{:02x} error=write_failed",
            allocation.allocated_variant_id, address, before_bytes[0], target
        ));
        return false;
    }

    let mut after_bytes = [0u8; 1];
    let after_matches =
        read_exact_process_memory(address, &mut after_bytes) && after_bytes[0] == target;
    log::write_line(format!(
        "Law extra slot variant metadata flags patch target_variant={} address=0x{:x} before=0x{:02x} after=0x{:02x} target=0x{:02x} match={}",
        allocation.allocated_variant_id,
        address,
        before_bytes[0],
        after_bytes[0],
        target,
        after_matches
    ));
    after_matches
}

fn patch_law_extra_slot_variant_model_resource(
    dump: &CostumeLayoutTableDump,
    allocation: CustomVariantAllocation,
    target: u16,
) -> bool {
    let Some(address) = costume_variant_metadata_record_address(
        dump.static_layouts,
        allocation.allocated_variant_id,
    )
    .and_then(|base| base.checked_add(COSTUME_VARIANT_METADATA_MODEL_RESOURCE_OFFSET)) else {
        log::write_line(format!(
            "Law extra slot model resource patch skipped target_variant={} error=address_unavailable",
            allocation.allocated_variant_id
        ));
        return false;
    };
    let mut before_bytes = [0u8; size_of::<u16>()];
    if !read_exact_process_memory(address, &mut before_bytes) {
        log::write_line(format!(
            "Law extra slot model resource patch failed target_variant={} address=0x{:x} error=read_before_failed",
            allocation.allocated_variant_id, address
        ));
        return false;
    }
    let before = u16::from_le_bytes(before_bytes);
    if before == target {
        log::write_line(format!(
            "Law extra slot model resource patch already present target_variant={} address=0x{:x} value={}",
            allocation.allocated_variant_id, address, before
        ));
        return true;
    }
    if !matches!(
        win::write_process_memory(address, &target.to_le_bytes()),
        Some(written) if written == size_of::<u16>()
    ) {
        log::write_line(format!(
            "Law extra slot model resource patch failed target_variant={} address=0x{:x} before={} target={} error=write_failed",
            allocation.allocated_variant_id, address, before, target
        ));
        return false;
    }
    let mut after_bytes = [0u8; size_of::<u16>()];
    let after = if read_exact_process_memory(address, &mut after_bytes) {
        u16::from_le_bytes(after_bytes)
    } else {
        u16::MAX
    };
    log::write_line(format!(
        "Law extra slot model resource patch target_variant={} address=0x{:x} before={} after={} target={} match={}",
        allocation.allocated_variant_id,
        address,
        before,
        after,
        target,
        after == target
    ));
    after == target
}

fn preview_model_mapped_id_from_metadata_value(value: u16) -> u32 {
    if value < 0x1b6 {
        u32::from(value) + 0x269
    } else {
        0x1269
    }
}

fn patch_law_extra_slot_variant_preview_mapping(
    dump: &CostumeLayoutTableDump,
    allocation: CustomVariantAllocation,
) -> bool {
    let source_variant_id = LAW_EXTRA_SLOT_PREVIEW_MAPPING_SOURCE_VARIANT_ID;
    let target_variant_id = allocation.allocated_variant_id;
    let Some(source_address) =
        costume_variant_metadata_record_address(dump.static_layouts, source_variant_id)
            .and_then(|base| base.checked_add(COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET))
    else {
        log::write_line(format!(
            "Law extra slot preview mapping patch skipped source_variant={} target_variant={} error=source_address_unavailable",
            source_variant_id, target_variant_id
        ));
        return false;
    };
    let Some(target_address) =
        costume_variant_metadata_record_address(dump.static_layouts, target_variant_id)
            .and_then(|base| base.checked_add(COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_OFFSET))
    else {
        log::write_line(format!(
            "Law extra slot preview mapping patch skipped source_variant={} target_variant={} error=target_address_unavailable",
            source_variant_id, target_variant_id
        ));
        return false;
    };

    let mut source_bytes = [0u8; COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE];
    if !read_exact_process_memory(source_address, &mut source_bytes) {
        log::write_line(format!(
            "Law extra slot preview mapping patch failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} error=read_source_failed",
            source_variant_id,
            target_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE
        ));
        return false;
    }

    let mut before_bytes = [0u8; COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE];
    if !read_exact_process_memory(target_address, &mut before_bytes) {
        log::write_line(format!(
            "Law extra slot preview mapping patch failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} error=read_target_failed",
            source_variant_id,
            target_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE
        ));
        return false;
    }

    let source_value = u16::from_le_bytes(source_bytes);
    let before_value = u16::from_le_bytes(before_bytes);
    if before_bytes == source_bytes {
        log::write_line(format!(
            "Law extra slot preview mapping patch already present source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} value={} mapped={}",
            source_variant_id,
            target_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE,
            source_value,
            preview_model_mapped_id_from_metadata_value(source_value)
        ));
        return true;
    }

    if !matches!(win::write_process_memory(target_address, &source_bytes), Some(written) if written == source_bytes.len())
    {
        log::write_line(format!(
            "Law extra slot preview mapping patch failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} before={} before_mapped={} source_value={} source_mapped={} error=write_failed",
            source_variant_id,
            target_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE,
            before_value,
            preview_model_mapped_id_from_metadata_value(before_value),
            source_value,
            preview_model_mapped_id_from_metadata_value(source_value)
        ));
        return false;
    }

    let mut after_bytes = [0u8; COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE];
    let after_matches =
        read_exact_process_memory(target_address, &mut after_bytes) && after_bytes == source_bytes;
    let after_value = u16::from_le_bytes(after_bytes);
    log::write_line(format!(
        "Law extra slot preview mapping patch source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} before={} before_mapped={} source_value={} source_mapped={} after={} after_mapped={} match={}",
        source_variant_id,
        target_variant_id,
        source_address,
        target_address,
        COSTUME_VARIANT_METADATA_PREVIEW_MAPPING_COPY_SIZE,
        before_value,
        preview_model_mapped_id_from_metadata_value(before_value),
        source_value,
        preview_model_mapped_id_from_metadata_value(source_value),
        after_value,
        preview_model_mapped_id_from_metadata_value(after_value),
        after_matches
    ));
    after_matches
}

fn patch_law_extra_slot_variant_color_variations(
    dump: &CostumeLayoutTableDump,
    allocation: CustomVariantAllocation,
) -> bool {
    let source_variant_id = LAW_EXTRA_SLOT_COLOR_VARIATION_SOURCE_VARIANT_ID;
    let target_variant_id = allocation.allocated_variant_id;
    let Some(source_address) =
        costume_variant_metadata_record_address(dump.static_layouts, source_variant_id)
            .and_then(|base| base.checked_add(COSTUME_VARIANT_METADATA_COLOR_VARIATION_OFFSET))
    else {
        log::write_line(format!(
            "Law extra slot color variation patch skipped source_variant={} target_variant={} error=source_address_unavailable",
            source_variant_id, target_variant_id
        ));
        return false;
    };
    let Some(target_address) =
        costume_variant_metadata_record_address(dump.static_layouts, target_variant_id)
            .and_then(|base| base.checked_add(COSTUME_VARIANT_METADATA_COLOR_VARIATION_OFFSET))
    else {
        log::write_line(format!(
            "Law extra slot color variation patch skipped source_variant={} target_variant={} error=target_address_unavailable",
            source_variant_id, target_variant_id
        ));
        return false;
    };

    let mut source_bytes = [0u8; COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE];
    if !read_exact_process_memory(source_address, &mut source_bytes) {
        log::write_line(format!(
            "Law extra slot color variation patch failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} error=read_source_failed",
            source_variant_id,
            target_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE
        ));
        return false;
    }

    let mut before_bytes = [0u8; COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE];
    if !read_exact_process_memory(target_address, &mut before_bytes) {
        log::write_line(format!(
            "Law extra slot color variation patch failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} error=read_target_failed",
            source_variant_id,
            target_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE
        ));
        return false;
    }

    if before_bytes == source_bytes {
        log::write_line(format!(
            "Law extra slot color variation patch already present source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} data={}",
            source_variant_id,
            target_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE,
            format_bytes(&source_bytes)
        ));
        return true;
    }

    if !matches!(win::write_process_memory(target_address, &source_bytes), Some(written) if written == source_bytes.len())
    {
        log::write_line(format!(
            "Law extra slot color variation patch failed source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} before={} source_data={} error=write_failed",
            source_variant_id,
            target_variant_id,
            source_address,
            target_address,
            COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE,
            format_bytes(&before_bytes),
            format_bytes(&source_bytes)
        ));
        return false;
    }

    let mut after_bytes = [0u8; COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE];
    let after_matches =
        read_exact_process_memory(target_address, &mut after_bytes) && after_bytes == source_bytes;
    log::write_line(format!(
        "Law extra slot color variation patch source_variant={} target_variant={} source=0x{:x} target=0x{:x} bytes={} before={} source_data={} after={} match={}",
        source_variant_id,
        target_variant_id,
        source_address,
        target_address,
        COSTUME_VARIANT_METADATA_COLOR_VARIATION_COPY_SIZE,
        format_bytes(&before_bytes),
        format_bytes(&source_bytes),
        format_bytes(&after_bytes),
        after_matches
    ));
    after_matches
}

fn patch_law_extra_slot_runtime_unlock_slot(
    dump: &CostumeLayoutTableDump,
    allocation: CustomVariantAllocation,
) {
    let source_slot = LAW_EXTRA_SLOT_SOURCE_SLOT_INDEX;
    let target_slot = allocation.slot_index;
    let Some(source_address) = runtime_category_slot_flag_address_for_dump(
        dump,
        LAW_MASTER_CATEGORY_ID as u16,
        source_slot,
    ) else {
        log::write_line(format!(
            "Law extra slot runtime unlock slot clone skipped category={} source_slot={} target_slot={} error=source_address_unavailable",
            LAW_MASTER_CATEGORY_ID, source_slot, target_slot
        ));
        return;
    };
    let Some(target_address) = runtime_category_slot_flag_address_for_dump(
        dump,
        LAW_MASTER_CATEGORY_ID as u16,
        target_slot,
    ) else {
        log::write_line(format!(
            "Law extra slot runtime unlock slot clone skipped category={} source_slot={} target_slot={} error=target_address_unavailable",
            LAW_MASTER_CATEGORY_ID, source_slot, target_slot
        ));
        return;
    };

    let mut source_bytes = [0u8; 1];
    if !read_exact_process_memory(source_address, &mut source_bytes) {
        log::write_line(format!(
            "Law extra slot runtime unlock slot clone failed category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} error=read_source_failed",
            LAW_MASTER_CATEGORY_ID, source_slot, target_slot, source_address, target_address
        ));
        return;
    }

    let mut before_bytes = [0u8; 1];
    if !read_exact_process_memory(target_address, &mut before_bytes) {
        log::write_line(format!(
            "Law extra slot runtime unlock slot clone failed category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} error=read_target_failed",
            LAW_MASTER_CATEGORY_ID, source_slot, target_slot, source_address, target_address
        ));
        return;
    }

    let target_value = custom_runtime_unlock_slot_flags(
        source_bytes[0],
        LAW_EXTRA_SLOT_CUSTOM_NON_DLC_ADMISSION_ENABLED,
    );
    if target_value == 0 {
        log::write_line(format!(
            "Law extra slot runtime unlock slot clone skipped category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} source_value=0x00 before=0x{:02x} error=source_zero",
            LAW_MASTER_CATEGORY_ID,
            source_slot,
            target_slot,
            source_address,
            target_address,
            before_bytes[0]
        ));
        return;
    }

    if before_bytes[0] == target_value {
        log::write_line(format!(
            "Law extra slot runtime unlock slot clone already present category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} source_value=0x{:02x} value=0x{:02x}",
            LAW_MASTER_CATEGORY_ID,
            source_slot,
            target_slot,
            source_address,
            target_address,
            source_bytes[0],
            before_bytes[0]
        ));
        return;
    }

    let patch_bytes = [target_value];
    if !matches!(win::write_process_memory(target_address, &patch_bytes), Some(written) if written == patch_bytes.len())
    {
        log::write_line(format!(
            "Law extra slot runtime unlock slot clone failed category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} before=0x{:02x} source_value=0x{:02x} target_value=0x{:02x} error=write_failed",
            LAW_MASTER_CATEGORY_ID,
            source_slot,
            target_slot,
            source_address,
            target_address,
            before_bytes[0],
            source_bytes[0],
            target_value
        ));
        return;
    }

    let mut after_bytes = [0u8; 1];
    let after_matches = read_exact_process_memory(target_address, &mut after_bytes)
        && after_bytes[0] == target_value;
    log::write_line(format!(
        "Law extra slot runtime unlock slot clone category={} source_slot={} target_slot={} source=0x{:x} target=0x{:x} before=0x{:02x} source_value=0x{:02x} target_value=0x{:02x} after=0x{:02x} match={}",
        LAW_MASTER_CATEGORY_ID,
        source_slot,
        target_slot,
        source_address,
        target_address,
        before_bytes[0],
        source_bytes[0],
        target_value,
        after_bytes[0],
        after_matches
    ));
}

fn custom_variant_metadata_flags(flags: u8) -> u8 {
    (flags | COSTUME_VARIANT_METADATA_ENABLED_FLAG) & !COSTUME_VARIANT_METADATA_DLC_ENTITLEMENT_FLAG
}

fn custom_runtime_unlock_slot_flags(flags: u8, custom_non_dlc_admission_enabled: bool) -> u8 {
    if custom_non_dlc_admission_enabled {
        flags | COSTUME_RUNTIME_UNLOCK_DIRECT_FLAG
    } else {
        flags
    }
}

fn patch_law_duplicate_variant_slot(address: usize) -> bool {
    let Some(target_variant) = current_law_extra_slot_probe_variant_id() else {
        log::write_line("Law extra slot variant patch skipped: custom variant id unavailable");
        return false;
    };
    let mut before_bytes = [0u8; 2];
    if !read_exact_process_memory(address, &mut before_bytes) {
        log::write_line(format!(
            "Law extra slot variant patch failed layout={} slot={} address=0x{:x} error=read_before_failed",
            LAW_MASTER_LAYOUT_ID, LAW_DUPLICATE_VARIANT_SLOT_INDEX, address
        ));
        return false;
    }

    let before = u16::from_le_bytes(before_bytes);
    if before == target_variant {
        log::write_line(format!(
            "Law extra slot variant patch already present layout={} slot={} address=0x{:x} value={}",
            LAW_MASTER_LAYOUT_ID, LAW_DUPLICATE_VARIANT_SLOT_INDEX, address, before
        ));
        return true;
    }
    if before != EMPTY_LAYOUT_VARIANT_ID {
        log::write_line(format!(
            "Law extra slot variant patch skipped layout={} slot={} address=0x{:x} before={} expected_empty={}",
            LAW_MASTER_LAYOUT_ID,
            LAW_DUPLICATE_VARIANT_SLOT_INDEX,
            address,
            before,
            EMPTY_LAYOUT_VARIANT_ID
        ));
        return false;
    }

    let patch_bytes = target_variant.to_le_bytes();
    if !matches!(win::write_process_memory(address, &patch_bytes), Some(written) if written == patch_bytes.len())
    {
        log::write_line(format!(
            "Law extra slot variant patch failed layout={} slot={} address=0x{:x} before={} target={} error=write_failed",
            LAW_MASTER_LAYOUT_ID,
            LAW_DUPLICATE_VARIANT_SLOT_INDEX,
            address,
            before,
            target_variant
        ));
        return false;
    }

    let mut after_bytes = [0u8; 2];
    let after = if read_exact_process_memory(address, &mut after_bytes) {
        u16::from_le_bytes(after_bytes)
    } else {
        0
    };
    log::write_line(format!(
        "Law extra slot variant patch layout={} slot={} address=0x{:x} before={} after={} target={}",
        LAW_MASTER_LAYOUT_ID,
        LAW_DUPLICATE_VARIANT_SLOT_INDEX,
        address,
        before,
        after,
        target_variant
    ));
    after == target_variant
}

fn patch_law_duplicate_variant_count(dump: &CostumeLayoutTableDump) {
    let Some(address) = dump.law_duplicate_variant_active_count_address() else {
        log::write_line("Law duplicate variant count patch skipped: address unavailable");
        return;
    };

    let mut before_bytes = [0u8; 1];
    if !read_exact_process_memory(address, &mut before_bytes) {
        log::write_line(format!(
            "Law duplicate variant count patch failed layout={} address=0x{:x} error=read_before_failed",
            LAW_MASTER_LAYOUT_ID, address
        ));
        return;
    }

    let before = before_bytes[0];
    if before >= LAW_DUPLICATE_VARIANT_ACTIVE_COUNT {
        log::write_line(format!(
            "Law duplicate variant count patch already present layout={} address=0x{:x} value={} target={}",
            LAW_MASTER_LAYOUT_ID, address, before, LAW_DUPLICATE_VARIANT_ACTIVE_COUNT
        ));
        return;
    }

    let patch_bytes = [LAW_DUPLICATE_VARIANT_ACTIVE_COUNT];
    if !matches!(win::write_process_memory(address, &patch_bytes), Some(written) if written == patch_bytes.len())
    {
        log::write_line(format!(
            "Law duplicate variant count patch failed layout={} address=0x{:x} before={} target={} error=write_failed",
            LAW_MASTER_LAYOUT_ID, address, before, LAW_DUPLICATE_VARIANT_ACTIVE_COUNT
        ));
        return;
    }

    let mut after_bytes = [0u8; 1];
    let after = if read_exact_process_memory(address, &mut after_bytes) {
        after_bytes[0]
    } else {
        0
    };
    log::write_line(format!(
        "Law duplicate variant count patch layout={} address=0x{:x} before={} after={} target={}",
        LAW_MASTER_LAYOUT_ID, address, before, after, LAW_DUPLICATE_VARIANT_ACTIVE_COUNT
    ));
}

fn log_costume_table_row(row: CostumeRowSnapshot) {
    log::write_line(format!(
        "Global costume row row={} flags=0x{:04x} group={} sort={} cat_a={} cat_b={} marker=0x{:02x} runtime_flags={} runtime_state={} enabled={} hidden={} sort26_candidate={} clone_candidate={}",
        row.row,
        row.flags,
        row.group,
        row.sort_key,
        row.primary_category_a,
        row.primary_category_b,
        row.marker,
        format_optional_u8(row.runtime_flags),
        format_optional_u16(row.runtime_state),
        row.is_enabled(),
        row.is_hidden(),
        row.is_sort26_candidate(),
        row.is_clone_candidate()
    ));
}

fn log_costume_table_row_detail(row: CostumeRowSnapshot) {
    log::write_line(format!(
        "Global costume row detail row={} primary_4c={} primary_b0={} secondary_d7c0={} runtime_1548={}",
        row.row,
        format_bytes(&row.primary_head),
        format_bytes(&row.primary_category_context),
        format_bytes(&row.secondary_context),
        row.runtime_context
            .as_ref()
            .map(|bytes| format_bytes(bytes))
            .unwrap_or_else(|| "none".to_string())
    ));
}

fn log_costume_group_lists(dump: &CostumeTableDump) {
    for group in 0..10u8 {
        let mut rows = dump
            .rows
            .iter()
            .copied()
            .filter(|row| row.is_enabled() && !row.is_hidden() && row.group == group)
            .collect::<Vec<_>>();
        if rows.is_empty() {
            continue;
        }
        rows.sort_by_key(|row| (row.sort_key, row.row));
        log::write_line(format!(
            "Global costume group group={group} visible_rows={}",
            rows.iter()
                .map(|row| format!("{}:{}", row.row, row.sort_key))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
}

fn format_optional_address(value: Option<usize>) -> String {
    value
        .map(|value| format!("0x{value:x}"))
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_usize_decimal(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_i32(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_u32(value: Option<u32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_hex_u32(value: Option<u32>) -> String {
    value
        .map(|value| format!("0x{value:08x}"))
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_u8(value: Option<u8>) -> String {
    value
        .map(|value| format!("0x{value:02x}"))
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_u16(value: Option<u16>) -> String {
    value
        .map(|value| format!("0x{value:04x}"))
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_u16_decimal(value: Option<u16>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn read_u16_from_bytes(bytes: &[u8], offset: usize) -> Option<u16> {
    let raw = bytes.get(offset..offset + size_of::<u16>())?;
    Some(u16::from_le_bytes([raw[0], raw[1]]))
}

fn format_group_counts(counts: &[usize; 10]) -> String {
    counts
        .iter()
        .enumerate()
        .filter(|(_, count)| **count != 0)
        .map(|(group, count)| format!("{group}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn format_u16_list(values: &[u16]) -> String {
    values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn format_u32_list(values: &[u32]) -> String {
    values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn format_costume_variant_owners(owners: &[CostumeVariantOwner]) -> String {
    if owners.is_empty() {
        return "none".to_string();
    }
    owners
        .iter()
        .map(|owner| {
            format!(
                "{}:{}(family={},category={},count={})",
                owner.layout_id,
                owner.slot_index,
                owner.family,
                owner.category,
                owner.active_variant_count
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_bytes(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

struct ProcessMemoryReader;

impl CostumeTableMemory for ProcessMemoryReader {
    fn read_exact(&mut self, address: usize, buffer: &mut [u8]) -> bool {
        read_exact_process_memory(address, buffer)
    }
}

fn read_exact_process_memory(address: usize, buffer: &mut [u8]) -> bool {
    matches!(win::read_process_memory(address, buffer), Some(read) if read == buffer.len())
}

fn read_i32_field(base: usize, offset: usize) -> Option<i32> {
    let mut bytes = [0u8; 4];
    read_exact_process_memory(base.checked_add(offset)?, &mut bytes)
        .then(|| i32::from_le_bytes(bytes))
}

fn read_u32_field(base: usize, offset: usize) -> Option<u32> {
    let mut bytes = [0u8; 4];
    read_exact_process_memory(base.checked_add(offset)?, &mut bytes)
        .then(|| u32::from_le_bytes(bytes))
}

fn read_u16_field(base: usize, offset: usize) -> Option<u16> {
    let mut bytes = [0u8; 2];
    read_exact_process_memory(base.checked_add(offset)?, &mut bytes)
        .then(|| u16::from_le_bytes(bytes))
}

fn read_u8_field(base: usize, offset: usize) -> Option<u8> {
    let mut bytes = [0u8; 1];
    read_exact_process_memory(base.checked_add(offset)?, &mut bytes).then(|| bytes[0])
}

fn read_usize_field(base: usize, offset: usize) -> Option<usize> {
    read_usize_absolute(base.checked_add(offset)?)
}

fn read_u32_pointer_values(pointer: *const u32, count: usize) -> Vec<u32> {
    if pointer.is_null() || count == 0 {
        return Vec::new();
    }

    let mut bytes = vec![0u8; count * size_of::<u32>()];
    if !read_exact_process_memory(pointer as usize, &mut bytes) {
        return Vec::new();
    }

    bytes
        .chunks_exact(size_of::<u32>())
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn read_focus_variant(layout_id: Option<u32>, variant_index: Option<u32>) -> Option<u32> {
    let layout_id = layout_id?;
    let variant_index = variant_index?;
    if layout_id >= COSTUME_LAYOUT_ROW_COUNT || variant_index >= COSTUME_LAYOUT_VARIANT_COUNT {
        return None;
    }

    let address = static_layouts_address()?
        + layout_id as usize * COSTUME_LAYOUT_ROW_STRIDE
        + COSTUME_LAYOUT_VARIANTS_OFFSET
        + variant_index as usize * size_of::<u16>();
    read_u16_absolute(address).map(u32::from)
}

fn find_layout_variant_index(layout_id: u32, variant_id: u16) -> Option<u32> {
    if layout_id >= COSTUME_LAYOUT_ROW_COUNT {
        return None;
    }

    let base = static_layouts_address()?
        + layout_id as usize * COSTUME_LAYOUT_ROW_STRIDE
        + COSTUME_LAYOUT_VARIANTS_OFFSET;
    (0..COSTUME_LAYOUT_VARIANT_COUNT).find(|index| {
        let address = base + *index as usize * size_of::<u16>();
        read_u16_absolute(address) == Some(variant_id)
    })
}

fn read_costume_row_slot_value(row_id: u32, slot_index: u32) -> Option<u16> {
    if row_id >= costume_table::ROW_COUNT as u32 || slot_index as usize >= LAW_MENU_ROW_SLOT_COUNT {
        return None;
    }

    let address = static_rows_address()?
        + row_id as usize * COSTUME_PRIMARY_ROW_STRIDE
        + LAW_MENU_ROW_SLOT_OFFSET
        + slot_index as usize * size_of::<u16>();
    read_u16_absolute(address)
}

fn static_layouts_address() -> Option<usize> {
    let root = static_root_address()?;
    read_usize_absolute(root + COSTUME_STATIC_LAYOUT_TABLE_OFFSET)
}

fn static_rows_address() -> Option<usize> {
    let root = static_root_address()?;
    read_usize_absolute(root + COSTUME_STATIC_ROW_TABLE_OFFSET)
}

fn static_root_address() -> Option<usize> {
    let main_module = win::main_module() as usize;
    if main_module == 0 {
        return None;
    }
    let database = read_usize_absolute(main_module + costume_table::STATIC_DATABASE_RVA)?;
    read_usize_absolute(database + COSTUME_STATIC_ROOT_OFFSET)
}

fn read_usize_absolute(address: usize) -> Option<usize> {
    let mut bytes = [0u8; size_of::<usize>()];
    read_exact_process_memory(address, &mut bytes).then(|| usize::from_le_bytes(bytes))
}

fn read_u32_absolute(address: usize) -> Option<u32> {
    let mut bytes = [0u8; size_of::<u32>()];
    read_exact_process_memory(address, &mut bytes).then(|| u32::from_le_bytes(bytes))
}

fn read_u16_absolute(address: usize) -> Option<u16> {
    let mut bytes = [0u8; size_of::<u16>()];
    read_exact_process_memory(address, &mut bytes).then(|| u16::from_le_bytes(bytes))
}

fn read_u8_absolute(address: usize) -> Option<u8> {
    let mut bytes = [0u8; 1];
    read_exact_process_memory(address, &mut bytes).then_some(bytes[0])
}

fn log_dlc_stack_result(
    path: &str,
    desired_access: Dword,
    creation_disposition: Dword,
    handle: Handle,
    last_error: Dword,
) {
    if !TRACE_DLC_STACKS.load(Ordering::Relaxed) {
        return;
    }
    let Some(kind) = dlc_stack_path_kind(path) else {
        return;
    };
    let index = DLC_STACK_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_DLC_STACK_LOGS {
        if index == MAX_DLC_STACK_LOGS {
            log::write_line("DLC/save stack trace logs suppressed".to_string());
        }
        return;
    }
    let frames = capture_stack_trace();
    let stack = frames
        .iter()
        .copied()
        .map(format_stack_frame)
        .collect::<Vec<_>>()
        .join(" ");
    let result = if handle == INVALID_HANDLE_VALUE {
        format!("missing error={last_error}")
    } else {
        format!("ok handle=0x{:x}", handle as usize)
    };
    log::write_line(format!(
        "CreateFileW result kind={kind} {result} path={path} access=0x{desired_access:x} disposition=0x{creation_disposition:x} frames={stack}"
    ));
    log_stack_code_contexts(kind, &frames);
}

fn log_find_first_result(api: &str, path: &str, handle: Handle, find_file_data: Lpvoid) {
    if !should_trace_dlc_discovery_path(path) {
        return;
    }
    if handle != INVALID_HANDLE_VALUE {
        remember_find_pattern(handle, path.to_string());
    }
    let result = if handle == INVALID_HANDLE_VALUE {
        format!("missing error={}", unsafe { GetLastError() })
    } else {
        format!("ok handle=0x{:x}", handle as usize)
    };
    let found_name = find_data_name(find_file_data.cast());
    log_dlc_discovery_event(api, path, result, found_name.as_deref());
}

fn log_dlc_discovery_event(api: &str, path: &str, result: String, found_name: Option<&str>) {
    if !TRACE_DLC_STACKS.load(Ordering::Relaxed) || !should_trace_dlc_discovery_path(path) {
        return;
    }
    let index = DLC_STACK_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= MAX_DLC_STACK_LOGS {
        if index == MAX_DLC_STACK_LOGS {
            log::write_line("DLC discovery logs suppressed".to_string());
        }
        return;
    }
    let frames = capture_stack_trace();
    let stack = frames
        .iter()
        .copied()
        .map(format_stack_frame)
        .collect::<Vec<_>>()
        .join(" ");
    let found = found_name
        .map(|name| format!(" found={name}"))
        .unwrap_or_default();
    log::write_line(format!(
        "DLC discovery {api} {result}{found} path={path} frames={stack}"
    ));
    log_stack_code_contexts(api, &frames);
}

fn dlc_stack_path_kind(path: &str) -> Option<&'static str> {
    let lower = path.replace('/', "\\").to_ascii_lowercase();
    if lower.contains("\\linkdata\\cmn\\linkdata_a.bin") {
        return Some("linkdata_a");
    }
    if lower.contains("\\file\\dlc\\dlc_costume_") {
        return Some("dlc_costume");
    }
    if lower.contains("\\file\\dlc\\dlc_character_018_972")
        || lower.contains("\\file\\dlc\\dlc_character_021_975")
    {
        return Some("dlc_character_law_pack");
    }
    None
}

fn should_trace_dlc_discovery_path(path: &str) -> bool {
    dlc_stack_path_kind(path).is_some()
}

fn remember_find_pattern(handle: Handle, pattern: String) {
    let tracker = FIND_TRACKER.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = tracker.lock() {
        guard.insert(handle as usize, pattern);
    }
}

fn find_pattern(handle: Handle) -> Option<String> {
    let tracker = FIND_TRACKER.get()?;
    let guard = tracker.lock().ok()?;
    guard.get(&(handle as usize)).cloned()
}

fn forget_find_pattern(handle: Handle) {
    let Some(tracker) = FIND_TRACKER.get() else {
        return;
    };
    if let Ok(mut guard) = tracker.lock() {
        guard.remove(&(handle as usize));
    }
}

fn find_data_name(data: *const Win32FindDataW) -> Option<String> {
    if data.is_null() {
        return None;
    }
    let name = unsafe { &(*data).file_name };
    wide_slice_to_string(name)
}

fn wide_slice_to_string(slice: &[u16]) -> Option<String> {
    let len = slice
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(slice.len());
    if len == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&slice[..len]))
}

fn capture_stack_trace() -> Vec<usize> {
    let mut frames = [std::ptr::null_mut(); DLC_STACK_FRAME_COUNT];
    let count = unsafe {
        RtlCaptureStackBackTrace(
            2,
            frames.len() as Dword,
            frames.as_mut_ptr(),
            std::ptr::null_mut(),
        )
    } as usize;
    frames[..count]
        .iter()
        .map(|frame| *frame as usize)
        .filter(|frame| *frame != 0)
        .collect()
}

fn format_stack_frame(frame: usize) -> String {
    if let Some(rva) = game_frame_rva(frame) {
        return format!("game+0x{rva:x}");
    }
    format!("0x{frame:x}")
}

fn format_stack_frames(frames: &[usize], limit: usize) -> String {
    frames
        .iter()
        .copied()
        .take(limit)
        .map(format_stack_frame)
        .collect::<Vec<_>>()
        .join(" ")
}

fn game_frame_rva(frame: usize) -> Option<usize> {
    let game_base = win::main_module() as usize;
    if game_base == 0 || frame < game_base {
        return None;
    }
    let rva = frame - game_base;
    (rva < 0x1000_0000).then_some(rva)
}

fn log_stack_code_contexts(kind: &str, frames: &[usize]) {
    for frame in frames
        .iter()
        .copied()
        .filter(|frame| game_frame_rva(*frame).is_some())
    {
        if STACK_CODE_CONTEXT_LOGS.load(Ordering::Relaxed) >= MAX_STACK_CODE_CONTEXT_LOGS {
            return;
        }
        let already_seen = {
            let tracker = STACK_CODE_CONTEXTS.get_or_init(|| Mutex::new(HashSet::new()));
            let Ok(mut guard) = tracker.lock() else {
                return;
            };
            !guard.insert(frame)
        };
        if already_seen {
            continue;
        }
        let index = STACK_CODE_CONTEXT_LOGS.fetch_add(1, Ordering::Relaxed);
        if index >= MAX_STACK_CODE_CONTEXT_LOGS {
            return;
        }
        let start = frame.saturating_sub(STACK_CODE_CONTEXT_BYTES / 2);
        let mut bytes = [0u8; STACK_CODE_CONTEXT_BYTES];
        let read = win::read_process_memory(start, &mut bytes).unwrap_or(0);
        let Some(frame_rva) = game_frame_rva(frame) else {
            continue;
        };
        let start_label = format_stack_frame(start);
        log::write_line(format!(
            "CreateFileW code kind={kind} frame=game+0x{frame_rva:x} start={start_label} bytes={}",
            hex_bytes(&bytes[..read])
        ));
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn log_data_read_hits(archive_name: &str, read_offset: u64, read_len: usize) {
    if !verbose_io_logs() {
        return;
    }
    if DATA_HIT_LOGS.load(Ordering::Relaxed) >= 240 {
        return;
    }
    let mut hits = with_manager(|manager| {
        manager
            .data_read_hits(archive_name, read_offset, read_len)
            .into_iter()
            .map(|replacement| {
                (
                    replacement.file_name.clone(),
                    replacement.hash,
                    replacement.original_bin_offset.unwrap_or_default(),
                    replacement.mod_size.unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    hits.extend(
        with_runtime_manager(VirtualRuntimeKind::LawCustomSlot, |manager| {
            manager
                .data_read_hits(archive_name, read_offset, read_len)
                .into_iter()
                .map(|replacement| {
                    (
                        replacement.file_name.clone(),
                        replacement.hash,
                        replacement.original_bin_offset.unwrap_or_default(),
                        replacement.mod_size.unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default(),
    );
    if hits.is_empty() {
        return;
    }
    for (file_name, hash, original_offset, mod_size) in hits {
        let index = DATA_HIT_LOGS.fetch_add(1, Ordering::Relaxed);
        if index >= 240 {
            return;
        }
        log::write_line(format!(
            "RDB BIN HIT {archive_name}: read=0x{read_offset:x}+0x{read_len:x} file={file_name} hash=0x{hash:08x} bin_offset=0x{original_offset:x} mod_size=0x{mod_size:x}"
        ));
    }
}

unsafe fn read_overlapped_offset(overlapped: Lpvoid) -> LargeInteger {
    let bytes = overlapped as *const u8;
    *(bytes.add(16) as *const LargeInteger)
}

unsafe fn read_overlapped_event(overlapped: Lpvoid) -> usize {
    let bytes = overlapped as *const u8;
    *(bytes.add(24) as *const usize)
}

fn hex_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

unsafe fn current_file_pointer(handle: Handle) -> Option<LargeInteger> {
    let original = ORIGINALS
        .get()
        .and_then(|originals| originals.set_file_pointer_ex)?;
    let mut position = 0;
    if original(handle, 0, &mut position, 1) == 0 {
        return None;
    }
    Some(position)
}

fn with_rdb_tracker<T>(action: impl FnOnce(&mut RdbTracker) -> T) -> Option<T> {
    let tracker = RDB_TRACKER.get_or_init(|| Mutex::new(RdbTracker::default()));
    let mut guard = tracker.lock().ok()?;
    Some(action(&mut guard))
}

#[cfg_attr(not(test), allow(dead_code))]
fn handle_to_fake(handle: VirtualHandle) -> Handle {
    handle_to_fake_for_runtime(handle, VirtualRuntimeKind::Global)
}

fn handle_to_fake_for_runtime(handle: VirtualHandle, runtime_kind: VirtualRuntimeKind) -> Handle {
    (FAKE_HANDLE_BITS
        | ((runtime_kind.id() << FAKE_HANDLE_RUNTIME_SHIFT) & FAKE_HANDLE_RUNTIME_MASK)
        | (handle.as_raw() as usize & FAKE_HANDLE_ID_MASK)) as Handle
}

#[cfg_attr(not(test), allow(dead_code))]
fn returned_virtual_handle(handle: VirtualHandle) -> Handle {
    handle_to_fake(handle)
}

fn returned_virtual_handle_for_runtime(
    handle: VirtualHandle,
    runtime_kind: VirtualRuntimeKind,
) -> Handle {
    handle_to_fake_for_runtime(handle, runtime_kind)
}

#[cfg_attr(not(test), allow(dead_code))]
fn fake_to_handle(handle: Handle) -> Option<VirtualHandle> {
    fake_virtual_handle(handle).map(|(handle, _runtime_kind)| handle)
}

fn fake_virtual_handle(handle: Handle) -> Option<(VirtualHandle, VirtualRuntimeKind)> {
    let raw = handle as usize;
    if raw & FAKE_HANDLE_MASK != FAKE_HANDLE_BITS {
        return None;
    }
    let runtime_id = (raw & FAKE_HANDLE_RUNTIME_MASK) >> FAKE_HANDLE_RUNTIME_SHIFT;
    let runtime_kind = VirtualRuntimeKind::from_id(runtime_id)?;
    Some((
        VirtualHandle::from_raw((raw & FAKE_HANDLE_ID_MASK) as u64),
        runtime_kind,
    ))
}

fn virtual_handle_for_os_handle(handle: Handle) -> Option<(VirtualHandle, VirtualRuntimeKind)> {
    fake_virtual_handle(handle)
}

fn wide_path_to_string(path: Lpcwstr) -> Option<String> {
    if path.is_null() {
        return None;
    }
    let mut len = 0;
    unsafe {
        while *path.add(len) != 0 {
            len += 1;
        }
        Some(String::from_utf16_lossy(std::slice::from_raw_parts(
            path, len,
        )))
    }
}

fn wide_null_string(path: &str) -> Vec<u16> {
    path.encode_utf16().chain(std::iter::once(0)).collect()
}

fn law_extra_slot_probe_alias_path(path: &str) -> Option<String> {
    let allocated_variant = current_law_extra_slot_probe_variant_id()?;
    law_extra_slot_probe_alias_path_for_variant(path, allocated_variant)
}

fn update_law_custom_slot_activity_from_path(path: &str) {
    let Some(allocated_variant) = current_law_extra_slot_probe_variant_id() else {
        return;
    };
    let Some(file_name) = Path::new(path).file_name() else {
        return;
    };
    let file_name = file_name.to_string_lossy();
    let custom_file_name = law_extra_slot_probe_dlc_file_name(allocated_variant);
    if file_name.eq_ignore_ascii_case(&custom_file_name) {
        set_law_custom_slot_active(true, "dlc-request", Some(allocated_variant));
        return;
    }
    if is_law_costume_dlc_file_name(&file_name) {
        set_law_custom_slot_active(false, "other-law-dlc-request", None);
    }
}

fn is_law_costume_dlc_file_name(file_name: &str) -> bool {
    let lower = file_name.to_ascii_lowercase();
    lower.starts_with("dlc_costume_006_") && lower.contains("_026_") && lower.ends_with(".bin")
}

fn law_extra_slot_probe_alias_path_for_variant(
    path: &str,
    allocated_variant: u16,
) -> Option<String> {
    let file_name = Path::new(path).file_name()?.to_string_lossy();
    let expected_file_name = law_extra_slot_probe_dlc_file_name(allocated_variant);
    if !file_name.eq_ignore_ascii_case(&expected_file_name) {
        return None;
    }
    set_law_custom_slot_active(true, "dlc-alias", Some(allocated_variant));
    let parent = Path::new(path).parent()?;
    Some(
        parent
            .join(LAW_EXTRA_SLOT_SOURCE_DLC_FILE)
            .to_string_lossy()
            .into_owned(),
    )
}

fn law_extra_slot_probe_dlc_file_name(allocated_variant: u16) -> String {
    format!("DLC_COSTUME_006_{allocated_variant:03}_026_004.bin")
}

fn log_virtual_io(args: std::fmt::Arguments<'_>) {
    if !verbose_io_logs() {
        return;
    }
    let index = VIRTUAL_IO_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 512 {
        log::write_line(args.to_string());
    } else if index == 512 {
        log::write_line("Virtual IO logs suppressed".to_string());
    }
}

fn verbose_io_logs() -> bool {
    VERBOSE_IO_LOGS.load(Ordering::Relaxed)
}

fn tracked_archive_name(file_name: &str) -> Option<(String, TrackedFileKind)> {
    let lower = file_name.to_ascii_lowercase();
    if let Some(index) = lower.find(".rdb.bin") {
        if lower[index + ".rdb.bin".len()..]
            .chars()
            .all(|character| character.is_ascii_digit())
        {
            return Some((file_name[..index].to_string(), TrackedFileKind::Data));
        }
    }
    if lower.ends_with(".rdb.bin") {
        return Some((
            file_name[..file_name.len() - ".rdb.bin".len()].to_string(),
            TrackedFileKind::Data,
        ));
    }
    if lower.ends_with(".rdb") {
        return Some((
            file_name[..file_name.len() - ".rdb".len()].to_string(),
            TrackedFileKind::Index,
        ));
    }
    None
}
