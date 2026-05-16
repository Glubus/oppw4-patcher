use std::{
    borrow::Cow,
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

const CONFIG_FILE: &str = "linkdata_ram_probe.txt";
const DEFAULT_INTERVAL_MS: u64 = 2_000;
const DEFAULT_PASSES: usize = 45;
const DEFAULT_MAX_HITS: usize = 80;
const DEFAULT_MAX_PATCHES: usize = 8;
const DEFAULT_CONTEXT_BYTES: usize = 0;
const DEFAULT_MIN_REGION_SIZE: usize = 0;
const READ_CHUNK_SIZE: usize = 64 * 1024;
const ENTRY32_COHERENCE_RADIUS: usize = 128 * 1024;

const DEFAULT_PATTERNS: &[(&str, &[u8])] = &[
    ("newgate_model", b"MPLC012_Newgate\0"),
    ("newgate_layout", b"806_044_costume_newgate\0"),
    ("hancock_dlc_layout", b"806_112_costume_hancock_ougenki\0"),
    ("law_dressrosa_layout", b"806_058_costume_law_dressrosa\0"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
    pub protect: u32,
}

#[derive(Clone, Copy)]
pub struct ProbeHost {
    pub writable_memory_regions: fn() -> Vec<MemoryRegion>,
    pub readable_memory_regions: fn() -> Vec<MemoryRegion>,
    pub read_process_memory: fn(usize, &mut [u8]) -> Option<usize>,
    pub write_process_memory: fn(usize, &[u8]) -> Option<usize>,
    pub log: fn(String),
}

#[derive(Debug, Clone)]
struct ProbeConfig {
    path: PathBuf,
    interval: Duration,
    passes: usize,
    max_hits: usize,
    max_patches: usize,
    context_bytes: usize,
    min_region_size: usize,
    scan_readable: bool,
    skip_probe_owned_memory: bool,
    patterns: Vec<ProbePattern>,
    patches: Vec<RamPatch>,
}

#[derive(Debug, Clone)]
struct ProbePattern {
    name: Cow<'static, str>,
    bytes: Cow<'static, [u8]>,
}

#[derive(Debug, Clone)]
struct RamPatch {
    name: Cow<'static, str>,
    from: Cow<'static, [u8]>,
    to: Cow<'static, [u8]>,
    entry32_offset: Option<usize>,
}

struct Entry32Anchor {
    offset: usize,
    bytes: &'static [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatchCoherence {
    Generic,
    Entry32Base(usize),
    AnchorWindow,
    NearbyAnchorWindow,
}

const ENTRY32_COHERENCE_ANCHORS: &[Entry32Anchor] = &[
    Entry32Anchor {
        offset: 0x3e4ca,
        bytes: b"806_044_costume_newgate\0",
    },
    Entry32Anchor {
        offset: 0x3e612,
        bytes: b"806_058_costume_law_dressrosa\0",
    },
    Entry32Anchor {
        offset: 0x3eb93,
        bytes: b"806_112_costume_hancock_ougenki\0",
    },
];

pub fn start_if_configured(config_root: PathBuf, host: ProbeHost) {
    let config_path = config_root.join(CONFIG_FILE);
    let Some(config) = ProbeConfig::load(&config_path, host) else {
        return;
    };
    (host.log)(format!(
        "LinkData RAM probe enabled: path={} patterns={} patches={} passes={} interval_ms={} max_hits={} max_patches={} scan={}",
        config.path.display(),
        config.patterns.len(),
        config.patches.len(),
        config.passes,
        config.interval.as_millis(),
        config.max_hits,
        config.max_patches,
        if config.scan_readable { "readable" } else { "writable" }
    ));
    if let Err(error) = thread::Builder::new()
        .name("oppw4-linkdata-ram-probe".to_string())
        .spawn(move || run_probe(config, host))
    {
        (host.log)(format!("LinkData RAM probe thread failed: {error}"));
    }
}

fn run_probe(config: ProbeConfig, host: ProbeHost) {
    thread::sleep(config.interval);
    let mut seen_hits = HashSet::new();
    let mut seen_patches = HashSet::new();
    let mut seen_skips = HashSet::new();
    for pass in 1..=config.passes {
        let hits_before = seen_hits.len();
        let patches_before = seen_patches.len();
        scan_once(
            &config,
            host,
            &mut seen_hits,
            &mut seen_patches,
            &mut seen_skips,
        );
        let new_hits = seen_hits.len().saturating_sub(hits_before);
        let new_patches = seen_patches.len().saturating_sub(patches_before);
        if new_hits != 0 || new_patches != 0 {
            (host.log)(format!(
                "LinkData RAM probe pass={pass} new_hits={new_hits} new_patches={new_patches} total_hits={} total_patches={}",
                seen_hits.len(),
                seen_patches.len()
            ));
        }
        if seen_hits.len() >= config.max_hits && seen_patches.len() >= config.max_patches {
            (host.log)("LinkData RAM probe stopped: max hits and patches reached".to_string());
            return;
        }
        thread::sleep(config.interval);
    }
    (host.log)(format!(
        "LinkData RAM probe finished: passes={} total_hits={} total_patches={}",
        config.passes,
        seen_hits.len(),
        seen_patches.len()
    ));
}

fn scan_once(
    config: &ProbeConfig,
    host: ProbeHost,
    seen_hits: &mut HashSet<(usize, usize)>,
    seen_patches: &mut HashSet<(usize, usize)>,
    seen_skips: &mut HashSet<(usize, usize)>,
) {
    let patterns = &config.patterns;
    if patterns.is_empty()
        || (seen_hits.len() >= config.max_hits && seen_patches.len() >= config.max_patches)
    {
        return;
    }
    let max_pattern_len = patterns
        .iter()
        .map(|pattern| pattern.bytes.len())
        .max()
        .unwrap_or(1);
    let regions = if config.scan_readable {
        (host.readable_memory_regions)()
    } else {
        (host.writable_memory_regions)()
    };
    for region in regions {
        if region.size < config.min_region_size {
            continue;
        }
        scan_region(
            region,
            config,
            host,
            max_pattern_len,
            seen_hits,
            seen_patches,
            seen_skips,
        );
        if seen_hits.len() >= config.max_hits && seen_patches.len() >= config.max_patches {
            return;
        }
    }
}

fn scan_region(
    region: MemoryRegion,
    config: &ProbeConfig,
    host: ProbeHost,
    max_pattern_len: usize,
    seen_hits: &mut HashSet<(usize, usize)>,
    seen_patches: &mut HashSet<(usize, usize)>,
    seen_skips: &mut HashSet<(usize, usize)>,
) {
    if region.size < max_pattern_len {
        return;
    }

    let overlap = max_pattern_len.saturating_sub(1);
    let mut carry = Vec::new();
    let mut offset = 0usize;
    while offset < region.size {
        let remaining = region.size - offset;
        let read_len = remaining.min(READ_CHUNK_SIZE);
        let mut buffer = vec![0u8; carry.len() + read_len];
        buffer[..carry.len()].copy_from_slice(&carry);
        let read = (host.read_process_memory)(
            region.base.saturating_add(offset),
            &mut buffer[carry.len()..],
        )
        .unwrap_or(0);
        buffer.truncate(carry.len() + read);

        if read != 0 {
            let window_base = region
                .base
                .saturating_add(offset)
                .saturating_sub(carry.len());
            log_pattern_hits(region, window_base, &buffer, config, seen_hits, host);
            apply_patch_hits(
                region,
                window_base,
                &buffer,
                config,
                host,
                config.skip_probe_owned_memory,
                seen_patches,
                seen_skips,
            );
        }
        if seen_hits.len() >= config.max_hits && seen_patches.len() >= config.max_patches {
            return;
        }

        let keep = buffer.len().min(overlap);
        carry.clear();
        carry.extend_from_slice(&buffer[buffer.len().saturating_sub(keep)..]);
        offset = offset.saturating_add(read_len);
    }
}

fn apply_patch_hits(
    region: MemoryRegion,
    window_base: usize,
    buffer: &[u8],
    config: &ProbeConfig,
    host: ProbeHost,
    skip_probe_owned_memory: bool,
    seen_patches: &mut HashSet<(usize, usize)>,
    seen_skips: &mut HashSet<(usize, usize)>,
) {
    if config.patches.is_empty() || seen_patches.len() >= config.max_patches {
        return;
    }
    for (patch_index, patch) in config.patches.iter().enumerate() {
        for position in find_all(buffer, patch.from.as_ref()) {
            let address = window_base.saturating_add(position);
            if skip_probe_owned_memory && is_probe_owned_address(config, address) {
                continue;
            }
            let Some(coherence) = patch_coherence(region, address, patch, host, buffer) else {
                if seen_skips.insert((patch_index, address)) {
                    (host.log)(format!(
                        "LinkData RAM patch skipped name={} address=0x{address:x} reason=entry32_coherence_failed",
                        patch.name
                    ));
                }
                continue;
            };
            if !seen_patches.insert((patch_index, address)) {
                continue;
            }
            let written = (host.write_process_memory)(address, patch.to.as_ref()).unwrap_or(0);
            if written == patch.to.len() {
                log_patch_success(region, address, written, patch, coherence, host);
            } else {
                (host.log)(format!(
                    "LinkData RAM patch failed name={} address=0x{address:x} requested=0x{:x} written=0x{written:x}",
                    patch.name,
                    patch.to.len()
                ));
            }
            if seen_patches.len() >= config.max_patches {
                return;
            }
        }
    }
}

fn log_patch_success(
    region: MemoryRegion,
    address: usize,
    written: usize,
    patch: &RamPatch,
    coherence: PatchCoherence,
    host: ProbeHost,
) {
    match coherence {
        PatchCoherence::Entry32Base(base) => (host.log)(format!(
            "LinkData RAM patch name={} address=0x{address:x} entry32_base=0x{base:x} bytes=0x{written:x} region=0x{:x}+0x{:x}",
            patch.name, region.base, region.size
        )),
        PatchCoherence::AnchorWindow => (host.log)(format!(
            "LinkData RAM patch name={} address=0x{address:x} coherence=anchor_window bytes=0x{written:x} region=0x{:x}+0x{:x}",
            patch.name, region.base, region.size
        )),
        PatchCoherence::NearbyAnchorWindow => (host.log)(format!(
            "LinkData RAM patch name={} address=0x{address:x} coherence=nearby_anchor_window bytes=0x{written:x} region=0x{:x}+0x{:x}",
            patch.name, region.base, region.size
        )),
        PatchCoherence::Generic => (host.log)(format!(
            "LinkData RAM patch name={} address=0x{address:x} bytes=0x{written:x} region=0x{:x}+0x{:x}",
            patch.name, region.base, region.size
        )),
    }
}

fn patch_coherence(
    region: MemoryRegion,
    address: usize,
    patch: &RamPatch,
    host: ProbeHost,
    buffer: &[u8],
) -> Option<PatchCoherence> {
    if patch.entry32_offset.is_none() {
        return Some(PatchCoherence::Generic);
    }
    let base = entry32_base(address, patch)?;
    if entry32_base_coherence_passes(base, host) {
        return Some(PatchCoherence::Entry32Base(base));
    }
    if entry32_anchor_window_passes(buffer) {
        return Some(PatchCoherence::AnchorWindow);
    }
    entry32_nearby_anchor_window_passes(region, address, host)
        .then_some(PatchCoherence::NearbyAnchorWindow)
}

fn entry32_base_coherence_passes(base: usize, host: ProbeHost) -> bool {
    ENTRY32_COHERENCE_ANCHORS
        .iter()
        .all(|anchor| memory_matches(base.saturating_add(anchor.offset), anchor.bytes, host))
}

fn entry32_anchor_window_passes(buffer: &[u8]) -> bool {
    ENTRY32_COHERENCE_ANCHORS
        .iter()
        .all(|anchor| !find_all(buffer, anchor.bytes).is_empty())
}

fn entry32_nearby_anchor_window_passes(
    region: MemoryRegion,
    address: usize,
    host: ProbeHost,
) -> bool {
    let region_end = region.base.saturating_add(region.size);
    let start = address
        .saturating_sub(ENTRY32_COHERENCE_RADIUS)
        .max(region.base);
    let end = address
        .saturating_add(ENTRY32_COHERENCE_RADIUS)
        .min(region_end);
    let len = end.saturating_sub(start);
    if len == 0 {
        return false;
    }
    let mut buffer = vec![0; len];
    let read = (host.read_process_memory)(start, &mut buffer).unwrap_or(0);
    buffer.truncate(read);
    entry32_anchor_window_passes(&buffer)
}

fn entry32_base(address: usize, patch: &RamPatch) -> Option<usize> {
    let offset = patch.entry32_offset?;
    address.checked_sub(offset)
}

fn memory_matches(address: usize, expected: &[u8], host: ProbeHost) -> bool {
    let mut bytes = vec![0; expected.len()];
    let read = (host.read_process_memory)(address, &mut bytes).unwrap_or(0);
    read == expected.len() && bytes == expected
}

fn log_pattern_hits(
    region: MemoryRegion,
    window_base: usize,
    buffer: &[u8],
    config: &ProbeConfig,
    seen: &mut HashSet<(usize, usize)>,
    host: ProbeHost,
) {
    for (pattern_index, pattern) in config.patterns.iter().enumerate() {
        for position in find_all(buffer, pattern.bytes.as_ref()) {
            let address = window_base.saturating_add(position);
            if config.skip_probe_owned_memory && is_probe_owned_address(config, address) {
                continue;
            }
            if !seen.insert((pattern_index, address)) {
                continue;
            }
            let context = hit_context(buffer, position, pattern.bytes.len(), config.context_bytes);
            let context_suffix = context
                .map(|(start, bytes)| {
                    format!(
                        " context_start=0x{:x} context={}",
                        window_base.saturating_add(start),
                        hex_preview(bytes)
                    )
                })
                .unwrap_or_default();
            (host.log)(format!(
                "LinkData RAM hit pattern={} address=0x{address:x} region=0x{:x}+0x{:x} protect=0x{:x}",
                pattern.name,
                region.base,
                region.size,
                region.protect,
            ));
            if !context_suffix.is_empty() {
                (host.log)(format!(
                    "LinkData RAM hit context pattern={} address=0x{address:x}{context_suffix}",
                    pattern.name
                ));
            }
            if seen.len() >= config.max_hits {
                return;
            }
        }
    }
}

fn is_probe_owned_address(config: &ProbeConfig, address: usize) -> bool {
    config
        .patterns
        .iter()
        .any(|pattern| slice_contains_address(pattern.bytes.as_ref(), address))
        || config.patches.iter().any(|patch| {
            slice_contains_address(patch.from.as_ref(), address)
                || slice_contains_address(patch.to.as_ref(), address)
        })
}

fn slice_contains_address(bytes: &[u8], address: usize) -> bool {
    let start = bytes.as_ptr() as usize;
    let end = start.saturating_add(bytes.len());
    start <= address && address < end
}

fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return Vec::new();
    }
    haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index))
        .collect()
}

