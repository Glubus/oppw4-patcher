use std::{
    collections::HashSet,
    ffi::CStr,
    io::{Cursor, Read},
    path::PathBuf,
};

use oppw4_plugin_api::{cstring_lossy, Oppw4PluginApi};

use crate::{ffi, log, mods::ModRepository, patching, LEGACY_NAME_HASH_CATALOG_ZIP};

const ARCHIVES: [&str; 8] = [
    "CharacterEditor",
    "FieldEditor4",
    "KIDSSystemResource",
    "MaterialEditor",
    "RRPreview",
    "ScreenLayout",
    "SequenceEditor",
    "system",
];

pub fn initialize(api: &Oppw4PluginApi) -> i32 {
    let Some(game_root) = game_root(api) else {
        log::write_line("skin_patcher: missing game_root");
        return -3;
    };
    let Some(mods_root) = plugin_mods_root(api) else {
        log::write_line("skin_patcher: missing plugin mods root");
        return -4;
    };
    let paths = RuntimePaths::from_roots(game_root, mods_root);
    log_paths(&paths);

    let catalog = load_name_catalog(&paths);
    let plugin_zip_paths = api
        .plugin_mod_zips()
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    log::write_line(format!("plugin mod zips: {}", plugin_zip_paths.len()));
    let replacements = scan_known_archives(&paths, &catalog, plugin_zip_paths);
    let plugin_id = cstring_lossy("skin_patcher");
    let replacement_count = replacements.len();
    let registered = ffi::register_replacements(api, &plugin_id, replacements);
    log::write_line(format!(
        "skin_patcher registered replacements result={registered} count={}",
        replacement_count
    ));
    if registered < 0 {
        registered
    } else {
        0
    }
}

fn game_root(api: &Oppw4PluginApi) -> Option<PathBuf> {
    path_from_cstr(api.game_root_utf8)
}

fn plugin_mods_root(api: &Oppw4PluginApi) -> Option<PathBuf> {
    path_from_cstr(api.plugin_mods_root_utf8)
}

fn path_from_cstr(value: *const std::ffi::c_char) -> Option<PathBuf> {
    if value.is_null() {
        return None;
    }
    let value = unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned();
    (!value.is_empty()).then(|| PathBuf::from(value))
}

struct RuntimePaths {
    mods_root: PathBuf,
    config_root: PathBuf,
    rdb_root: PathBuf,
}

impl RuntimePaths {
    fn from_roots(game_root: PathBuf, mods_root: PathBuf) -> Self {
        Self {
            config_root: mods_root.join("_oppw4"),
            mods_root,
            rdb_root: game_root
                .join("File")
                .join("CMN")
                .join("AssetRelease")
                .join("Retail"),
        }
    }
}

fn log_paths(paths: &RuntimePaths) {
    log::write_line(format!("mods root: {}", paths.mods_root.display()));
    log::write_line(format!("config root: {}", paths.config_root.display()));
    log::write_line(format!("rdb root: {}", paths.rdb_root.display()));
}

fn load_name_catalog(paths: &RuntimePaths) -> Vec<oppw4_rdb::NameHashEntry> {
    let override_path = paths.config_root.join("name_hash_catalog.txt");
    if let Ok(bytes) = std::fs::read(&override_path) {
        let catalog = oppw4_rdb::parse_name_hash_catalog(&bytes);
        log::write_line(format!(
            "name catalog override: entries={} path={}",
            catalog.len(),
            override_path.display()
        ));
        return catalog;
    }

    let catalog = load_embedded_name_catalog();
    log::write_line(format!(
        "name catalog embedded: entries={} compressed=0x{:x}",
        catalog.len(),
        LEGACY_NAME_HASH_CATALOG_ZIP.len()
    ));
    catalog
}

