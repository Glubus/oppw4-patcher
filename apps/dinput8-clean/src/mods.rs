use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use oppw4_rdb::{ModAsset, ReplacementSource};

pub struct ModRepository {
    root: PathBuf,
}

impl ModRepository {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn archive_assets(&self, archive_name: &str) -> Vec<ModAsset> {
        let mut assets = HashMap::new();
        for zip_path in self.zip_paths() {
            self.collect_zip_archive_assets(archive_name, &zip_path, &mut assets);
        }
        self.collect_loose_archive_assets(archive_name, &mut assets);

        let mut assets: Vec<_> = assets.into_values().collect();
        assets.sort_by_key(|asset| asset.file_name.to_ascii_lowercase());
        assets
    }

    fn collect_loose_archive_assets(
        &self,
        archive_name: &str,
        assets: &mut HashMap<String, ModAsset>,
    ) {
        for archive_root in self.loose_archive_roots(archive_name) {
            collect_loose_archive_root_assets(&archive_root, assets);
        }
    }

    fn loose_archive_roots(&self, archive_name: &str) -> Vec<PathBuf> {
        named_loose_archive_roots(&self.root, archive_name)
    }

    fn collect_zip_archive_assets(
        &self,
        archive_name: &str,
        zip_path: &Path,
        assets: &mut HashMap<String, ModAsset>,
    ) {
        let Ok(file) = fs::File::open(zip_path) else {
            return;
        };
        let Ok(mut archive) = zip::ZipArchive::new(file) else {
            return;
        };

        for index in 0..archive.len() {
            let Ok(entry) = archive.by_index(index) else {
                continue;
            };
            if !entry.is_file() {
                continue;
            };
            let entry_name = entry.name().replace('\\', "/");
            let Some(file_name) = archive_entry_file_name(archive_name, &entry_name) else {
                continue;
            };
            insert_asset(
                assets,
                ModAsset {
                    file_name,
                    source: ReplacementSource::ZipEntry {
                        zip_path: zip_path.to_path_buf(),
                        entry_name,
                    },
                },
            );
        }
    }

    fn zip_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        collect_zip_paths(&self.root, &mut paths);
        paths.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
        paths
    }
}

fn collect_loose_archive_root_assets(archive_root: &Path, assets: &mut HashMap<String, ModAsset>) {
    let Some(entries) = read_directory(archive_root) else {
        return;
    };

    for entry in entries.flatten() {
        collect_loose_file_asset(entry, assets);
    }
}

fn collect_loose_file_asset(entry: fs::DirEntry, assets: &mut HashMap<String, ModAsset>) {
    let Some(file_type) = entry.file_type().ok() else {
        return;
    };
    if !file_type.is_file() {
        return;
    }
    let file_name = entry.file_name().to_string_lossy().into_owned();
    insert_asset(
        assets,
        ModAsset {
            file_name,
            source: ReplacementSource::File(entry.path()),
        },
    );
}

fn named_loose_archive_roots(root: &Path, archive_name: &str) -> Vec<PathBuf> {
    let Some(entries) = read_directory(root) else {
        return Vec::new();
    };

    let mut roots: Vec<_> = entries
        .flatten()
        .filter_map(|entry| named_loose_archive_root(entry, archive_name))
        .collect();
    roots.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
    roots
}

fn named_loose_archive_root(entry: fs::DirEntry, archive_name: &str) -> Option<PathBuf> {
    if !is_directory_entry(&entry) || is_config_directory(&entry) {
        return None;
    }
    if entry_name_eq(&entry, archive_name) {
        return None;
    }

    let archive_root = entry.path().join(archive_name);
    archive_root.is_dir().then_some(archive_root)
}

fn insert_asset(assets: &mut HashMap<String, ModAsset>, asset: ModAsset) {
    assets.insert(asset.file_name.to_ascii_lowercase(), asset);
}

fn collect_zip_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    let Some(entries) = read_directory(root) else {
        return;
    };
    for entry in entries.flatten() {
        collect_zip_path_entry(entry, paths);
    }
}

fn read_directory(root: &Path) -> Option<fs::ReadDir> {
    fs::read_dir(root).ok()
}

fn collect_zip_path_entry(entry: fs::DirEntry, paths: &mut Vec<PathBuf>) {
    let Some(file_type) = entry.file_type().ok() else {
        return;
    };
    let path = entry.path();

    if file_type.is_dir() {
        collect_zip_paths_from_directory(&entry, &path, paths);
    } else if file_type.is_file() && is_zip_file(&path) {
        paths.push(path);
    }
}

fn is_directory_entry(entry: &fs::DirEntry) -> bool {
    entry.file_type().is_ok_and(|file_type| file_type.is_dir())
}

fn collect_zip_paths_from_directory(entry: &fs::DirEntry, path: &Path, paths: &mut Vec<PathBuf>) {
    if is_config_directory(entry) {
        return;
    }
    collect_zip_paths(path, paths);
}

fn is_config_directory(entry: &fs::DirEntry) -> bool {
    entry_name_eq(entry, "_oppw4")
}

fn entry_name_eq(entry: &fs::DirEntry, expected: &str) -> bool {
    entry
        .file_name()
        .to_string_lossy()
        .eq_ignore_ascii_case(expected)
}

