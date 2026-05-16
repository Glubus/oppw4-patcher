use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use oppw4_rdb::{inflate_linkdata_entry, parse_linkdata};

use crate::linkdata_costume_dump;

const DEFAULT_INTERESTING_IDS: &[u32] = &[26, 292, 294, 308, 586, 643, 699, 911, 1957, 4713];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceCsvCommand {
    pub(crate) linkdata_path: String,
    pub(crate) out_dir: String,
    pub(crate) ids: Vec<u32>,
    pub(crate) include_all_u16: bool,
    pub(crate) include_all_u32: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InflatedEntrySummary {
    index: usize,
    data_offset: usize,
    field_04: u32,
    compressed_span: u32,
    uncompressed_size: u32,
    inflated_size: Option<usize>,
    inflate_status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IdRef {
    entry_index: usize,
    offset: usize,
    width: usize,
    value: u32,
}

pub(crate) fn parse_command(
    mut args: impl Iterator<Item = String>,
) -> Result<ReferenceCsvCommand, String> {
    let Some(linkdata_path) = args.next() else {
        return Err(usage());
    };

    let mut out_dir = None;
    let mut ids = DEFAULT_INTERESTING_IDS.to_vec();
    let mut include_all_u16 = false;
    let mut include_all_u32 = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out-dir" => out_dir = Some(next_arg(&mut args, &arg)?),
            "--ids" => ids = parse_value_list(next_arg(&mut args, &arg)?)?,
            "--all-u16" => include_all_u16 = true,
            "--all-u32" => include_all_u32 = true,
            _ => return Err(format!("unknown reference csv option: {arg}\n{}", usage())),
        }
    }

    ids.sort_unstable();
    ids.dedup();

    Ok(ReferenceCsvCommand {
        linkdata_path,
        out_dir: out_dir.ok_or_else(usage)?,
        ids,
        include_all_u16,
        include_all_u32,
    })
}

