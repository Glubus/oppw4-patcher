use std::{collections::BTreeMap, fs};

use oppw4_rdb::{inflate_linkdata_entry, parse_linkdata};

const ENTRY_DLC: usize = 29;
const ENTRY_NAMES: usize = 32;
const ENTRY_MODELS: usize = 35;
const ENTRY_COSTUME_PARAMS: usize = 39;
const ENTRY_COSTUME_SECTIONS: usize = 58;
const MODEL_ROW_STRIDE: usize = 96;
const MODEL_ROW_BASE: usize = 0x10;
const COSTUME_PARAM_ROW_STRIDE: usize = 48;
const COSTUME_PARAM_ROW_BASE: usize = 0x10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DumpConfig {
    pub linkdata_path: String,
    pub owner_filter: Option<Vec<u32>>,
    pub name_contains: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CostumeDatabase {
    pub(crate) owners: Vec<OwnerDump>,
    pub(crate) layouts: Vec<CostumeLayout>,
    pub(crate) entry58: Vec<u8>,
    pub(crate) entry39: Vec<u8>,
    pub(crate) model_row_count: usize,
    pub(crate) section6_count: usize,
    pub(crate) section6_names: Vec<String>,
    pub(crate) section7_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StringRegistry {
    sections: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CostumeLayout {
    pub(crate) suffix: u32,
    pub(crate) section7_id: usize,
    pub(crate) name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DlcCostume {
    pub(crate) code: String,
    pub(crate) character_id: u32,
    pub(crate) costume_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelRow {
    pub(crate) row: usize,
    pub(crate) name: String,
    pub(crate) owner: i16,
    pub(crate) relation: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry58Section {
    record_count: u32,
    first_pairs: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CostumeParamRow {
    present: bool,
    first_i16: Vec<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnerDump {
    pub(crate) owner_id: u32,
    pub(crate) owner_name: String,
    pub(crate) stem: String,
    pub(crate) model_rows: Vec<ModelRow>,
    pub(crate) layouts: Vec<CostumeLayout>,
    pub(crate) dlc_costumes: Vec<DlcCostume>,
}

pub fn parse_command(mut args: impl Iterator<Item = String>) -> Result<DumpConfig, String> {
    let Some(linkdata_path) = args.next() else {
        return Err(usage());
    };

    let mut owner_filter = None;
    let mut name_contains = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--owners" => owner_filter = Some(parse_value_list(next_arg(&mut args, "--owners")?)?),
            "--name-contains" => name_contains = Some(next_arg(&mut args, "--name-contains")?),
            _ => return Err(format!("unknown costume dump option: {arg}\n{}", usage())),
        }
    }

    Ok(DumpConfig {
        linkdata_path,
        owner_filter,
        name_contains,
    })
}

pub fn run(config: DumpConfig) -> Result<(), String> {
    let database = load_database_with_config(&config)?;

    println!(
        "{{\"event\":\"costume_dump\",\"linkdata_path\":\"{}\",\"owners\":{}}}",
        json_str(&config.linkdata_path),
        owners_json(&database.owners, &database.entry58, &database.entry39)
    );

    Ok(())
}

pub(crate) fn load_database(linkdata_path: &str) -> Result<CostumeDatabase, String> {
    load_database_with_config(&DumpConfig {
        linkdata_path: linkdata_path.to_string(),
        owner_filter: None,
        name_contains: None,
    })
}

fn load_database_with_config(config: &DumpConfig) -> Result<CostumeDatabase, String> {
    let linkdata = fs::read(&config.linkdata_path)
        .map_err(|error| format!("failed to read LINKDATA {}: {error}", config.linkdata_path))?;
    let entries = parse_linkdata(&linkdata).map_err(|error| {
        format!(
            "failed to parse LINKDATA {}: {error:?}",
            config.linkdata_path
        )
    })?;

    let entry32 = inflate_required_entry(&linkdata, &entries.entries, ENTRY_NAMES)?;
    let registry = parse_string_registry(&entry32)?;
    let layouts = collect_costume_layouts(&registry);
    let dlc_costumes = collect_dlc_costumes(&inflate_required_entry(
        &linkdata,
        &entries.entries,
        ENTRY_DLC,
    )?);
    let model_entry = inflate_required_entry(&linkdata, &entries.entries, ENTRY_MODELS)?;
    let model_row_count = read_u32(&model_entry, 0).unwrap_or(0) as usize;
    let model_rows = parse_model_rows(&model_entry, &registry);
    let entry58 = inflate_required_entry(&linkdata, &entries.entries, ENTRY_COSTUME_SECTIONS)?;
    let entry39 = inflate_required_entry(&linkdata, &entries.entries, ENTRY_COSTUME_PARAMS)?;
    let owners = build_owner_dumps(&config, &registry, model_rows, &layouts, &dlc_costumes);

    Ok(CostumeDatabase {
        owners,
        layouts,
        entry58,
        entry39,
        model_row_count,
        section6_count: registry.sections.get(6).map_or(0, Vec::len),
        section6_names: registry.sections.get(6).cloned().unwrap_or_default(),
        section7_count: registry.sections.get(7).map_or(0, Vec::len),
    })
}

fn inflate_required_entry(
    linkdata: &[u8],
    entries: &[oppw4_rdb::LinkDataEntry],
    index: usize,
) -> Result<Vec<u8>, String> {
    let entry = entries
        .get(index)
        .ok_or_else(|| format!("missing LINKDATA entry {index}"))?;
    inflate_linkdata_entry(linkdata, entry)
        .map_err(|error| format!("failed to inflate LINKDATA entry {index}: {error:?}"))
}

fn build_owner_dumps(
    config: &DumpConfig,
    registry: &StringRegistry,
    model_rows: Vec<ModelRow>,
    layouts: &[CostumeLayout],
    dlc_costumes: &[DlcCostume],
) -> Vec<OwnerDump> {
    let mut grouped: BTreeMap<u32, Vec<ModelRow>> = BTreeMap::new();
    for row in model_rows {
        if let Ok(owner) = u32::try_from(row.owner) {
            grouped.entry(owner).or_default().push(row);
        }
    }

    grouped
        .into_iter()
        .filter_map(|(owner_id, mut model_rows)| {
            if let Some(filter) = &config.owner_filter {
                if !filter.contains(&owner_id) {
                    return None;
                }
            }

            let owner_name = registry
                .name(6, owner_id as usize)
                .or_else(|| registry.name(4, owner_id as usize))
                .unwrap_or_default()
                .to_string();
            let stem = model_stem(&owner_name)?;
            if let Some(needle) = &config.name_contains {
                let needle = needle.to_ascii_lowercase();
                if !owner_name.to_ascii_lowercase().contains(&needle) && !stem.contains(&needle) {
                    return None;
                }
            }

            let layouts = layouts_for_stem(layouts, &stem);
            let dlc_costumes = dlc_costumes
                .iter()
                .filter(|costume| costume.character_id == owner_id)
                .cloned()
                .collect::<Vec<_>>();

            if layouts.is_empty() && dlc_costumes.is_empty() {
                return None;
            }

            model_rows.sort_by_key(|row| row.row);
            Some(OwnerDump {
                owner_id,
                owner_name,
                stem,
                model_rows,
                layouts,
                dlc_costumes,
            })
        })
        .collect()
}

fn parse_string_registry(bytes: &[u8]) -> Result<StringRegistry, String> {
    let section_count = read_u32(bytes, 0)? as usize;
    let mut offsets = Vec::with_capacity(section_count);
    for index in 0..section_count {
        offsets.push(read_u32(bytes, 4 + index * 4)? as usize);
    }

    let mut sections = Vec::with_capacity(section_count);
    for offset in offsets {
        sections.push(parse_string_section(bytes, offset)?);
    }

    Ok(StringRegistry { sections })
}

fn parse_string_section(bytes: &[u8], section_offset: usize) -> Result<Vec<String>, String> {
    if section_offset == 0 || section_offset >= bytes.len() {
        return Ok(Vec::new());
    }

    let count = read_u32(bytes, section_offset)? as usize;
    let mut strings = Vec::with_capacity(count);
    for index in 0..count {
        let row = section_offset + 4 + index * 8;
        let string_offset = read_u32(bytes, row)? as usize;
        let string_len = read_u32(bytes, row + 4)? as usize;
        let start = section_offset + string_offset;
        let end = start + string_len;
        let Some(slice) = bytes.get(start..end) else {
            return Err(format!(
                "string section out of bounds: section=0x{section_offset:x} start=0x{start:x} len=0x{string_len:x}"
            ));
        };
        strings.push(trim_nul(slice));
    }
    Ok(strings)
}

fn collect_costume_layouts(registry: &StringRegistry) -> Vec<CostumeLayout> {
    let mut layouts = registry
        .sections
        .get(7)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(section7_id, name)| {
            costume_layout_suffix(name).map(|suffix| CostumeLayout {
                suffix,
                section7_id,
                name: name.clone(),
            })
        })
        .collect::<Vec<_>>();
    layouts.sort_by_key(|layout| (layout.suffix, layout.section7_id));
    layouts
}

fn collect_dlc_costumes(bytes: &[u8]) -> Vec<DlcCostume> {
    let mut costumes = ascii_strings(bytes, 8)
        .into_iter()
        .filter_map(|string| parse_dlc_costume(&string))
        .collect::<Vec<_>>();
    costumes.sort_by_key(|costume| {
        (
            costume.character_id,
            costume.costume_index,
            costume.code.clone(),
        )
    });
    costumes.dedup_by(|left, right| left.code == right.code);
    costumes
}

fn parse_model_rows(bytes: &[u8], registry: &StringRegistry) -> Vec<ModelRow> {
    let Ok(count) = read_u32(bytes, 0).map(|count| count as usize) else {
        return Vec::new();
    };

    (0..count)
        .filter_map(|row| {
            let start = MODEL_ROW_BASE + row * MODEL_ROW_STRIDE;
            let end = start + MODEL_ROW_STRIDE;
            if end > bytes.len() {
                return None;
            }
            let name = registry.name(6, row)?;
            if name.is_empty() {
                return None;
            }
            Some(ModelRow {
                row,
                name: name.to_string(),
                owner: read_i16_or(bytes, start + 28 * 2, -1),
                relation: read_i16_or(bytes, start + 34 * 2, -1),
            })
        })
        .filter(|row| row.owner >= 0)
        .collect()
}

fn layouts_for_stem(layouts: &[CostumeLayout], stem: &str) -> Vec<CostumeLayout> {
    let needle = format!("_costume_{stem}");
    layouts
        .iter()
        .filter(|layout| layout.name.to_ascii_lowercase().contains(&needle))
        .cloned()
        .collect()
}

fn costume_layout_suffix(name: &str) -> Option<u32> {
    let rest = name.strip_prefix("806_")?;
    let (raw_suffix, tail) = rest.split_once('_')?;
    if !tail.starts_with("costume") {
        return None;
    }
    raw_suffix.parse().ok()
}

fn model_stem(name: &str) -> Option<String> {
    let (_, tail) = name.split_once('_')?;
    let stem = tail.split('_').next()?.to_ascii_lowercase();
    (!stem.is_empty()).then_some(stem)
}

fn parse_dlc_costume(string: &str) -> Option<DlcCostume> {
    let rest = string.strip_prefix("DLC_COSTUME_")?;
    let parts = rest.split('_').collect::<Vec<_>>();
    if parts.len() != 4 {
        return None;
    }
    Some(DlcCostume {
        code: string.to_string(),
        character_id: parts[2].parse().ok()?,
        costume_index: parts[3].parse().ok()?,
    })
}

fn parse_entry58_section(bytes: &[u8], suffix: u32) -> Option<Entry58Section> {
    let section_count = read_u32(bytes, 0).ok()? as usize;
    let index = suffix as usize;
    if index >= section_count {
        return None;
    }

    let section_offset = read_u32(bytes, 4 + index * 4).ok()? as usize;
    if section_offset == 0 || section_offset + 0x10 > bytes.len() {
        return None;
    }

    let record_count = read_u32(bytes, section_offset).ok()?;
    let records_start = section_offset + 0x10;
    let mut first_pairs = Vec::new();
    for record in 0..record_count.min(8) as usize {
        let offset = records_start + record * 0x10;
        if offset + 8 > bytes.len() {
            break;
        }
        first_pairs.push((
            read_u32(bytes, offset).ok()?,
            read_u32(bytes, offset + 4).ok()?,
        ));
    }

    Some(Entry58Section {
        record_count,
        first_pairs,
    })
}

fn parse_costume_param_row(bytes: &[u8], suffix: u32) -> Option<CostumeParamRow> {
    let count = read_u32(bytes, 0).ok()? as usize;
    let row = suffix as usize;
    if row >= count {
        return None;
    }

    let start = COSTUME_PARAM_ROW_BASE + row * COSTUME_PARAM_ROW_STRIDE;
    let end = start + COSTUME_PARAM_ROW_STRIDE;
    let row_bytes = bytes.get(start..end)?;
    let present = row_bytes
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes(chunk.try_into().unwrap()))
        .any(|value| value != 0 && value != -1);
    let first_i16 = row_bytes
        .chunks_exact(2)
        .take(12)
        .map(|chunk| i16::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    Some(CostumeParamRow { present, first_i16 })
}

impl StringRegistry {
    fn name(&self, section: usize, index: usize) -> Option<&str> {
        self.sections.get(section)?.get(index).map(String::as_str)
    }
}

fn owners_json(owners: &[OwnerDump], entry58: &[u8], entry39: &[u8]) -> String {
    format!(
        "[{}]",
        owners
            .iter()
            .map(|owner| owner_json(owner, entry58, entry39))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn owner_json(owner: &OwnerDump, entry58: &[u8], entry39: &[u8]) -> String {
    format!(
        "{{\"owner_id\":{},\"owner_name\":\"{}\",\"stem\":\"{}\",\"model_count\":{},\"layout_count\":{},\"dlc_count\":{},\"model_rows\":{},\"costume_layouts\":{},\"dlc_costumes\":{}}}",
        owner.owner_id,
        json_str(&owner.owner_name),
        json_str(&owner.stem),
        owner.model_rows.len(),
        owner.layouts.len(),
        owner.dlc_costumes.len(),
        model_rows_json(&owner.model_rows),
        layouts_json(&owner.layouts, entry58, entry39),
        dlc_json(&owner.dlc_costumes)
    )
}

fn model_rows_json(rows: &[ModelRow]) -> String {
    format!(
        "[{}]",
        rows.iter()
            .map(|row| {
                format!(
                    "{{\"row\":{},\"name\":\"{}\",\"owner\":{},\"relation\":{}}}",
                    row.row,
                    json_str(&row.name),
                    row.owner,
                    row.relation
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn layouts_json(layouts: &[CostumeLayout], entry58: &[u8], entry39: &[u8]) -> String {
    format!(
        "[{}]",
        layouts
            .iter()
            .map(|layout| layout_json(layout, entry58, entry39))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn layout_json(layout: &CostumeLayout, entry58: &[u8], entry39: &[u8]) -> String {
    let section = parse_entry58_section(entry58, layout.suffix);
    let params = parse_costume_param_row(entry39, layout.suffix);
    format!(
        "{{\"suffix\":{},\"section7_id\":{},\"name\":\"{}\",\"entry58\":{},\"entry39\":{}}}",
        layout.suffix,
        layout.section7_id,
        json_str(&layout.name),
        entry58_json(section.as_ref()),
        costume_param_json(params.as_ref())
    )
}

fn entry58_json(section: Option<&Entry58Section>) -> String {
    match section {
        Some(section) => format!(
            "{{\"record_count\":{},\"first_pairs\":{}}}",
            section.record_count,
            pairs_json(&section.first_pairs)
        ),
        None => "null".to_string(),
    }
}

fn costume_param_json(row: Option<&CostumeParamRow>) -> String {
    match row {
        Some(row) => format!(
            "{{\"present\":{},\"first_i16\":{}}}",
            row.present,
            i16_array_json(&row.first_i16)
        ),
        None => "null".to_string(),
    }
}

fn dlc_json(costumes: &[DlcCostume]) -> String {
    format!(
        "[{}]",
        costumes
            .iter()
            .map(|costume| {
                format!(
                    "{{\"code\":\"{}\",\"character_id\":{},\"costume_index\":{}}}",
                    json_str(&costume.code),
                    costume.character_id,
                    costume.costume_index
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn pairs_json(pairs: &[(u32, u32)]) -> String {
    format!(
        "[{}]",
        pairs
            .iter()
            .map(|(left, right)| format!("[{left},{right}]"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn i16_array_json(values: &[i16]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(i16::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn ascii_strings(bytes: &[u8], min_len: usize) -> Vec<String> {
    let mut strings = Vec::new();
    let mut start = None;
    for (index, &byte) in bytes.iter().enumerate() {
        if byte.is_ascii_graphic() || byte == b' ' {
            start.get_or_insert(index);
        } else if let Some(begin) = start.take() {
            if index - begin >= min_len {
                strings.push(String::from_utf8_lossy(&bytes[begin..index]).to_string());
            }
        }
    }
    if let Some(begin) = start {
        if bytes.len() - begin >= min_len {
            strings.push(String::from_utf8_lossy(&bytes[begin..]).to_string());
        }
    }
    strings
}

fn trim_nul(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .rposition(|&byte| byte != 0)
        .map(|index| index + 1)
        .unwrap_or(0);
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let Some(slice) = bytes.get(offset..offset + 4) else {
        return Err(format!("u32 read out of bounds at 0x{offset:x}"));
    };
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

fn read_i16_or(bytes: &[u8], offset: usize, fallback: i16) -> i16 {
    bytes
        .get(offset..offset + 2)
        .map(|slice| i16::from_le_bytes(slice.try_into().unwrap()))
        .unwrap_or(fallback)
}

fn next_arg(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {option}\n{}", usage()))
}

fn parse_value_list(raw: String) -> Result<Vec<u32>, String> {
    raw.split(',')
        .filter(|part| !part.trim().is_empty())
        .map(|part| parse_u32(part.trim()))
        .collect()
}

fn parse_u32(raw: &str) -> Result<u32, String> {
    let hex = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X"));
    match hex {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => raw.parse(),
    }
    .map_err(|_| format!("invalid integer: {raw}"))
}

fn json_str(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            _ => vec![ch],
        })
        .collect()
}

fn usage() -> String {
    "usage: oppw4-rdb --linkdata-costume-dump <linkdata-bin> [--owners <ids>] [--name-contains <text>]"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_costume_layout_suffix() {
        assert_eq!(
            costume_layout_suffix("806_058_costume_law_dressrosa"),
            Some(58)
        );
        assert_eq!(costume_layout_suffix("806_131_costume_law_oni"), Some(131));
        assert_eq!(costume_layout_suffix("800_026_face_law"), None);
    }

    #[test]
    fn extracts_model_stem() {
        assert_eq!(model_stem("MPLC026_Law"), Some("law".to_string()));
        assert_eq!(model_stem("MPLC012_Newgate"), Some("newgate".to_string()));
        assert_eq!(model_stem("MDLC033_Law_Souhi"), Some("law".to_string()));
    }

    #[test]
    fn parses_dlc_costume_code() {
        let parsed = parse_dlc_costume("DLC_COSTUME_006_586_026_003").unwrap();

        assert_eq!(parsed.code, "DLC_COSTUME_006_586_026_003");
        assert_eq!(parsed.character_id, 26);
        assert_eq!(parsed.costume_index, 3);
    }

    #[test]
    fn matches_layouts_to_owner_stem() {
        let layouts = vec![
            CostumeLayout {
                suffix: 44,
                section7_id: 2092,
                name: "806_044_costume_newgate".to_string(),
            },
            CostumeLayout {
                suffix: 57,
                section7_id: 2105,
                name: "806_057_costume_law".to_string(),
            },
            CostumeLayout {
                suffix: 58,
                section7_id: 2106,
                name: "806_058_costume_law_dressrosa".to_string(),
            },
        ];

        let matched = layouts_for_stem(&layouts, "law");

        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].suffix, 57);
        assert_eq!(matched[1].suffix, 58);
    }

    #[test]
    fn skips_unnamed_model_rows() {
        let mut sections = vec![Vec::new(); 7];
        sections[6] = vec!["MPLC026_Law".to_string(), "".to_string()];
        let registry = StringRegistry { sections };
        let mut bytes = vec![0u8; MODEL_ROW_BASE + MODEL_ROW_STRIDE * 2];
        bytes[0..4].copy_from_slice(&2u32.to_le_bytes());
        bytes[MODEL_ROW_BASE + 28 * 2..MODEL_ROW_BASE + 28 * 2 + 2]
            .copy_from_slice(&26i16.to_le_bytes());
        let second = MODEL_ROW_BASE + MODEL_ROW_STRIDE;
        bytes[second + 28 * 2..second + 28 * 2 + 2].copy_from_slice(&26i16.to_le_bytes());

        let rows = parse_model_rows(&bytes, &registry);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "MPLC026_Law");
    }
}