impl ProbeConfig {
    fn load(path: &Path, host: ProbeHost) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                (host.log)(format!(
                    "LinkData RAM probe config unreadable: {} ({error})",
                    path.display()
                ));
                String::new()
            }
        };
        parse_config(path.to_path_buf(), &text)
    }
}

fn parse_config(path: PathBuf, text: &str) -> Option<ProbeConfig> {
    let mut enabled = true;
    let mut interval_ms = DEFAULT_INTERVAL_MS;
    let mut passes = DEFAULT_PASSES;
    let mut max_hits = DEFAULT_MAX_HITS;
    let mut max_patches = DEFAULT_MAX_PATCHES;
    let mut context_bytes = DEFAULT_CONTEXT_BYTES;
    let mut min_region_size = DEFAULT_MIN_REGION_SIZE;
    let mut scan_readable = false;
    let mut skip_probe_owned_memory = false;
    let mut patterns = Vec::new();
    let mut patches = Vec::new();

    for line in text.lines().map(strip_comment).map(str::trim) {
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        match key.as_str() {
            "enabled" => enabled = parse_bool(value).unwrap_or(enabled),
            "interval_ms" => interval_ms = value.parse().unwrap_or(interval_ms),
            "passes" => passes = value.parse().unwrap_or(passes),
            "max_hits" => max_hits = value.parse().unwrap_or(max_hits),
            "max_patches" => max_patches = value.parse().unwrap_or(max_patches),
            "context_bytes" => context_bytes = value.parse().unwrap_or(context_bytes),
            "min_region_size" => {
                min_region_size = parse_usize_number(value).unwrap_or(min_region_size)
            }
            "scan_readable" => scan_readable = parse_bool(value).unwrap_or(scan_readable),
            "skip_probe_owned_memory" => {
                skip_probe_owned_memory = parse_bool(value).unwrap_or(skip_probe_owned_memory)
            }
            "pattern" => {
                if !value.is_empty() {
                    patterns.push(config_pattern(value));
                }
            }
            "pattern_hex" => {
                if let Some(pattern) = config_hex_pattern(value) {
                    patterns.push(pattern);
                }
            }
            "pattern_utf16" => {
                if let Some(pattern) = config_utf16_pattern(value) {
                    patterns.push(pattern);
                }
            }
            "patch" => {
                if let Some(patch) = config_patch(value) {
                    patches.push(patch);
                }
            }
            "patch_hex" => {
                if let Some(patch) = config_hex_patch(value) {
                    patches.push(patch);
                }
            }
            "entry32_patch" => {
                if let Some(patch) = config_entry32_patch(value) {
                    patches.push(patch);
                }
            }
            _ => {}
        }
    }

    if !enabled {
        return None;
    }
    if patterns.is_empty() {
        patterns = default_patterns();
    }
    for patch in &patches {
        append_unique_pattern(&mut patterns, patch.name.clone(), patch.from.clone());
    }
    Some(ProbeConfig {
        path,
        interval: Duration::from_millis(interval_ms.max(1)),
        passes: passes.max(1),
        max_hits: max_hits.max(1),
        max_patches: max_patches.max(1),
        context_bytes,
        min_region_size,
        scan_readable,
        skip_probe_owned_memory,
        patterns,
        patches,
    })
}