pub(crate) fn run(command: ReferenceCsvCommand) -> Result<(), String> {
    let out_dir = PathBuf::from(&command.out_dir);
    fs::create_dir_all(&out_dir)
        .map_err(|error| format!("failed to create output dir {}: {error}", out_dir.display()))?;

    let linkdata = fs::read(&command.linkdata_path)
        .map_err(|error| format!("failed to read LINKDATA {}: {error}", command.linkdata_path))?;
    let index = parse_linkdata(&linkdata).map_err(|error| {
        format!(
            "failed to parse LINKDATA {}: {error:?}",
            command.linkdata_path
        )
    })?;

    let mut summaries = Vec::new();
    let mut refs = Vec::new();
    let filter = command.ids.iter().copied().collect::<BTreeSet<_>>();
    for entry in &index.entries {
        match inflate_linkdata_entry(&linkdata, entry) {
            Ok(inflated) => {
                refs.extend(scan_refs(
                    entry.index,
                    &inflated,
                    &filter,
                    command.include_all_u16,
                    command.include_all_u32,
                ));
                summaries.push(InflatedEntrySummary {
                    index: entry.index,
                    data_offset: entry.data_offset,
                    field_04: entry.field_04,
                    compressed_span: entry.compressed_span,
                    uncompressed_size: entry.uncompressed_size,
                    inflated_size: Some(inflated.len()),
                    inflate_status: "ok".to_string(),
                });
            }
            Err(error) => summaries.push(InflatedEntrySummary {
                index: entry.index,
                data_offset: entry.data_offset,
                field_04: entry.field_04,
                compressed_span: entry.compressed_span,
                uncompressed_size: entry.uncompressed_size,
                inflated_size: None,
                inflate_status: format!("error:{error:?}"),
            }),
        }
    }

    write_entry_summary_csv(&out_dir.join("linkdata_entries.csv"), &summaries)?;
    write_id_refs_csv(&out_dir.join("linkdata_id_refs.csv"), &refs)?;
    write_ref_summary_csv(&out_dir.join("linkdata_ref_summary.csv"), &refs)?;
    write_ref_clusters_csv(&out_dir.join("linkdata_ref_clusters.csv"), &refs, 0x100)?;
    write_ref_context_csv(
        &out_dir.join("linkdata_ref_context.csv"),
        &refs,
        &linkdata,
        &index.entries,
        0x20,
    )?;
    write_aligned_record_guess_csv(
        &out_dir.join("linkdata_aligned_record_guesses.csv"),
        &refs,
        &linkdata,
        &index.entries,
    )?;
    write_costume_csvs(&command.linkdata_path, &out_dir)?;

    println!(
        "{{\"event\":\"linkdata_reference_csv\",\"linkdata_path\":\"{}\",\"out_dir\":\"{}\",\"entries\":{},\"refs\":{},\"ids\":\"{}\"}}",
        json_str(&command.linkdata_path),
        json_str(&command.out_dir),
        summaries.len(),
        refs.len(),
        command
            .ids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    Ok(())
}

fn scan_refs(
    entry_index: usize,
    bytes: &[u8],
    filter: &BTreeSet<u32>,
    include_all_u16: bool,
    include_all_u32: bool,
) -> Vec<IdRef> {
    let mut refs = Vec::new();
    for offset in (0..bytes.len().saturating_sub(1)).step_by(2) {
        let value = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as u32;
        if include_all_u16 || filter.contains(&value) {
            refs.push(IdRef {
                entry_index,
                offset,
                width: 2,
                value,
            });
        }
    }
    for offset in (0..bytes.len().saturating_sub(3)).step_by(4) {
        let value = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        if include_all_u32 || filter.contains(&value) {
            refs.push(IdRef {
                entry_index,
                offset,
                width: 4,
                value,
            });
        }
    }
    refs.sort_by_key(|reference| (reference.entry_index, reference.offset, reference.width));
    refs
}

fn write_entry_summary_csv(path: &Path, summaries: &[InflatedEntrySummary]) -> Result<(), String> {
    let mut csv =
        "entry_index,data_offset_hex,field_04,compressed_span,uncompressed_size,inflated_size,inflate_status\n"
            .to_string();
    for summary in summaries {
        csv.push_str(&format!(
            "{},0x{:x},{},{},{},{},{}\n",
            summary.index,
            summary.data_offset,
            summary.field_04,
            summary.compressed_span,
            summary.uncompressed_size,
            summary
                .inflated_size
                .map(|value| value.to_string())
                .unwrap_or_default(),
            csv_escape(&summary.inflate_status)
        ));
    }
    write_csv(path, csv)
}

fn write_id_refs_csv(path: &Path, refs: &[IdRef]) -> Result<(), String> {
    let mut csv = "entry_index,offset_hex,width,value\n".to_string();
    for reference in refs {
        csv.push_str(&format!(
            "{},0x{:x},{},{}\n",
            reference.entry_index, reference.offset, reference.width, reference.value
        ));
    }
    write_csv(path, csv)
}

fn write_ref_summary_csv(path: &Path, refs: &[IdRef]) -> Result<(), String> {
    let mut counts = BTreeMap::<(u32, usize, usize), usize>::new();
    for reference in refs {
        *counts
            .entry((reference.value, reference.entry_index, reference.width))
            .or_default() += 1;
    }

    let mut csv = "value,entry_index,width,count\n".to_string();
    for ((value, entry_index, width), count) in counts {
        csv.push_str(&format!("{value},{entry_index},{width},{count}\n"));
    }
    write_csv(path, csv)
}

fn write_ref_clusters_csv(path: &Path, refs: &[IdRef], window_size: usize) -> Result<(), String> {
    let mut clusters = BTreeMap::<(usize, usize), Vec<IdRef>>::new();
    for reference in refs {
        clusters
            .entry((reference.entry_index, reference.offset / window_size))
            .or_default()
            .push(*reference);
    }

    let mut csv =
        "entry_index,window_start_hex,window_end_hex,ref_count,values,widths,offsets_hex\n"
            .to_string();
    for ((entry_index, window), mut refs) in clusters {
        refs.sort_by_key(|reference| (reference.offset, reference.width, reference.value));
        let values = refs
            .iter()
            .map(|reference| reference.value.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let widths = refs
            .iter()
            .map(|reference| reference.width.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let offsets = refs
            .iter()
            .map(|reference| format!("0x{:x}", reference.offset))
            .collect::<Vec<_>>()
            .join(" ");
        let window_start = window * window_size;
        let window_end = window_start + window_size;
        csv.push_str(&format!(
            "{entry_index},0x{window_start:x},0x{window_end:x},{},{},{},{}\n",
            refs.len(),
            csv_escape(&values),
            csv_escape(&widths),
            csv_escape(&offsets)
        ));
    }
    write_csv(path, csv)
}

fn write_ref_context_csv(
    path: &Path,
    refs: &[IdRef],
    linkdata: &[u8],
    entries: &[oppw4_rdb::LinkDataEntry],
    radius: usize,
) -> Result<(), String> {
    let mut csv =
        "entry_index,offset_hex,width,value,context_start_hex,u16_words,u32_words,context_hex\n"
            .to_string();
    let mut inflated_cache = BTreeMap::<usize, Vec<u8>>::new();
    for reference in refs {
        if !inflated_cache.contains_key(&reference.entry_index) {
            let Some(entry) = entries.get(reference.entry_index) else {
                continue;
            };
            let inflated = inflate_linkdata_entry(linkdata, entry).map_err(|error| {
                format!(
                    "failed to inflate LINKDATA entry {} while writing contexts: {error:?}",
                    reference.entry_index
                )
            })?;
            inflated_cache.insert(reference.entry_index, inflated);
        }
        let Some(bytes) = inflated_cache.get(&reference.entry_index) else {
            continue;
        };
        let start = reference.offset.saturating_sub(radius);
        let end = (reference.offset + reference.width + radius).min(bytes.len());
        let context = &bytes[start..end];
        csv.push_str(&format!(
            "{},0x{:x},{},{},0x{:x},{},{},{}\n",
            reference.entry_index,
            reference.offset,
            reference.width,
            reference.value,
            start,
            csv_escape(&format_u16_words(context)),
            csv_escape(&format_u32_words(context)),
            csv_escape(&hex_bytes(context)),
        ));
    }
    write_csv(path, csv)
}

fn write_aligned_record_guess_csv(
    path: &Path,
    refs: &[IdRef],
    linkdata: &[u8],
    entries: &[oppw4_rdb::LinkDataEntry],
) -> Result<(), String> {
    let mut csv = "entry_index,record_start_hex,id_offset_hex,value,u32_00,u32_04,u32_08,u32_0c,u32_10,u32_14,u32_18,u32_1c,u32_20,u32_24,u32_28,u32_2c,u32_30,u32_34,u32_38,u32_3c\n".to_string();
    let mut inflated_cache = BTreeMap::<usize, Vec<u8>>::new();
    let mut seen = BTreeSet::<(usize, usize, u32)>::new();
    for reference in refs {
        if reference.width != 4 || reference.offset < 0x20 || reference.offset % 4 != 0 {
            continue;
        }
        let record_start = reference.offset - 0x20;
        if !seen.insert((reference.entry_index, record_start, reference.value)) {
            continue;
        }
        if !inflated_cache.contains_key(&reference.entry_index) {
            let Some(entry) = entries.get(reference.entry_index) else {
                continue;
            };
            let inflated = inflate_linkdata_entry(linkdata, entry).map_err(|error| {
                format!(
                    "failed to inflate LINKDATA entry {} while writing record guesses: {error:?}",
                    reference.entry_index
                )
            })?;
            inflated_cache.insert(reference.entry_index, inflated);
        }
        let Some(bytes) = inflated_cache.get(&reference.entry_index) else {
            continue;
        };
        if record_start + 0x40 > bytes.len() {
            continue;
        }
        let words = (0..16)
            .map(|index| read_u32(bytes, record_start + index * 4).unwrap_or_default())
            .collect::<Vec<_>>();
        csv.push_str(&format!(
            "{},0x{:x},0x{:x},{}",
            reference.entry_index, record_start, reference.offset, reference.value
        ));
        for word in words {
            csv.push_str(&format!(",{word}"));
        }
        csv.push('\n');
    }
    write_csv(path, csv)
}

fn write_costume_csvs(linkdata_path: &str, out_dir: &Path) -> Result<(), String> {
    let database = linkdata_costume_dump::load_database(linkdata_path)?;

    let mut model_csv = "owner_id,owner_name,row,name,relation\n".to_string();
    let mut layout_csv = "owner_id,owner_name,suffix,section7_id,name\n".to_string();
    let mut dlc_csv = "owner_id,owner_name,code,character_id,costume_index\n".to_string();

    for owner in &database.owners {
        for model in &owner.model_rows {
            model_csv.push_str(&format!(
                "{},{},{},{},{}\n",
                owner.owner_id,
                csv_escape(&owner.owner_name),
                model.row,
                csv_escape(&model.name),
                model.relation
            ));
        }
        for layout in &owner.layouts {
            layout_csv.push_str(&format!(
                "{},{},{},{},{}\n",
                owner.owner_id,
                csv_escape(&owner.owner_name),
                layout.suffix,
                layout.section7_id,
                csv_escape(&layout.name)
            ));
        }
        for dlc in &owner.dlc_costumes {
            dlc_csv.push_str(&format!(
                "{},{},{},{},{}\n",
                owner.owner_id,
                csv_escape(&owner.owner_name),
                csv_escape(&dlc.code),
                dlc.character_id,
                dlc.costume_index
            ));
        }
    }

    write_csv(&out_dir.join("costume_model_rows.csv"), model_csv)?;
    write_csv(&out_dir.join("costume_layouts.csv"), layout_csv)?;
    write_csv(&out_dir.join("costume_dlc_rows.csv"), dlc_csv)?;
    Ok(())
}

fn write_csv(path: &Path, csv: String) -> Result<(), String> {
    fs::write(path, csv).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn format_u16_words(bytes: &[u8]) -> String {
    bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]).to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_u32_words(bytes: &[u8]) -> String {
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]).to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let slice = bytes.get(offset..end)?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn parse_value_list(text: String) -> Result<Vec<u32>, String> {
    text.split(',')
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            part.trim()
                .parse::<u32>()
                .map_err(|error| format!("invalid id {part}: {error}"))
        })
        .collect()
}