fn is_zip_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
}

fn archive_entry_file_name(archive_name: &str, entry_name: &str) -> Option<String> {
    let parts: Vec<_> = entry_name
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect();
    let archive_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case(archive_name))?;
    let file_name = parts.get(archive_index + 1..)?.last()?;
    (!file_name.is_empty()).then(|| (*file_name).to_string())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn zip_entry_archive_match_accepts_direct_or_nested_archive_paths() {
        assert_eq!(
            archive_entry_file_name("ScreenLayout", "ScreenLayout/800_face.g1t"),
            Some("800_face.g1t".to_string())
        );
        assert_eq!(
            archive_entry_file_name("ScreenLayout", "SomeMod/ScreenLayout/800_face.g1t"),
            Some("800_face.g1t".to_string())
        );
        assert_eq!(
            archive_entry_file_name("ScreenLayout", "SomeMod/MaterialEditor/800_face.g1t"),
            None
        );
    }

    #[test]
    fn archive_assets_loads_zip_entries_for_requested_archive() {
        let root = temp_root("zip-assets");
        let zip_path = root.join("law.zip");
        write_zip(
            &zip_path,
            &[
                (
                    "Law/ScreenLayout/800_294_face_law_dressrosa_External_00.g1t",
                    b"face",
                ),
                ("Law/MaterialEditor/not_this_archive.g1t", b"skip"),
            ],
        );

        let assets = ModRepository::new(root.clone()).archive_assets("ScreenLayout");

        assert_eq!(assets.len(), 1);
        assert_eq!(
            assets[0].file_name,
            "800_294_face_law_dressrosa_External_00.g1t"
        );
        assert_eq!(
            assets[0].source,
            ReplacementSource::ZipEntry {
                zip_path,
                entry_name: "Law/ScreenLayout/800_294_face_law_dressrosa_External_00.g1t"
                    .to_string(),
            }
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn named_loose_archive_file_overrides_same_named_zip_entry() {
        let root = temp_root("loose-priority");
        write_zip(
            &root.join("base.zip"),
            &[("CharacterEditor/modded.g1m", b"zip")],
        );
        let archive_root = root.join("legacy").join("CharacterEditor");
        fs::create_dir_all(&archive_root).unwrap();
        fs::write(archive_root.join("modded.g1m"), b"loose").unwrap();

        let assets = ModRepository::new(root.clone()).archive_assets("CharacterEditor");

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].file_name, "modded.g1m");
        assert!(matches!(assets[0].source, ReplacementSource::File(_)));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn named_loose_mod_folder_loads_requested_archive_assets() {
        let root = temp_root("named-loose");
        let archive_root = root.join("law-pack").join("ScreenLayout");
        fs::create_dir_all(&archive_root).unwrap();
        fs::write(
            archive_root.join("800_294_face_law_dressrosa_External_00.g1t"),
            b"face",
        )
        .unwrap();

        let assets = ModRepository::new(root.clone()).archive_assets("ScreenLayout");

        assert_eq!(assets.len(), 1);
        assert_eq!(
            assets[0].file_name,
            "800_294_face_law_dressrosa_External_00.g1t"
        );
        assert_eq!(
            assets[0].source,
            ReplacementSource::File(
                root.join("law-pack")
                    .join("ScreenLayout")
                    .join("800_294_face_law_dressrosa_External_00.g1t")
            )
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn direct_loose_archive_folder_is_not_loaded() {
        let root = temp_root("direct-loose-ignored");
        let direct_archive_root = root.join("ScreenLayout");
        fs::create_dir_all(&direct_archive_root).unwrap();
        fs::write(direct_archive_root.join("shared.g1t"), b"direct").unwrap();

        let assets = ModRepository::new(root.clone()).archive_assets("ScreenLayout");

        assert!(assets.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn oppw4_config_folder_is_not_scanned_for_mod_assets() {
        let root = temp_root("skip-config");
        let config_root = root.join("_oppw4");
        let config_archive_root = config_root.join("ScreenLayout");
        fs::create_dir_all(&config_archive_root).unwrap();
        write_zip(
            &config_root.join("debug.zip"),
            &[("ScreenLayout/should_not_load.g1t", b"nope")],
        );
        fs::write(
            config_archive_root.join("also_should_not_load.g1t"),
            b"nope",
        )
        .unwrap();

        let assets = ModRepository::new(root.clone()).archive_assets("ScreenLayout");

        assert!(assets.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zip_paths_recurses_sorts_and_ignores_non_zip_files() {
        let root = temp_root("zip-paths");
        let nested_root = root.join("nested");
        fs::create_dir_all(&nested_root).unwrap();
        fs::write(root.join("a.zip"), b"zip").unwrap();
        fs::write(root.join("not-a-zip.txt"), b"skip").unwrap();
        fs::write(nested_root.join("b.zip"), b"zip").unwrap();

        let paths = ModRepository::new(root.clone()).zip_paths();

        assert_eq!(paths, vec![root.join("a.zip"), nested_root.join("b.zip")]);

        let _ = fs::remove_dir_all(root);
    }

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "oppw4-mod-repository-{label}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, bytes) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
}