fn default_patterns() -> Vec<ProbePattern> {
    DEFAULT_PATTERNS
        .iter()
        .map(|(name, bytes)| ProbePattern {
            name: Cow::Borrowed(*name),
            bytes: Cow::Borrowed(*bytes),
        })
        .collect()
}

fn config_pattern(value: &str) -> ProbePattern {
    ProbePattern {
        name: Cow::Owned(value.to_string()),
        bytes: Cow::Owned(bytes_with_nul(value)),
    }
}

fn config_hex_pattern(value: &str) -> Option<ProbePattern> {
    let (name, raw_hex) = value.split_once(':')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    Some(ProbePattern {
        name: Cow::Owned(name.to_string()),
        bytes: Cow::Owned(parse_hex_bytes(raw_hex)?),
    })
}

fn config_utf16_pattern(value: &str) -> Option<ProbePattern> {
    let (name, text) = value
        .split_once(':')
        .map(|(name, text)| (name.trim().to_string(), text.trim()))
        .unwrap_or_else(|| (format!("{}_u16", value.trim()), value.trim()));
    if name.is_empty() || text.is_empty() {
        return None;
    }
    Some(ProbePattern {
        name: Cow::Owned(name),
        bytes: Cow::Owned(utf16_bytes_with_nul(text)),
    })
}

