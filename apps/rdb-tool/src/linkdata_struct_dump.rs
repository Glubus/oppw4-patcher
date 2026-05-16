use std::fs;

use oppw4_data_struct::{
    entry17::{Entry17, Entry17Occurrence, Entry17RecordCandidate, Entry17Window},
    entry29::{DlcCostumeCode, DlcEntry},
    entry3::{CostumeLayoutRow, CostumeStaticEntry, CostumeVariantMetadata},
    entry32::{Entry32, Entry32String},
    entry35::{ModelEntry, ModelRow},
    entry39::{CostumeParamEntry, CostumeParamRow},
    entry52::{CostumeRecord, CostumeRecordEntry},
    entry58::{CostumeSection, CostumeSectionEntry},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructDumpCommand {
    linkdata_path: String,
    target: Option<u16>,
    preview: Option<u16>,
    variant: Option<u16>,
    registry_refs: Vec<(usize, usize)>,
}

pub(crate) fn parse_command(
    mut args: impl Iterator<Item = String>,
) -> Result<StructDumpCommand, String> {
    let Some(linkdata_path) = args.next() else {
        return Err(usage());
    };

    let mut target = None;
    let mut preview = None;
    let mut variant = None;
    let mut registry_refs = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--target" => target = Some(parse_u16(&next_arg(&mut args, &arg)?)?),
            "--preview" => preview = Some(parse_u16(&next_arg(&mut args, &arg)?)?),
            "--variant" => variant = Some(parse_u16(&next_arg(&mut args, &arg)?)?),
            "--registry" => registry_refs.push(parse_registry_ref(&next_arg(&mut args, &arg)?)?),
            _ => return Err(format!("unknown struct dump option: {arg}\n{}", usage())),
        }
    }

    Ok(StructDumpCommand {
        linkdata_path,
        target,
        preview,
        variant,
        registry_refs,
    })
}

pub(crate) fn run(command: StructDumpCommand) -> Result<(), String> {
    let linkdata = fs::read(&command.linkdata_path)
        .map_err(|error| format!("failed to read {}: {error}", command.linkdata_path))?;
    let entry3 = CostumeStaticEntry::from_linkdata_bytes(&linkdata)
        .map_err(|error| format!("failed to parse entry3: {error}"))?;

    let rows = match (command.target, command.preview) {
        (Some(target), Some(preview)) => entry3
            .find_layout_rows(target, preview)
            .map_err(|error| error.to_string())?,
        _ => entry3
            .scan_layout_rows()
            .map_err(|error| error.to_string())?,
    };
    let variant = command
        .variant
        .map(|variant_id| entry3.variant_metadata(variant_id))
        .transpose()
        .map_err(|error| error.to_string())?;
    let registry = if command.registry_refs.is_empty() {
        None
    } else {
        Some(
            Entry32::from_linkdata_bytes(&linkdata)
                .map_err(|error| format!("failed to parse entry32: {error}"))?,
        )
    };
    let extra = linked_entries_json(&linkdata, command.target, command.variant, variant.as_ref())?;

    println!(
        "{{\"event\":\"linkdata_struct_dump\",\"linkdata_path\":\"{}\",\"entry3\":{{\"layout_rows\":{},\"variant_metadata\":{}}},\"entry32\":{},\"linked_entries\":{}}}",
        json_str(&command.linkdata_path),
        layout_rows_json(&rows),
        variant_metadata_json(variant.as_ref()),
        registry_refs_json(registry.as_ref(), &command.registry_refs),
        extra
    );
    Ok(())
}

