use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::host::log;

#[derive(Debug, PartialEq)]
pub(crate) struct PluginManifest {
    pub(crate) id: String,
    pub(crate) version: String,
    pub(crate) root: PathBuf,
    pub(crate) mods_root: PathBuf,
    pub(crate) entry_path: PathBuf,
    pub(crate) log_root: PathBuf,
}

impl PluginManifest {
    pub(crate) fn read_from_dir(plugin_dir: &Path) -> Option<Self> {
        let manifest_path = plugin_dir.join("plugin.toml");
        if !manifest_path.is_file() {
            if let Err(error) = create_default_manifest(plugin_dir, &manifest_path) {
                log::write_line(format!(
                    "plugin host: manifest missing path={} error={error}",
                    manifest_path.display()
                ));
                return None;
            }
        }

        let text = match fs::read_to_string(&manifest_path) {
            Ok(text) => text,
            Err(error) => {
                log::write_line(format!(
                    "plugin host: manifest missing path={} error={error}",
                    manifest_path.display()
                ));
                return None;
            }
        };

        match Self::parse(plugin_dir, &text) {
            Ok(manifest) => Some(manifest),
            Err(error) => {
                log::write_line(format!(
                    "plugin host: manifest invalid path={} error={error}",
                    manifest_path.display()
                ));
                None
            }
        }
    }

    fn parse(plugin_dir: &Path, text: &str) -> Result<Self, String> {
        let value = text
            .parse::<toml::Value>()
            .map_err(|error| error.to_string())?;
        let plugin = value
            .get("plugin")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| "missing [plugin] table".to_string())?;
        let id = plugin
            .get("id")
            .and_then(toml::Value::as_str)
            .map(sanitize_plugin_id)
            .filter(|id| id != "unknown_plugin")
            .ok_or_else(|| "missing plugin.id".to_string())?;
        let version = plugin
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| "missing plugin.version".to_string())?
            .to_string();
        let entry = plugin
            .get("entry")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| "missing plugin.entry".to_string())?;

        Ok(Self {
            id,
            version,
            root: plugin_dir.to_path_buf(),
            mods_root: plugin_dir.join("mods"),
            entry_path: entry_file_path(plugin_dir, entry)?,
            log_root: plugin_dir.join("logs"),
        })
    }
}

fn create_default_manifest(plugin_dir: &Path, manifest_path: &Path) -> Result<(), String> {
    let entry = infer_entry_file(plugin_dir)?;
    let id = plugin_dir
        .file_name()
        .and_then(|name| name.to_str())
        .map(sanitize_plugin_id)
        .filter(|id| id != "unknown_plugin")
        .ok_or_else(|| "cannot infer plugin id from folder name".to_string())?;
    let text = format!("[plugin]\nid = \"{id}\"\nversion = \"0.2.0\"\nentry = \"{entry}\"\n");
    fs::write(manifest_path, text).map_err(|error| error.to_string())
}

fn infer_entry_file(plugin_dir: &Path) -> Result<String, String> {
    let folder_name = plugin_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "cannot infer plugin folder name".to_string())?;
    let preferred = plugin_dir.join(format!("{folder_name}.dll"));
    if preferred.is_file() {
        return Ok(preferred
            .file_name()
            .and_then(|name| name.to_str())
            .expect("preferred file name")
            .to_string());
    }

    let mut dlls = fs::read_dir(plugin_dir)
        .map_err(|error| error.to_string())?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let is_dll = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"));
            if is_dll {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    dlls.sort();

    match dlls.as_slice() {
        [entry] => Ok(entry.clone()),
        [] => Err("cannot create plugin.toml without a dll in the plugin folder".to_string()),
        _ => Err("cannot create plugin.toml because multiple dll files exist".to_string()),
    }
}

pub(crate) fn sanitize_plugin_id(raw: &str) -> String {
    let sanitized = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if sanitized.is_empty() {
        "unknown_plugin".to_string()
    } else {
        sanitized
    }
}