fn config_patch(value: &str) -> Option<RamPatch> {
    let (from, to) = value.split_once("=>")?;
    config_patch_payload(from, to, None, false)
}

fn config_hex_patch(value: &str) -> Option<RamPatch> {
    let (name, payload) = value.split_once(':')?;
    let (from, to) = payload.split_once("=>")?;
    let from = parse_hex_bytes(from)?;
    let to = parse_hex_bytes(to)?;
    if name.trim().is_empty() || from.is_empty() || from.len() != to.len() {
        return None;
    }
    Some(RamPatch {
        name: Cow::Owned(name.trim().to_string()),
        from: Cow::Owned(from),
        to: Cow::Owned(to),
        entry32_offset: None,
    })
}

fn config_entry32_patch(value: &str) -> Option<RamPatch> {
    let (raw_offset, payload) = value.split_once(':')?;
    let (from, to) = payload.split_once("=>")?;
    config_patch_payload(from, to, Some(parse_usize_number(raw_offset.trim())?), true)
}

fn config_patch_payload(
    from: &str,
    to: &str,
    entry32_offset: Option<usize>,
    allow_shorter_target: bool,
) -> Option<RamPatch> {
    let from = from.trim();
    let to = to.trim();
    if from.is_empty() || to.is_empty() {
        return None;
    }
    let from_bytes = bytes_with_nul(from);
    let mut to_bytes = bytes_with_nul(to);
    if allow_shorter_target && to_bytes.len() <= from_bytes.len() {
        to_bytes.resize(from_bytes.len(), 0);
    }
    if from_bytes.len() != to_bytes.len() {
        return None;
    }
    Some(RamPatch {
        name: Cow::Owned(format!("{from}->{to}")),
        from: Cow::Owned(from_bytes),
        to: Cow::Owned(to_bytes),
        entry32_offset,
    })
}

