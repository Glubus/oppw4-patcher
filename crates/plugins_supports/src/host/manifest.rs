use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::log;

#[derive(Debug, PartialEq)]
pub(crate) struct PluginManifest {
    pub(crate) id: String,
    pub(crate) version: String,
    pub(crate) entry_path: PathBuf,
    pub(crate) log_root: PathBuf,
}

impl PluginManifest {
    pub(crate) fn read_from_dir(plugin_dir: &Path) -> Option<Self> {
        let manifest_path = plugin_dir.join("mod.toml");
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
            entry_path: entry_file_path(plugin_dir, entry)?,
            log_root: plugin_dir.join("logs"),
        })
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
}