fn linked_entries_json(
    linkdata: &[u8],
    target: Option<u16>,
    variant: Option<u16>,
    metadata: Option<&CostumeVariantMetadata>,
) -> Result<String, String> {
    let Some(variant) = variant else {
        return Ok("null".to_string());
    };
    let owner = target.unwrap_or(26);

    let entry29 = DlcEntry::from_linkdata_bytes(linkdata)
        .map_err(|error| format!("failed to parse entry29: {error}"))?;
    let dlc_codes = entry29.find_costume_codes(owner, variant);

    let entry35 = ModelEntry::from_linkdata_bytes(linkdata)
        .map_err(|error| format!("failed to parse entry35: {error}"))?;
    let model_row = metadata
        .and_then(|metadata| {
            (metadata.model_resource != u16::MAX).then_some(metadata.model_resource)
        })
        .map(|model| entry35.row(model))
        .transpose()
        .map_err(|error| format!("failed to parse entry35 model row: {error}"))?;
    let owner_model_rows = entry35
        .rows_for_owner(owner as i16)
        .map_err(|error| format!("failed to parse entry35 owner rows: {error}"))?;

    let entry39 = CostumeParamEntry::from_linkdata_bytes(linkdata)
        .map_err(|error| format!("failed to parse entry39: {error}"))?;
    let param_row = entry39.row(variant).ok();

    let entry52 = CostumeRecordEntry::from_linkdata_bytes(linkdata)
        .map_err(|error| format!("failed to parse entry52: {error}"))?;
    let records = entry52
        .records_for_owner_layout(u32::from(owner), u32::from(variant))
        .map_err(|error| format!("failed to parse entry52 records: {error}"))?;

    let entry58 = CostumeSectionEntry::from_linkdata_bytes(linkdata)
        .map_err(|error| format!("failed to parse entry58: {error}"))?;
    let section = entry58
        .section(variant)
        .map_err(|error| format!("failed to parse entry58 section: {error}"))?;

    let entry17_values = entry17_values(owner, variant, metadata);
    let entry17 = entry17_json(linkdata, &entry17_values)?;

    Ok(format!(
        "{{\"entry17\":{},\"entry29\":{{\"dlc_costume_codes\":{}}},\"entry35\":{{\"variant_model_row\":{},\"owner_model_rows\":{}}},\"entry39\":{},\"entry52\":{{\"owner_layout_records\":{}}},\"entry58\":{}}}",
        entry17,
        dlc_codes_json(&dlc_codes),
        model_row_json(model_row.as_ref()),
        model_rows_json(&owner_model_rows),
        costume_param_json(param_row.as_ref()),
        costume_records_json(&records),
        costume_section_json(section.as_ref())
    ))
}

fn entry17_values(owner: u16, variant: u16, metadata: Option<&CostumeVariantMetadata>) -> Vec<u32> {
    let mut values = vec![
        u32::from(owner),
        u32::from(variant),
        57,
        58,
        131,
        555,
        586,
        911,
        1957,
        4713,
    ];
    if let Some(metadata) = metadata {
        if metadata.model_resource != u16::MAX {
            values.push(u32::from(metadata.model_resource));
        }
        if metadata.preview_mapping != u16::MAX {
            values.push(u32::from(metadata.preview_mapping));
        }
    }
    values.sort_unstable();
    values.dedup();
    values
}

fn entry17_json(linkdata: &[u8], values: &[u32]) -> Result<String, String> {
    let entry = Entry17::from_linkdata_bytes(linkdata)
        .map_err(|error| format!("failed to parse entry17: {error}"))?;
    let occurrences = entry.occurrences_for_values(values);
    let windows = entry.clustered_windows(&occurrences, 0x20, 0x18);
    let focused = focused_entry17_windows(&windows);
    let record_ids = entry17_record_ids(values);
    let records = entry
        .record_candidates_for_ids(&record_ids)
        .map_err(|error| format!("failed to parse entry17 record candidates: {error}"))?;
    let diff = entry17_record_diff_json(&records, 586, 699);
    let verdict = entry17_verdict(&records, 586, 699);
    Ok(format!(
        "{{\"len\":{},\"watched_values\":[{}],\"hit_counts\":{},\"windows\":{},\"focused_windows\":{},\"records\":{},\"record_diff\":{},\"verdict\":\"{}\"}}",
        entry.len(),
        values
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
        entry17_counts_json(values, &occurrences),
        entry17_windows_json(&windows),
        entry17_windows_json(&focused),
        entry17_records_json(&records, &record_ids),
        diff,
        verdict
    ))
}