fn parse_usize_number(raw: &str) -> Option<usize> {
    let hex = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X"));
    match hex {
        Some(hex) => usize::from_str_radix(hex, 16).ok(),
        None => raw.parse().ok(),
    }
}

fn append_unique_pattern(
    patterns: &mut Vec<ProbePattern>,
    name: Cow<'static, str>,
    bytes: Cow<'static, [u8]>,
) {
    if patterns
        .iter()
        .any(|pattern| pattern.bytes.as_ref() == bytes.as_ref())
    {
        return;
    }
    patterns.push(ProbePattern { name, bytes });
}

fn bytes_with_nul(value: &str) -> Vec<u8> {
    let mut bytes = value.as_bytes().to_vec();
    if !bytes.ends_with(&[0]) {
        bytes.push(0);
    }
    bytes
}

fn utf16_bytes_with_nul(value: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for unit in value.encode_utf16().chain(std::iter::once(0)) {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

fn parse_hex_bytes(raw: &str) -> Option<Vec<u8>> {
    let mut cleaned = String::new();
    for part in raw
        .split(|ch: char| ch.is_ascii_whitespace() || ch == '_' || ch == ',')
        .filter(|part| !part.is_empty())
    {
        cleaned.push_str(
            part.strip_prefix("0x")
                .or_else(|| part.strip_prefix("0X"))
                .unwrap_or(part),
        );
    }
    if cleaned.is_empty() || cleaned.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(cleaned.len() / 2);
    for index in (0..cleaned.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&cleaned[index..index + 2], 16).ok()?);
    }
    Some(bytes)
}