fn entry_file_path(root: &Path, child: &str) -> Result<PathBuf, String> {
    let path = Path::new(child);
    if path.is_absolute()
        || path.components().count() != 1
        || path.file_name().and_then(|value| value.to_str()) != Some(child)
    {
        return Err(format!("entry must be a file name only: {child}"));
    }
    Ok(root.join(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_points_entry_and_logs_inside_plugin_folder() {
        let root = PathBuf::from(r"D:\Game\OPPW4\plugins\skin_patcher");
        let manifest = PluginManifest::parse(
            &root,
            r#"
                [plugin]
                id = "skin_patcher"
                version = "0.1.0"
                entry = "skin_patcher.dll"
            "#,
        )
        .expect("manifest");

        assert_eq!(manifest.id, "skin_patcher");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.root, root);
        assert_eq!(manifest.mods_root, root.join("mods"));
        assert_eq!(manifest.entry_path, root.join("skin_patcher.dll"));
        assert_eq!(manifest.log_root, root.join("logs"));
    }

    #[test]
    fn manifest_rejects_entry_with_path_segments() {
        let error = PluginManifest::parse(
            Path::new(r"D:\Game\OPPW4\plugins\bad"),
            r#"
                [plugin]
                id = "bad"
                version = "0.1.0"
                entry = "bin/bad.dll"
            "#,
        )
        .expect_err("entry should be rejected");

        assert!(error.contains("file name only"));
    }

    #[test]
    fn manifest_requires_version() {
        let error = PluginManifest::parse(
            Path::new(r"D:\Game\OPPW4\plugins\bad"),
            r#"
                [plugin]
                id = "bad"
                entry = "bad.dll"
            "#,
        )
        .expect_err("version should be required");

        assert!(error.contains("plugin.version"));
    }

    #[test]
    fn manifest_file_is_named_plugin_toml() {
        let root = temp_root("plugin-toml");
        fs::create_dir_all(&root).expect("temp plugin dir");
        fs::write(
            root.join("plugin.toml"),
            r#"
                [plugin]
                id = "skin_patcher"
                version = "0.1.0"
                entry = "skin_patcher.dll"
            "#,
        )
        .expect("plugin manifest");

        let manifest = PluginManifest::read_from_dir(&root).expect("manifest");

        assert_eq!(manifest.id, "skin_patcher");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn creates_plugin_toml_from_matching_dll_when_missing() {
        let root = temp_root("auto-plugin-toml").join("fx_director");
        fs::create_dir_all(&root).expect("temp plugin dir");
        fs::write(root.join("fx_director.dll"), []).expect("plugin dll");

        let manifest = PluginManifest::read_from_dir(&root).expect("manifest");

        assert_eq!(manifest.id, "fx_director");
        assert_eq!(manifest.version, "0.2.0");
        assert_eq!(manifest.entry_path, root.join("fx_director.dll"));
        assert!(root.join("plugin.toml").is_file());
        let _ = fs::remove_dir_all(root.parent().expect("temp root"));
    }

    #[test]
    fn does_not_create_plugin_toml_when_entry_is_ambiguous() {
        let root = temp_root("ambiguous-plugin-toml");
        fs::create_dir_all(&root).expect("temp plugin dir");
        fs::write(root.join("a.dll"), []).expect("first dll");
        fs::write(root.join("b.dll"), []).expect("second dll");

        assert!(PluginManifest::read_from_dir(&root).is_none());
        assert!(!root.join("plugin.toml").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_mod_toml_is_not_a_plugin_manifest_without_a_dll() {
        let root = temp_root("mod-toml");
        fs::create_dir_all(&root).expect("temp plugin dir");
        fs::write(
            root.join("mod.toml"),
            r#"
                [plugin]
                id = "skin_patcher"
                version = "0.1.0"
                entry = "skin_patcher.dll"
            "#,
        )
        .expect("legacy manifest");

        assert!(PluginManifest::read_from_dir(&root).is_none());
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