fn load_embedded_name_catalog() -> Vec<oppw4_rdb::NameHashEntry> {
    let cursor = Cursor::new(LEGACY_NAME_HASH_CATALOG_ZIP);
    let Ok(mut archive) = zip::ZipArchive::new(cursor) else {
        log::write_line("embedded name catalog zip parse failed");
        return Vec::new();
    };
    let Ok(mut file) = archive.by_name("name_hash_catalog.txt") else {
        log::write_line("embedded name catalog entry missing");
        return Vec::new();
    };
    let mut bytes = Vec::with_capacity(file.size().min(usize::MAX as u64) as usize);
    if let Err(error) = file.read_to_end(&mut bytes) {
        log::write_line(format!("embedded name catalog read failed: {error}"));
        return Vec::new();
    }
    oppw4_rdb::parse_name_hash_catalog(&bytes)
}

fn scan_known_archives(
    paths: &RuntimePaths,
    catalog: &[oppw4_rdb::NameHashEntry],
    plugin_zip_paths: Vec<PathBuf>,
) -> Vec<patching::VirtualReplacement> {
    let mut replacements = Vec::new();
    let mods = ModRepository::with_zip_paths(paths.mods_root.clone(), plugin_zip_paths);
    for archive in ARCHIVES {
        replacements.extend(scan_archive(paths, &mods, archive, catalog));
    }
    let mut attached = match patching::attach_mod_file_sizes(replacements) {
        Ok(replacements) => replacements,
        Err(error) => {
            log::write_line(format!("virtual table size attach failed: {error}"));
            Vec::new()
        }
    };
    let disabled_hashes = load_disabled_hashes(paths);
    if !disabled_hashes.is_empty() {
        let before = attached.len();
        attached.retain(|replacement| !disabled_hashes.contains(&replacement.hash));
        log::write_line(format!(
            "disabled replacements applied: {}",
            before.saturating_sub(attached.len())
        ));
    }
    for archive in ARCHIVES {
        let bin_path = paths.rdb_root.join(format!("{archive}.rdb.bin"));
        let Ok(metadata) = std::fs::metadata(&bin_path) else {
            continue;
        };
        attached = patching::assign_virtual_bin_offsets(attached, archive, metadata.len());
    }
    attached
}

fn load_disabled_hashes(paths: &RuntimePaths) -> HashSet<u32> {
    let path = paths.config_root.join("disabled_hashes.txt");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return HashSet::new();
    };

    text.lines()
        .filter_map(|line| {
            let raw = line
                .split_once('#')
                .map(|(value, _)| value)
                .unwrap_or(line)
                .trim()
                .trim_end_matches(',');
            let hex = raw.strip_prefix("0x").unwrap_or(raw);
            (!hex.is_empty())
                .then(|| u32::from_str_radix(hex, 16).ok())
                .flatten()
        })
        .collect()
}

fn scan_archive(
    paths: &RuntimePaths,
    mods: &ModRepository,
    archive: &str,
    catalog: &[oppw4_rdb::NameHashEntry],
) -> Vec<patching::VirtualReplacement> {
    let rdb_path = paths.rdb_root.join(format!("{archive}.rdb"));
    let assets = mods.archive_assets(archive);
    if assets.is_empty() {
        log::write_line(format!("{archive}: no runtime mod assets"));
        return Vec::new();
    }
    let names: Vec<_> = assets
        .iter()
        .map(|asset| asset.file_name.as_str())
        .collect();
    let Ok(bytes) = std::fs::read(&rdb_path) else {
        log::write_line(format!(
            "{archive}: rdb not readable: {}",
            rdb_path.display()
        ));
        return Vec::new();
    };
    let Ok(index) = oppw4_rdb::parse_rdb(&bytes) else {
        log::write_line(format!("{archive}: rdb parse failed"));
        return Vec::new();
    };

    let scan = oppw4_rdb::scan_archive_names_with_catalog(archive, &index, &names, catalog);
    let counts = scan.counts();
    log::write_line(format!(
        "{archive}: files={} matched={} hash_missing={} unresolved={}",
        counts.total, counts.matched, counts.hash_missing, counts.unresolved_names
    ));
    patching::build_virtualization_table_from_assets(&scan, &assets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_contains_known_law_replacement_hashes() {
        let catalog = load_embedded_name_catalog();

        assert!(catalog.iter().any(|entry| {
            entry.hash == 0x359b9672
                && entry
                    .name
                    .eq_ignore_ascii_case("800_294_face_law_dressrosa_External_00.g1t")
        }));
    }
}