fn hit_context(
    buffer: &[u8],
    position: usize,
    pattern_len: usize,
    context_bytes: usize,
) -> Option<(usize, &[u8])> {
    if context_bytes == 0 {
        return None;
    }
    let before = context_bytes / 2;
    let start = position.saturating_sub(before);
    let end = position
        .saturating_add(pattern_len)
        .saturating_add(context_bytes.saturating_sub(before))
        .min(buffer.len());
    Some((start, &buffer[start..end]))
}

fn hex_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#').map(|(value, _)| value).unwrap_or(line)
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Mutex, time::Duration};

    use super::*;

    static TEST_MEMORY: Mutex<Option<TestMemory>> = Mutex::new(None);

    struct TestMemory {
        base: usize,
        bytes: Vec<u8>,
    }

    #[test]
    fn disabled_config_does_not_start_probe() {
        assert!(parse_config(PathBuf::from("probe.txt"), "enabled=0").is_none());
    }

    #[test]
    fn patch_config_is_opt_in_and_adds_source_pattern() {
        let config = parse_config(
            PathBuf::from("probe.txt"),
            "patch=803_012_name_newgate=>803_012_name_NEWGATE",
        )
        .unwrap();

        assert_eq!(config.patches.len(), 1);
        assert_eq!(config.interval, Duration::from_millis(DEFAULT_INTERVAL_MS));
        assert!(config
            .patterns
            .iter()
            .any(|pattern| pattern.bytes.as_ref() == b"803_012_name_newgate\0"));
    }

    #[test]
    fn mismatched_patch_sizes_are_ignored() {
        let config = parse_config(PathBuf::from("probe.txt"), "patch=short=>too_long").unwrap();

        assert!(config.patches.is_empty());
    }

    #[test]
    fn entry32_patch_parses_source_offset_and_pads_shorter_target() {
        let config = parse_config(
            PathBuf::from("probe.txt"),
            "entry32_patch=0x40fc3:821_028_eula_french_pg05=>806_044_costume_newgate",
        )
        .unwrap();

        assert_eq!(config.patches.len(), 1);
        assert_eq!(config.patches[0].entry32_offset, Some(0x40fc3));
        assert_eq!(config.patches[0].from.as_ref().len(), 25);
        assert_eq!(config.patches[0].to.as_ref().len(), 25);
        assert!(config.patches[0]
            .to
            .as_ref()
            .starts_with(b"806_044_costume_newgate\0"));
    }

    #[test]
    fn hex_pattern_config_parses_raw_bytes_without_nul() {
        let config = parse_config(
            PathBuf::from("probe.txt"),
            "pattern_hex=law_suffixes_u16:39003a008300",
        )
        .unwrap();

        assert!(config.patterns.iter().any(|pattern| {
            pattern.name.as_ref() == "law_suffixes_u16"
                && pattern.bytes.as_ref() == [0x39, 0x00, 0x3a, 0x00, 0x83, 0x00]
        }));
    }

    #[test]
    fn utf16_pattern_config_parses_wide_nul_string() {
        let config = parse_config(
            PathBuf::from("probe.txt"),
            "pattern_utf16=DLC_CHARACTER_018_972",
        )
        .unwrap();

        assert!(config.patterns.iter().any(|pattern| {
            pattern.name.as_ref() == "DLC_CHARACTER_018_972_u16"
                && pattern.bytes.as_ref()
                    == [
                        0x44, 0x00, 0x4c, 0x00, 0x43, 0x00, 0x5f, 0x00, 0x43, 0x00, 0x48, 0x00,
                        0x41, 0x00, 0x52, 0x00, 0x41, 0x00, 0x43, 0x00, 0x54, 0x00, 0x45, 0x00,
                        0x52, 0x00, 0x5f, 0x00, 0x30, 0x00, 0x31, 0x00, 0x38, 0x00, 0x5f, 0x00,
                        0x39, 0x00, 0x37, 0x00, 0x32, 0x00, 0x00, 0x00,
                    ]
        }));
    }

    #[test]
    fn min_region_size_and_owned_memory_filter_are_configurable() {
        let config = parse_config(
            PathBuf::from("probe.txt"),
            "min_region_size=0x100000\nskip_probe_owned_memory=1",
        )
        .unwrap();

        assert_eq!(config.min_region_size, 0x100000);
        assert!(config.skip_probe_owned_memory);
    }

    #[test]
    fn readable_memory_scan_is_configurable() {
        let config = parse_config(PathBuf::from("probe.txt"), "scan_readable=1").unwrap();

        assert!(config.scan_readable);
    }

    #[test]
    fn hex_patch_config_adds_source_pattern() {
        let config = parse_config(
            PathBuf::from("probe.txt"),
            "patch_hex=law_duplicate:39003a008300=>39003a003a00",
        )
        .unwrap();

        assert_eq!(config.patches.len(), 1);
        assert_eq!(config.patches[0].name.as_ref(), "law_duplicate");
        assert_eq!(
            config.patches[0].from.as_ref(),
            &[0x39, 0x00, 0x3a, 0x00, 0x83, 0x00]
        );
        assert_eq!(
            config.patches[0].to.as_ref(),
            &[0x39, 0x00, 0x3a, 0x00, 0x3a, 0x00]
        );
        assert!(config.patterns.iter().any(|pattern| {
            pattern.name.as_ref() == "law_duplicate"
                && pattern.bytes.as_ref() == [0x39, 0x00, 0x3a, 0x00, 0x83, 0x00]
        }));
    }

    #[test]
    fn probe_owned_pattern_memory_is_not_reported_as_a_game_hit() {
        let config = parse_config(
            PathBuf::from("probe.txt"),
            "pattern_hex=law_active_suffixes_u16:39003a008300",
        )
        .unwrap();
        let pattern = config
            .patterns
            .iter()
            .find(|pattern| pattern.name.as_ref() == "law_active_suffixes_u16")
            .unwrap();
        let pattern_address = pattern.bytes.as_ref().as_ptr() as usize;

        assert!(is_probe_owned_address(&config, pattern_address));
        assert!(is_probe_owned_address(&config, pattern_address + 2));
        assert!(!is_probe_owned_address(&config, pattern_address - 1));
        assert!(!is_probe_owned_address(
            &config,
            pattern_address + pattern.bytes.len()
        ));
    }

    #[test]
    fn entry32_base_rejects_addresses_before_source_offset() {
        let patch =
            config_entry32_patch("0x40fc3:821_028_eula_french_pg05=>806_044_costume_newgate")
                .unwrap();

        assert_eq!(entry32_base(0x40fc2, &patch), None);
        assert_eq!(entry32_base(0x50000, &patch), Some(0xf03d));
    }

    #[test]
    fn entry32_anchor_window_requires_all_known_costume_anchors() {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(b"noise");
        buffer.extend_from_slice(b"806_044_costume_newgate\0");
        buffer.extend_from_slice(b"more noise");
        buffer.extend_from_slice(b"806_058_costume_law_dressrosa\0");
        buffer.extend_from_slice(b"even more noise");
        buffer.extend_from_slice(b"806_112_costume_hancock_ougenki\0");

        assert!(entry32_anchor_window_passes(&buffer));

        buffer.truncate(b"noise806_044_costume_newgate\0".len());
        assert!(!entry32_anchor_window_passes(&buffer));
    }

    #[test]
    fn entry32_coherence_reads_nearby_region_when_anchors_cross_scan_chunks() {
        let patch = config_entry32_patch(
            "0x41bfb:826_031_eula_mss_english_pg10=>806_058_costume_law_dressrosa",
        )
        .unwrap();
        let region = MemoryRegion {
            base: 0x1000_0000,
            size: 0x60000,
            protect: 0x4,
        };
        let source_offset = READ_CHUNK_SIZE - 0x20;
        let source_address = region.base + source_offset;
        let mut bytes = vec![0; region.size];

        write_test_bytes(&mut bytes, source_offset, patch.from.as_ref());
        write_test_bytes(
            &mut bytes,
            source_offset + 0x8000,
            b"806_044_costume_newgate\0",
        );
        write_test_bytes(
            &mut bytes,
            source_offset + 0x8600,
            b"806_058_costume_law_dressrosa\0",
        );
        write_test_bytes(
            &mut bytes,
            source_offset + 0x8c00,
            b"806_112_costume_hancock_ougenki\0",
        );
        set_test_memory(region.base, bytes);

        assert_eq!(
            patch_coherence(
                region,
                source_address,
                &patch,
                test_host(),
                patch.from.as_ref()
            ),
            Some(PatchCoherence::NearbyAnchorWindow)
        );
    }

    #[test]
    fn finds_all_pattern_offsets() {
        assert_eq!(find_all(b"abc abc abc", b"abc"), vec![0, 4, 8]);
    }

    fn write_test_bytes(buffer: &mut [u8], offset: usize, bytes: &[u8]) {
        buffer[offset..offset + bytes.len()].copy_from_slice(bytes);
    }

    fn set_test_memory(base: usize, bytes: Vec<u8>) {
        *TEST_MEMORY.lock().unwrap() = Some(TestMemory { base, bytes });
    }

    fn test_host() -> ProbeHost {
        ProbeHost {
            writable_memory_regions: test_regions,
            readable_memory_regions: test_regions,
            read_process_memory: test_read_process_memory,
            write_process_memory: test_write_process_memory,
            log: test_log,
        }
    }

    fn test_regions() -> Vec<MemoryRegion> {
        Vec::new()
    }

    fn test_read_process_memory(address: usize, output: &mut [u8]) -> Option<usize> {
        let guard = TEST_MEMORY.lock().unwrap();
        let memory = guard.as_ref()?;
        let start = address.checked_sub(memory.base)?;
        if start >= memory.bytes.len() {
            return Some(0);
        }
        let read = output.len().min(memory.bytes.len() - start);
        output[..read].copy_from_slice(&memory.bytes[start..start + read]);
        Some(read)
    }

    fn test_write_process_memory(_address: usize, bytes: &[u8]) -> Option<usize> {
        Some(bytes.len())
    }

    fn test_log(_message: String) {}
}