fn next_arg(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {option}\n{}", usage()))
}

fn json_str(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn usage() -> String {
    "usage: oppw4-rdb --linkdata-reference-csv <linkdata-bin> --out-dir <dir> [--ids <comma-list>] [--all-u16] [--all-u32]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_escape_quotes_special_cells() {
        assert_eq!(csv_escape("Law"), "Law");
        assert_eq!(csv_escape("Law, Oni"), "\"Law, Oni\"");
        assert_eq!(csv_escape("Law \"Oni\""), "\"Law \"\"Oni\"\"\"");
    }

    #[test]
    fn scans_filtered_u16_and_u32_refs() {
        let bytes = [0x8f, 0x03, 0x00, 0x00, 0xa4, 0x01, 0x34, 0x12];
        let filter = [911, 420].into_iter().collect::<BTreeSet<_>>();
        let refs = scan_refs(7, &bytes, &filter, false, false);
        assert!(refs.contains(&IdRef {
            entry_index: 7,
            offset: 0,
            width: 2,
            value: 911,
        }));
        assert!(refs.contains(&IdRef {
            entry_index: 7,
            offset: 4,
            width: 2,
            value: 420,
        }));
    }

    #[test]
    fn writes_refs_in_same_cluster_window() {
        let path = std::env::temp_dir().join("oppw4_linkdata_ref_cluster_test.csv");
        let refs = [
            IdRef {
                entry_index: 1,
                offset: 0x10,
                width: 2,
                value: 292,
            },
            IdRef {
                entry_index: 1,
                offset: 0x18,
                width: 4,
                value: 911,
            },
        ];

        write_ref_clusters_csv(&path, &refs, 0x100).unwrap();
        let csv = fs::read_to_string(&path).unwrap();

        assert!(csv.contains("1,0x0,0x100,2,292 911,2 4,0x10 0x18"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn formats_words_and_hex_context() {
        let bytes = [0x8f, 0x03, 0x00, 0x00, 0xa4, 0x01, 0x00, 0x00];

        assert_eq!(format_u16_words(&bytes), "911 0 420 0");
        assert_eq!(format_u32_words(&bytes), "911 420");
        assert_eq!(hex_bytes(&bytes), "8f 03 00 00 a4 01 00 00");
    }

    #[test]
    fn writes_aligned_record_guess_from_id_at_0x20() {
        let path = std::env::temp_dir().join("oppw4_linkdata_record_guess_test.csv");
        let mut bytes = vec![0u8; 0x80];
        bytes[0x20..0x24].copy_from_slice(&911u32.to_le_bytes());
        let refs = [IdRef {
            entry_index: 0,
            offset: 0x20,
            width: 4,
            value: 911,
        }];
        let encoded =
            oppw4_rdb::rebuild_linkdata_with_edits(&synthetic_linkdata(), [(0, bytes)]).unwrap();
        let index = parse_linkdata(&encoded).unwrap();

        write_aligned_record_guess_csv(&path, &refs, &encoded, &index.entries).unwrap();
        let csv = fs::read_to_string(&path).unwrap();

        assert!(csv.contains("0,0x0,0x20,911"));
        let _ = fs::remove_file(path);
    }

    fn synthetic_linkdata() -> Vec<u8> {
        let mut bytes = vec![0u8; 0x100];
        bytes[0..4].copy_from_slice(&0x0007_7df9u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes
    }
}
