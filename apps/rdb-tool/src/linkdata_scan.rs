use std::{
    fs,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
};

use oppw4_rdb::{inflate_linkdata_entry, parse_linkdata, LinkDataEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanConfig {
    pub linkdata_path: String,
    pub owners: Vec<u32>,
    pub models: Vec<u32>,
    pub layouts: Vec<u32>,
    pub radius: usize,
    pub threads: usize,
    pub max_matches: usize,
    pub context_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Owner,
    Model,
    Layout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pattern {
    role: Role,
    value: u32,
    width: usize,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Occurrence {
    offset: usize,
    value: u32,
    width: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntryMatch {
    entry: usize,
    inflated_size: usize,
    model: Occurrence,
    owners: Vec<Occurrence>,
    layouts: Vec<Occurrence>,
    context_start: usize,
    context_hex: String,
}

pub fn parse_command(mut args: impl Iterator<Item = String>) -> Result<ScanConfig, String> {
    let Some(linkdata_path) = args.next() else {
        return Err(usage());
    };

    let mut owners = Vec::new();
    let mut models = Vec::new();
    let mut layouts = Vec::new();
    let mut radius = 96usize;
    let mut threads = available_threads();
    let mut max_matches = 10_000usize;
    let mut context_bytes = 96usize;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--owners" => owners = parse_value_list(next_arg(&mut args, "--owners")?)?,
            "--models" => models = parse_value_list(next_arg(&mut args, "--models")?)?,
            "--layouts" => layouts = parse_value_list(next_arg(&mut args, "--layouts")?)?,
            "--radius" => radius = parse_usize(next_arg(&mut args, "--radius")?)?,
            "--threads" => threads = parse_usize(next_arg(&mut args, "--threads")?)?.max(1),
            "--max-matches" => max_matches = parse_usize(next_arg(&mut args, "--max-matches")?)?,
            "--context-bytes" => {
                context_bytes = parse_usize(next_arg(&mut args, "--context-bytes")?)?
            }
            _ => {
                return Err(format!(
                    "unknown linkdata proximity option: {arg}\n{}",
                    usage()
                ))
            }
        }
    }

    if owners.is_empty() || models.is_empty() || layouts.is_empty() {
        return Err(usage());
    }

    Ok(ScanConfig {
        linkdata_path,
        owners,
        models,
        layouts,
        radius,
        threads,
        max_matches,
        context_bytes,
    })
}

pub fn run(config: ScanConfig) -> Result<(), String> {
    let bytes = fs::read(&config.linkdata_path)
        .map_err(|error| format!("failed to read LINKDATA {}: {error}", config.linkdata_path))?;
    let index = parse_linkdata(&bytes).map_err(|error| {
        format!(
            "failed to parse LINKDATA {}: {error:?}",
            config.linkdata_path
        )
    })?;
    let entries = Arc::new(index.entries);
    let bytes = Arc::new(bytes);
    let patterns = Arc::new(build_patterns(&config));
    let next_entry = Arc::new(AtomicUsize::new(0));
    let emitted_matches = Arc::new(AtomicUsize::new(0));
    let inflate_errors = Arc::new(AtomicUsize::new(0));
    let scanned_entries = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel();

    println!("{}", scan_start_json(&config, entries.len()));

    let worker_count = config.threads.min(entries.len().max(1));
    for _ in 0..worker_count {
        spawn_worker(WorkerInputs {
            config: config.clone(),
            bytes: Arc::clone(&bytes),
            entries: Arc::clone(&entries),
            patterns: Arc::clone(&patterns),
            next_entry: Arc::clone(&next_entry),
            emitted_matches: Arc::clone(&emitted_matches),
            inflate_errors: Arc::clone(&inflate_errors),
            scanned_entries: Arc::clone(&scanned_entries),
            tx: tx.clone(),
        });
    }
    drop(tx);

    for line in rx {
        println!("{line}");
    }

    let matches = emitted_matches.load(Ordering::Relaxed);
    println!(
        "{{\"event\":\"scan_summary\",\"entries\":{},\"scanned_entries\":{},\"matches\":{},\"inflate_errors\":{},\"stopped_early\":{}}}",
        entries.len(),
        scanned_entries.load(Ordering::Relaxed),
        matches,
        inflate_errors.load(Ordering::Relaxed),
        matches >= config.max_matches
    );

    Ok(())
}

struct WorkerInputs {
    config: ScanConfig,
    bytes: Arc<Vec<u8>>,
    entries: Arc<Vec<LinkDataEntry>>,
    patterns: Arc<Vec<Pattern>>,
    next_entry: Arc<AtomicUsize>,
    emitted_matches: Arc<AtomicUsize>,
    inflate_errors: Arc<AtomicUsize>,
    scanned_entries: Arc<AtomicUsize>,
    tx: mpsc::Sender<String>,
}

fn spawn_worker(inputs: WorkerInputs) {
    thread::spawn(move || loop {
        if inputs.emitted_matches.load(Ordering::Relaxed) >= inputs.config.max_matches {
            break;
        }

        let entry_index = inputs.next_entry.fetch_add(1, Ordering::Relaxed);
        let Some(entry) = inputs.entries.get(entry_index) else {
            break;
        };

        let inflated = match inflate_linkdata_entry(&inputs.bytes, entry) {
            Ok(inflated) if !inflated.is_empty() => inflated,
            Ok(_) => {
                inputs.scanned_entries.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            Err(_) => {
                inputs.inflate_errors.fetch_add(1, Ordering::Relaxed);
                inputs.scanned_entries.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        for candidate in scan_entry(&inputs.config, entry.index, &inflated, &inputs.patterns) {
            if !reserve_match_slot(&inputs.emitted_matches, inputs.config.max_matches) {
                break;
            }
            let _ = inputs.tx.send(match_json(&candidate));
        }
        inputs.scanned_entries.fetch_add(1, Ordering::Relaxed);
    });
}

fn reserve_match_slot(emitted_matches: &AtomicUsize, max_matches: usize) -> bool {
    let mut current = emitted_matches.load(Ordering::Relaxed);
    loop {
        if current >= max_matches {
            return false;
        }
        match emitted_matches.compare_exchange_weak(
            current,
            current + 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return true,
            Err(next) => current = next,
        }
    }
}

fn scan_entry(
    config: &ScanConfig,
    entry: usize,
    bytes: &[u8],
    patterns: &[Pattern],
) -> Vec<EntryMatch> {
    let owners = collect_occurrences(bytes, patterns, Role::Owner);
    if owners.is_empty() {
        return Vec::new();
    }
    let layouts = collect_occurrences(bytes, patterns, Role::Layout);
    if layouts.is_empty() {
        return Vec::new();
    }
    let models = collect_occurrences(bytes, patterns, Role::Model);
    if models.is_empty() {
        return Vec::new();
    }

    models
        .into_iter()
        .filter_map(|model| {
            let nearby_owners = nearby_occurrences(&owners, model.offset, config.radius);
            if nearby_owners.is_empty() {
                return None;
            }
            let nearby_layouts = nearby_occurrences(&layouts, model.offset, config.radius);
            if nearby_layouts.is_empty() {
                return None;
            }
            Some(build_match(
                config,
                entry,
                bytes,
                model,
                nearby_owners,
                nearby_layouts,
            ))
        })
        .collect()
}

fn build_match(
    config: &ScanConfig,
    entry: usize,
    bytes: &[u8],
    model: Occurrence,
    owners: Vec<Occurrence>,
    layouts: Vec<Occurrence>,
) -> EntryMatch {
    let context_start = model.offset.saturating_sub(config.context_bytes / 2);
    let context_end = (context_start + config.context_bytes).min(bytes.len());

    EntryMatch {
        entry,
        inflated_size: bytes.len(),
        model,
        owners,
        layouts,
        context_start,
        context_hex: hex_string(&bytes[context_start..context_end]),
    }
}

fn collect_occurrences(bytes: &[u8], patterns: &[Pattern], role: Role) -> Vec<Occurrence> {
    let mut occurrences = Vec::new();
    for pattern in patterns.iter().filter(|pattern| pattern.role == role) {
        let mut cursor = 0usize;
        while let Some(relative) = find_bytes_from(bytes, &pattern.bytes, cursor) {
            let offset = cursor + relative;
            occurrences.push(Occurrence {
                offset,
                value: pattern.value,
                width: pattern.width,
            });
            cursor = offset + 1;
        }
    }
    occurrences.sort_by_key(|occurrence| (occurrence.offset, occurrence.value, occurrence.width));
    occurrences.dedup_by(|left, right| left.offset == right.offset && left.value == right.value);
    occurrences
}

fn nearby_occurrences(occurrences: &[Occurrence], center: usize, radius: usize) -> Vec<Occurrence> {
    occurrences
        .iter()
        .filter(|occurrence| occurrence.offset.abs_diff(center) <= radius)
        .take(12)
        .cloned()
        .collect()
}

fn build_patterns(config: &ScanConfig) -> Vec<Pattern> {
    let mut patterns = Vec::new();
    push_patterns(&mut patterns, Role::Owner, &config.owners);
    push_patterns(&mut patterns, Role::Model, &config.models);
    push_patterns(&mut patterns, Role::Layout, &config.layouts);
    patterns
}

fn push_patterns(patterns: &mut Vec<Pattern>, role: Role, values: &[u32]) {
    for &value in values {
        if let Some(bytes) = le16_bytes(value) {
            patterns.push(Pattern {
                role,
                value,
                width: 2,
                bytes,
            });
        }
        patterns.push(Pattern {
            role,
            value,
            width: 4,
            bytes: value.to_le_bytes().to_vec(),
        });
    }
}

fn le16_bytes(value: u32) -> Option<Vec<u8>> {
    u16::try_from(value)
        .ok()
        .map(|value| value.to_le_bytes().to_vec())
}

fn find_bytes_from(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start >= haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
}

fn scan_start_json(config: &ScanConfig, entries: usize) -> String {
    format!(
        "{{\"event\":\"scan_start\",\"linkdata_path\":\"{}\",\"entries\":{},\"owners\":{},\"models\":{},\"layouts\":{},\"radius\":{},\"threads\":{},\"max_matches\":{},\"context_bytes\":{}}}",
        json_str(&config.linkdata_path),
        entries,
        json_u32_array(&config.owners),
        json_u32_array(&config.models),
        json_u32_array(&config.layouts),
        config.radius,
        config.threads,
        config.max_matches,
        config.context_bytes
    )
}

fn match_json(candidate: &EntryMatch) -> String {
    format!(
        "{{\"event\":\"match\",\"entry\":{},\"inflated_size\":{},\"model\":{},\"near_owners\":{},\"near_layouts\":{},\"context_start\":{},\"context_hex\":\"{}\"}}",
        candidate.entry,
        candidate.inflated_size,
        occurrence_json(&candidate.model),
        occurrence_array_json(&candidate.owners),
        occurrence_array_json(&candidate.layouts),
        candidate.context_start,
        candidate.context_hex
    )
}

fn occurrence_json(occurrence: &Occurrence) -> String {
    format!(
        "{{\"offset\":{},\"value\":{},\"width\":{}}}",
        occurrence.offset, occurrence.value, occurrence.width
    )
}

fn occurrence_array_json(occurrences: &[Occurrence]) -> String {
    let values = occurrences
        .iter()
        .map(occurrence_json)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn json_u32_array(values: &[u32]) -> String {
    let values = values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
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

fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
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

fn parse_usize(raw: String) -> Result<usize, String> {
    parse_u32(&raw).map(|value| value as usize)
}

fn available_threads() -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
}

fn usage() -> String {
    "usage: oppw4-rdb --linkdata-proximity-scan <linkdata-bin> --owners <ids> --models <ids> --layouts <ids> [--radius <bytes>] [--threads <n>] [--max-matches <n>] [--context-bytes <n>]"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_decimal_and_hex_value_lists() {
        assert_eq!(
            parse_value_list("22,0x1a,308".to_string()).unwrap(),
            vec![22, 26, 308]
        );
    }

    #[test]
    fn builds_u16_and_u32_patterns_when_possible() {
        let config = ScanConfig {
            linkdata_path: "x".to_string(),
            owners: vec![26],
            models: vec![0x1_0000],
            layouts: vec![2106],
            radius: 96,
            threads: 1,
            max_matches: 10,
            context_bytes: 32,
        };

        let patterns = build_patterns(&config);

        assert!(patterns
            .iter()
            .any(|pattern| pattern.role == Role::Owner && pattern.width == 2));
        assert!(patterns
            .iter()
            .any(|pattern| pattern.role == Role::Owner && pattern.width == 4));
        assert!(patterns
            .iter()
            .all(|pattern| pattern.value != 0x1_0000 || pattern.width == 4));
    }

    #[test]
    fn finds_model_with_near_owner_and_layout() {
        let bytes = [
            0x1a, 0x00, // owner 26
            0x00, 0x00, 0xe3, 0x00, // model 227
            0x00, 0x00, 0x3a, 0x00, // layout 58
        ];
        let config = ScanConfig {
            linkdata_path: "x".to_string(),
            owners: vec![26],
            models: vec![227],
            layouts: vec![58],
            radius: 8,
            threads: 1,
            max_matches: 10,
            context_bytes: 16,
        };
        let patterns = build_patterns(&config);

        let matches = scan_entry(&config, 123, &bytes, &patterns);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].entry, 123);
        assert_eq!(matches[0].model.value, 227);
        assert_eq!(matches[0].owners[0].value, 26);
        assert_eq!(matches[0].layouts[0].value, 58);
    }
}
