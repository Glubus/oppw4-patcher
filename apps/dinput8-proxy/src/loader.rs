use std::{
    collections::HashSet,
    ffi::c_void,
    io::{Cursor, Read},
    path::PathBuf,
    sync::Once,
};

use crate::{hooks, log, mods::ModRepository, win};

const EMBEDDED_NAME_CATALOG_ZIP: &[u8] = include_bytes!("../../../resources/name_hash_catalog.zip");

static INIT: Once = Once::new();

pub fn initialize_once(module: *mut c_void) {
    INIT.call_once(|| initialize_loader(module));
}

fn initialize_loader(module: *mut c_void) {
    if let Some(base_dir) = win::module_directory(module) {
        log::initialize(base_dir.clone());
        initialize_loader_thread(base_dir);
    }
    log::write_line("loader init placeholder reached");
}

fn initialize_loader_thread(base_dir: PathBuf) {
    log::write_line("loader init started");
    hooks::install_main_module_hooks();
    let paths = LoaderPaths::from_base_dir(base_dir);
    log_loader_paths(&paths);
    let catalog = load_name_catalog(&paths);
    let replacements = scan_known_archives(&paths, &catalog);
    hooks::publish_replacements(replacements);
}

struct LoaderPaths {
    game_root: PathBuf,
    mods_root: PathBuf,
    config_root: PathBuf,
    rdb_root: PathBuf,
}

impl LoaderPaths {
    fn from_base_dir(base_dir: PathBuf) -> Self {
        let mods_root = base_dir.join("mods");
        Self {
            game_root: base_dir.clone(),
            config_root: mods_root.join("_oppw4"),
            mods_root,
            rdb_root: base_dir
                .join("File")
                .join("CMN")
                .join("AssetRelease")
                .join("Retail"),
        }
    }
}

fn log_loader_paths(paths: &LoaderPaths) {
    log::write_line(format!("game root: {}", paths.game_root.display()));
    log::write_line(format!("mods root: {}", paths.mods_root.display()));
    log::write_line(format!("config root: {}", paths.config_root.display()));
    log::write_line(format!("rdb root: {}", paths.rdb_root.display()));
}

fn load_name_catalog(paths: &LoaderPaths) -> Vec<oppw4_rdb::NameHashEntry> {
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
        EMBEDDED_NAME_CATALOG_ZIP.len()
    ));
    catalog
}

fn load_embedded_name_catalog() -> Vec<oppw4_rdb::NameHashEntry> {
    let cursor = Cursor::new(EMBEDDED_NAME_CATALOG_ZIP);
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
    paths: &LoaderPaths,
    catalog: &[oppw4_rdb::NameHashEntry],
) -> Vec<oppw4_rdb::VirtualReplacement> {
    let archives = [
        "CharacterEditor",
        "FieldEditor4",
        "KIDSSystemResource",
        "MaterialEditor",
        "RRPreview",
        "ScreenLayout",
        "SequenceEditor",
        "system",
    ];

    let mut replacements = Vec::new();
    let mods = ModRepository::new(paths.mods_root.clone());
    for archive in archives {
        replacements.extend(scan_archive(paths, &mods, archive, catalog));
    }
    let mut attached = match oppw4_rdb::attach_mod_file_sizes(replacements) {
        Ok(replacements) => replacements,
        Err(error) => {
            log::write_line(format!("virtual table size attach failed: {error}"));
            Vec::new()
        }
    };
    log::write_line("virtual prefixes: using RDB index prefixes".to_string());
    let metadata_aliases = attached
        .iter()
        .filter(|replacement| looks_like_external_metadata_alias(replacement))
        .count();
    for replacement in attached
        .iter()
        .filter(|replacement| looks_like_external_metadata_alias(replacement))
    {
        log::write_line(format!(
            "metadata alias virtual candidate: archive={} hash=0x{:08x} file={} original=0x{:x} mod=0x{:x}",
            replacement.archive_name,
            replacement.hash,
            replacement.file_name,
            replacement.original_bin_size.unwrap_or_default(),
            replacement.mod_size.unwrap_or_default()
        ));
    }
    log::write_line(format!(
        "metadata alias filter: skipped=0 patched_like_original={metadata_aliases}"
    ));
    let disabled_hashes = load_disabled_hashes(paths);
    if !disabled_hashes.is_empty() {
        let before = attached.len();
        attached.retain(|replacement| {
            let enabled = !disabled_hashes.contains(&replacement.hash);
            if !enabled {
                log::write_line(format!(
                    "disabled replacement: archive={} hash=0x{:08x} file={}",
                    replacement.archive_name, replacement.hash, replacement.file_name
                ));
            }
            enabled
        });
        log::write_line(format!(
            "disabled replacements applied: {}",
            before.saturating_sub(attached.len())
        ));
    }
    for archive in archives {
        let bin_path = paths.rdb_root.join(format!("{archive}.rdb.bin"));
        let Ok(metadata) = std::fs::metadata(&bin_path) else {
            continue;
        };
        attached = oppw4_rdb::assign_virtual_bin_offsets(attached, archive, metadata.len());
    }
    log::write_line(format!("virtual replacements ready: {}", attached.len()));
    attached
}

fn looks_like_external_metadata_alias(replacement: &oppw4_rdb::VirtualReplacement) -> bool {
    let file_name = replacement.file_name.to_ascii_lowercase();
    file_name.contains("_external_")
        && replacement
            .original_bin_size
            .zip(replacement.mod_size)
            .is_some_and(|(original, replacement)| original < 0x1000 && replacement > 0x10000)
}

fn load_disabled_hashes(paths: &LoaderPaths) -> HashSet<u32> {
    let path = paths.config_root.join("disabled_hashes.txt");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return HashSet::new();
    };

    let mut hashes = HashSet::new();
    for line in text.lines() {
        let raw = line
            .split_once('#')
            .map(|(value, _)| value)
            .unwrap_or(line)
            .trim()
            .trim_end_matches(',');
        if raw.is_empty() {
            continue;
        }
        let hex = raw.strip_prefix("0x").unwrap_or(raw);
        match u32::from_str_radix(hex, 16) {
            Ok(hash) => {
                hashes.insert(hash);
            }
            Err(error) => log::write_line(format!("disabled hash ignored: {raw} ({error})")),
        }
    }

    log::write_line(format!(
        "disabled hash entries: {} from {}",
        hashes.len(),
        path.display()
    ));
    hashes
}

fn scan_archive(
    paths: &LoaderPaths,
    mods: &ModRepository,
    archive: &str,
    catalog: &[oppw4_rdb::NameHashEntry],
) -> Vec<oppw4_rdb::VirtualReplacement> {
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
    oppw4_rdb::build_virtualization_table_from_assets(&scan, &assets)
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
        assert!(catalog.iter().any(|entry| {
            entry.hash == 0x3bff0f13
                && entry
                    .name
                    .eq_ignore_ascii_case("801_294_chara_law_dressrosa_External_00.g1t")
        }));
    }

    #[test]
    fn loader_paths_use_game_side_mods_not_patcher_folder() {
        let game_root = PathBuf::from(r"D:\Game\OPPW4");
        let paths = LoaderPaths::from_base_dir(game_root.clone());

        assert_eq!(paths.game_root, game_root);
        assert_eq!(paths.mods_root, PathBuf::from(r"D:\Game\OPPW4\mods"));
        assert_eq!(
            paths.config_root,
            PathBuf::from(r"D:\Game\OPPW4\mods\_oppw4")
        );
    }
}