fn entry17_record_ids(values: &[u32]) -> Vec<u32> {
    let mut ids = values
        .iter()
        .copied()
        .filter(|value| !matches!(*value, 26 | 555))
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn focused_entry17_windows(windows: &[Entry17Window]) -> Vec<Entry17Window> {
    windows
        .iter()
        .filter(|window| {
            window
                .hits
                .iter()
                .any(|hit| !matches!(hit.value, 26 | 57 | 58 | 131 | 555 | 586))
        })
        .cloned()
        .collect()
}

fn entry17_counts_json(values: &[u32], occurrences: &[Entry17Occurrence]) -> String {
    let items = values
        .iter()
        .map(|value| {
            let u16_count = occurrences
                .iter()
                .filter(|hit| hit.value == *value && hit.width == 2)
                .count();
            let u32_count = occurrences
                .iter()
                .filter(|hit| hit.value == *value && hit.width == 4)
                .count();
            format!("{{\"value\":{value},\"u16\":{u16_count},\"u32\":{u32_count}}}")
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn entry17_windows_json(windows: &[Entry17Window]) -> String {
    let items = windows
        .iter()
        .take(12)
        .map(|window| {
            format!(
                "{{\"start\":\"0x{:x}\",\"end\":\"0x{:x}\",\"hits\":{},\"raw\":\"{}\"}}",
                window.start,
                window.end,
                entry17_hits_json(&window.hits),
                hex_bytes(&window.raw)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn entry17_hits_json(hits: &[Entry17Occurrence]) -> String {
    let items = hits
        .iter()
        .map(|hit| {
            format!(
                "{{\"offset\":\"0x{:x}\",\"value\":{},\"width\":{}}}",
                hit.offset, hit.value, hit.width
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn entry17_records_json(records: &[Entry17RecordCandidate], ids: &[u32]) -> String {
    let items = ids
        .iter()
        .map(|id| {
            let matches = records
                .iter()
                .filter(|record| record.id == *id)
                .collect::<Vec<_>>();
            let examples = matches
                .iter()
                .take(3)
                .map(|record| entry17_record_json(record))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":{},\"count\":{},\"examples\":[{}]}}",
                id,
                matches.len(),
                examples
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn entry17_record_json(record: &Entry17RecordCandidate) -> String {
    format!(
        "{{\"offset\":\"0x{:x}\",\"id_offset\":\"0x{:x}\",\"fields\":[{}],\"raw\":\"{}\"}}",
        record.offset,
        record.id_offset,
        record
            .fields
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
        hex_bytes(&record.raw)
    )
}

fn entry17_record_diff_json(
    records: &[Entry17RecordCandidate],
    source_id: u32,
    target_id: u32,
) -> String {
    let source = records.iter().find(|record| record.id == source_id);
    let target = records.iter().find(|record| record.id == target_id);
    let (Some(source), Some(target)) = (source, target) else {
        return format!(
            "{{\"source_id\":{source_id},\"target_id\":{target_id},\"available\":false}}"
        );
    };
    let field_diffs = source
        .fields
        .iter()
        .zip(target.fields.iter())
        .enumerate()
        .filter_map(|(index, (source, target))| {
            (source != target)
                .then(|| format!("{{\"index\":{index},\"source\":{source},\"target\":{target}}}"))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"source_id\":{source_id},\"target_id\":{target_id},\"available\":true,\"source_offset\":\"0x{:x}\",\"target_offset\":\"0x{:x}\",\"field_diffs\":[{}]}}",
        source.offset, target.offset, field_diffs
    )
}

fn entry17_verdict(
    records: &[Entry17RecordCandidate],
    source_id: u32,
    target_id: u32,
) -> &'static str {
    let source = records.iter().find(|record| record.id == source_id);
    let target = records.iter().find(|record| record.id == target_id);
    let (Some(source), Some(target)) = (source, target) else {
        return "unknown";
    };

    let non_id_fields_match = source
        .fields
        .iter()
        .zip(target.fields.iter())
        .enumerate()
        .all(|(index, (source, target))| index == 8 || source == target);
    if non_id_fields_match {
        "matches_known_costume"
    } else {
        "missing_binding"
    }
}

fn layout_rows_json(rows: &[CostumeLayoutRow]) -> String {
    let items = rows
        .iter()
        .map(layout_row_json)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn layout_row_json(row: &CostumeLayoutRow) -> String {
    format!(
        "{{\"offset\":\"0x{:x}\",\"target\":{},\"preview\":{},\"active_count\":{},\"flags_4a\":{},\"variants\":[{}],\"raw\":\"{}\"}}",
        row.offset,
        row.target,
        row.preview,
        row.active_count,
        row.flags_4a,
        row.variants
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(","),
        hex_bytes(&row.raw)
    )
}

fn variant_metadata_json(metadata: Option<&CostumeVariantMetadata>) -> String {
    let Some(metadata) = metadata else {
        return "null".to_string();
    };
    format!(
        "{{\"variant_id\":{},\"offset\":\"0x{:x}\",\"model_resource\":{},\"preview_mapping\":{},\"flags\":{},\"color_bytes\":\"{}\",\"raw\":\"{}\"}}",
        metadata.variant_id,
        metadata.offset,
        metadata.model_resource,
        metadata.preview_mapping,
        metadata.flags,
        hex_bytes(&metadata.color_bytes),
        hex_bytes(&metadata.raw)
    )
}

fn dlc_codes_json(codes: &[DlcCostumeCode]) -> String {
    let items = codes
        .iter()
        .map(|code| {
            format!(
                "{{\"code\":\"{}\",\"pack\":{},\"variant\":{},\"owner\":{},\"costume_index\":{}}}",
                json_str(&code.code),
                code.pack,
                code.variant,
                code.owner,
                code.costume_index
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn model_rows_json(rows: &[ModelRow]) -> String {
    let items = rows
        .iter()
        .take(32)
        .map(|row| model_row_json(Some(row)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn model_row_json(row: Option<&ModelRow>) -> String {
    let Some(row) = row else {
        return "null".to_string();
    };
    format!(
        "{{\"row\":{},\"offset\":\"0x{:x}\",\"owner\":{},\"relation\":{},\"raw\":\"{}\"}}",
        row.row,
        row.offset,
        row.owner,
        row.relation,
        hex_bytes(&row.raw)
    )
}

fn costume_param_json(row: Option<&CostumeParamRow>) -> String {
    let Some(row) = row else {
        return "null".to_string();
    };
    format!(
        "{{\"row\":{},\"offset\":\"0x{:x}\",\"present\":{},\"fields\":[{}],\"raw\":\"{}\"}}",
        row.row,
        row.offset,
        row.present,
        row.fields
            .iter()
            .map(i16::to_string)
            .collect::<Vec<_>>()
            .join(","),
        hex_bytes(&row.raw)
    )
}

fn costume_records_json(records: &[CostumeRecord]) -> String {
    let items = records
        .iter()
        .map(|record| {
            format!(
                "{{\"row\":{},\"offset\":\"0x{:x}\",\"fields\":[{}],\"raw\":\"{}\"}}",
                record.row,
                record.offset,
                record
                    .fields
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                hex_bytes(&record.raw)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn costume_section_json(section: Option<&CostumeSection>) -> String {
    let Some(section) = section else {
        return "null".to_string();
    };
    let records = section
        .records
        .iter()
        .take(16)
        .map(|record| {
            format!(
                "{{\"index\":{},\"offset\":\"0x{:x}\",\"fields\":[{}],\"raw\":\"{}\"}}",
                record.index,
                record.offset,
                record
                    .fields
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                hex_bytes(&record.raw)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"suffix\":{},\"offset\":\"0x{:x}\",\"record_count\":{},\"header\":[{}],\"records\":[{}]}}",
        section.suffix,
        section.offset,
        section.record_count,
        section
            .header
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
        records
    )
}

fn parse_u16(text: &str) -> Result<u16, String> {
    if let Some(hex) = text.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).map_err(|error| format!("invalid u16 '{text}': {error}"))
    } else {
        text.parse::<u16>()
            .map_err(|error| format!("invalid u16 '{text}': {error}"))
    }
}

fn parse_registry_ref(text: &str) -> Result<(usize, usize), String> {
    let Some((section, id)) = text.split_once(':') else {
        return Err(format!(
            "invalid registry ref '{text}', expected section:id"
        ));
    };
    Ok((parse_usize(section)?, parse_usize(id)?))
}

fn parse_usize(text: &str) -> Result<usize, String> {
    if let Some(hex) = text.strip_prefix("0x") {
        usize::from_str_radix(hex, 16).map_err(|error| format!("invalid usize '{text}': {error}"))
    } else {
        text.parse::<usize>()
            .map_err(|error| format!("invalid usize '{text}': {error}"))
    }
}

fn registry_refs_json(entry32: Option<&Entry32>, refs: &[(usize, usize)]) -> String {
    let Some(entry32) = entry32 else {
        return "null".to_string();
    };
    let items = refs
        .iter()
        .map(|(section, id)| {
            let item = entry32
                .section(*section)
                .and_then(|section| section.strings.get(*id));
            registry_ref_json(*section, *id, item)
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"refs\":[{items}]}}")
}

fn registry_ref_json(section: usize, id: usize, item: Option<&Entry32String>) -> String {
    let Some(item) = item else {
        return format!("{{\"section\":{section},\"id\":{id},\"present\":false}}");
    };
    format!(
        "{{\"section\":{section},\"id\":{id},\"present\":true,\"byte_len\":{},\"relative_offset\":\"0x{:x}\",\"value\":\"{}\"}}",
        item.byte_len,
        item.relative_offset,
        json_str(&item.value)
    )
}

fn next_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn json_str(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn usage() -> String {
    "usage: oppw4-rdb --linkdata-struct-dump <linkdata-bin> [--target <id> --preview <id>] [--variant <id>] [--registry <section:id>]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_dump_command_parses_target_preview_and_variant() {
        let command = parse_command(
            [
                "LINKDATA_A.BIN",
                "--target",
                "26",
                "--preview",
                "0x1a",
                "--variant",
                "699",
                "--registry",
                "7:4713",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .unwrap();

        assert_eq!(
            command,
            StructDumpCommand {
                linkdata_path: "LINKDATA_A.BIN".to_string(),
                target: Some(26),
                preview: Some(26),
                variant: Some(699),
                registry_refs: vec![(7, 4713)],
            }
        );
    }
}
