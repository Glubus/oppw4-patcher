use std::path::{Path, PathBuf};

pub const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";
pub const PLUGIN_LOGS_DIR: &str = "logs";
pub const PLUGIN_MODS_DIR: &str = "mods";
pub const DEFAULT_PLUGIN_VERSION: &str = "0.2.0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDescriptor {
    pub id: String,
    pub version: String,
    pub entry: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginManifestError {
    InvalidToml,
    MissingPluginTable,
    MissingId,
    MissingVersion,
    MissingEntry,
    InvalidEntry(String),
}

impl PluginDescriptor {
    pub fn parse_toml(text: &str) -> Result<Self, PluginManifestError> {
        let value = text
            .parse::<toml::Value>()
            .map_err(|_| PluginManifestError::InvalidToml)?;
        let plugin = value
            .get("plugin")
            .and_then(toml::Value::as_table)
            .ok_or(PluginManifestError::MissingPluginTable)?;
        let id = plugin
            .get("id")
            .and_then(toml::Value::as_str)
            .map(sanitize_plugin_id)
            .filter(|id| id != "unknown_plugin")
            .ok_or(PluginManifestError::MissingId)?;
        let version = plugin
            .get("version")
            .and_then(toml::Value::as_str)
            .filter(|version| !version.trim().is_empty())
            .ok_or(PluginManifestError::MissingVersion)?
            .to_string();
        let entry = plugin
            .get("entry")
            .and_then(toml::Value::as_str)
            .ok_or(PluginManifestError::MissingEntry)?;
        let entry = plugin_entry_file_name(entry)?;

        Ok(Self { id, version, entry })
    }

    pub fn default_for_folder(folder_name: &str, entry: &str) -> Result<Self, PluginManifestError> {
        let id = sanitize_plugin_id(folder_name);
        if id == "unknown_plugin" {
            return Err(PluginManifestError::MissingId);
        }
        Ok(Self {
            id,
            version: DEFAULT_PLUGIN_VERSION.to_string(),
            entry: plugin_entry_file_name(entry)?,
        })
    }

    pub fn to_toml(&self) -> String {
        format!(
            "[plugin]\nid = \"{}\"\nversion = \"{}\"\nentry = \"{}\"\n",
            escape_toml_string(&self.id),
            escape_toml_string(&self.version),
            escape_toml_string(&self.entry)
        )
    }
}

pub fn plugin_toml_path(plugin_dir: &Path) -> PathBuf {
    plugin_dir.join(PLUGIN_MANIFEST_FILE)
}

pub fn plugin_logs_root(plugin_dir: &Path) -> PathBuf {
    plugin_dir.join(PLUGIN_LOGS_DIR)
}

pub fn plugin_mods_root(plugin_dir: &Path) -> PathBuf {
    plugin_dir.join(PLUGIN_MODS_DIR)
}

pub fn sanitize_plugin_id(raw: &str) -> String {
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

pub fn plugin_entry_file_name(raw: &str) -> Result<String, PluginManifestError> {
    let path = Path::new(raw);
    if path.is_absolute()
        || path.components().count() != 1
        || path.file_name().and_then(|value| value.to_str()) != Some(raw)
    {
        return Err(PluginManifestError::InvalidEntry(raw.to_string()));
    }
    Ok(raw.to_string())
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plugin_toml_descriptor() {
        let descriptor = PluginDescriptor::parse_toml(
            r#"
                [plugin]
                id = "fx_director"
                version = "0.2.0"
                entry = "fx_director.dll"
            "#,
        )
        .expect("descriptor");

        assert_eq!(
            descriptor,
            PluginDescriptor {
                id: "fx_director".to_string(),
                version: "0.2.0".to_string(),
                entry: "fx_director.dll".to_string(),
            }
        );
    }

    #[test]
    fn rejects_nested_entry_paths() {
        let error = PluginDescriptor::parse_toml(
            r#"
                [plugin]
                id = "bad"
                version = "0.2.0"
                entry = "bin/bad.dll"
            "#,
        )
        .expect_err("entry path");

        assert_eq!(
            error,
            PluginManifestError::InvalidEntry("bin/bad.dll".to_string())
        );
    }

    #[test]
    fn default_manifest_uses_folder_name_and_entry() {
        let descriptor =
            PluginDescriptor::default_for_folder("fx director", "fx_director.dll").unwrap();

        assert_eq!(descriptor.id, "fx_director");
        assert_eq!(descriptor.version, DEFAULT_PLUGIN_VERSION);
        assert_eq!(
            descriptor.to_toml(),
            "[plugin]\nid = \"fx_director\"\nversion = \"0.2.0\"\nentry = \"fx_director.dll\"\n"
        );
    }
}
