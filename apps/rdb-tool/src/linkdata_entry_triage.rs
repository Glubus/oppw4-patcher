use std::fs;

use oppw4_rdb::{inflate_linkdata_entry, parse_linkdata};

const DEFAULT_NUMERIC_NEEDLES: &[u32] = &[26, 57, 58, 131, 294, 586, 699, 730, 911];
const DEFAULT_TEXT_NEEDLES: &[&str] = &[
    "MPLC026",
    "MDLC999",
    "806_131",
    "DLC_COSTUME_006_586_026_003",
    "DLC_COSTUME_007_699_026_004",
];

pub struct TriageCommand {
    pub linkdata_path: String,
}

pub fn parse_command(mut args: impl Iterator<Item = String>) -> Result<TriageCommand, String> {
    let Some(linkdata_path) = args.next() else {
        return Err("usage: oppw4-rdb --linkdata-entry-triage <linkdata-bin>".to_string());
    };
    Ok(TriageCommand { linkdata_path })
}

pub fn run(command: TriageCommand) -> Result<(), String> {
    let bytes = fs::read(&command.linkdata_path)
        .map_err(|error| format!("failed to read LINKDATA {}: {error}", command.linkdata_path))?;
    let index = parse_linkdata(&bytes).map_err(|error| {
        format!(
            "failed to parse LINKDATA {}: {error:?}",
            command.linkdata_path
        )
    })?;

    println!("entry,offset,span,inflated_size,numeric_hits,text_hits,known_struct,notes");
    for entry in &index.entries {
        let inflated = inflate_linkdata_entry(&bytes, entry)
            .map_err(|error| format!("failed to inflate entry {}: {error:?}", entry.index))?;
        let numeric_hits = numeric_hits(&inflated);
        let text_hits = text_hits(&inflated);
        let known_struct = known_struct(entry.index);
        if numeric_hits.is_empty() && text_hits.is_empty() && known_struct.is_empty() {
            continue;
        }
        println!(
            "{},0x{:x},0x{:x},0x{:x},{},{},{},{}",
            entry.index,
            entry.data_offset,
            entry.compressed_span,
            inflated.len(),
            csv_cell(&numeric_hits),
            csv_cell(&text_hits),
            known_struct,
            csv_cell(&entry_notes(entry.index, &inflated)),
        );
    }

    Ok(())
}

fn numeric_hits(bytes: &[u8]) -> Vec<String> {
    DEFAULT_NUMERIC_NEEDLES
        .iter()
        .filter_map(|needle| {
            let u16_count = (*needle <= u16::MAX as u32)
                .then(|| count_pattern(bytes, &(*needle as u16).to_le_bytes()))
                .unwrap_or(0);
            let u32_count = count_pattern(bytes, &needle.to_le_bytes());
            (u16_count > 0 || u32_count > 0)
                .then(|| format!("{needle}:u16={u16_count}:u32={u32_count}"))
        })
        .collect()
}

fn text_hits(bytes: &[u8]) -> Vec<String> {
    DEFAULT_TEXT_NEEDLES
        .iter()
        .filter_map(|needle| {
            let count = count_pattern(bytes, needle.as_bytes());
            (count > 0).then(|| format!("{needle}={count}"))
        })
        .collect()
}

fn count_pattern(bytes: &[u8], pattern: &[u8]) -> usize {
    if pattern.is_empty() || bytes.len() < pattern.len() {
        return 0;
    }
    bytes
        .windows(pattern.len())
        .filter(|window| *window == pattern)
        .count()
}

fn known_struct(entry: usize) -> &'static str {
    match entry {
        3 => "entry3_costume_static",
        29 => "entry29_dlc_codes",
        32 => "entry32_string_registry",
        35 => "entry35_model_rows",
        39 => "entry39_costume_params",
        52 => "entry52_costume_records",
        58 => "entry58_costume_sections",
        _ => "",
    }
}

fn entry_notes(entry: usize, bytes: &[u8]) -> Vec<String> {
    let mut notes = Vec::new();
    if bytes.iter().filter(|byte| byte.is_ascii_graphic()).count() > bytes.len() / 4 {
        notes.push("text_heavy".to_string());
    }
    if entry <= 64 {
        notes.push("early_table".to_string());
    }
    notes
}

fn csv_cell(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    format!("\"{}\"", items.join(";").replace('"', "\"\""))
}
