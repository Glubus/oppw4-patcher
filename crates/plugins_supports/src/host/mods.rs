use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn list_zip_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_zip_paths(root, &mut paths);
    paths.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
    paths
}

fn collect_zip_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            collect_zip_paths(&path, paths);
        } else if file_type.is_file() && is_zip_file(&path) {
            paths.push(path);
        }
    }
}

fn is_zip_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn lists_plugin_mod_zips_recursively_and_sorted() {
        let root = temp_root("plugin-zips");
        fs::create_dir_all(root.join("nested")).expect("nested dir");
        fs::write(root.join("zoro.zip"), []).expect("zip");
        fs::write(root.join("nested").join("law.ZIP"), []).expect("nested zip");
        fs::write(root.join("notes.txt"), []).expect("txt");

        let zips = list_zip_paths(&root);

        assert_eq!(
            zips,
            vec![root.join("nested").join("law.ZIP"), root.join("zoro.zip")]
        );
        let _ = fs::remove_dir_all(root);
    }

    fn temp_root(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("oppw4-{label}-{nanos}"))
    }
}
