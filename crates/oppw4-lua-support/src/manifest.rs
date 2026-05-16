use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaModManifest {
    pub id: String,
    pub name: String,
    pub uses_plugins: Vec<String>,
    pub entry_lua: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LuaMod {
    pub manifest: LuaModManifest,
    pub source: ModSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModSource {
    Directory(PathBuf),
    Zip(PathBuf),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModManifestError {
    InvalidToml,
    MissingModTable,
    MissingId,
    MissingEntry,
    InvalidEntryPath,
}

pub fn discover_mods(root: &Path) -> Vec<LuaMod> {
    let mut mods = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return mods;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            collect_directory_mod(&path, &mut mods);
        } else if file_type.is_file() && is_zip_file(&path) {
            collect_zip_mod(&path, &mut mods);
        }
    }
    mods.sort_by_key(|entry| entry.manifest.id.to_ascii_lowercase());
    mods
}

pub fn parse_mod_manifest(text: &str) -> Result<LuaModManifest, ModManifestError> {
    let value = text
        .parse::<toml::Value>()
        .map_err(|_| ModManifestError::InvalidToml)?;
    let mod_table = value
        .get("mod")
        .and_then(toml::Value::as_table)
        .ok_or(ModManifestError::MissingModTable)?;
    let id = mod_table
        .get("id")
        .and_then(toml::Value::as_str)
        .map(sanitize_id)
        .filter(|id| !id.is_empty())
        .ok_or(ModManifestError::MissingId)?;
    let name = mod_table
        .get("name")
        .and_then(toml::Value::as_str)
        .unwrap_or(&id)
        .to_string();
    let uses_plugins = value
        .get("uses")
        .and_then(|uses| uses.get("plugins"))
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(sanitize_id)
                .filter(|id| !id.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let entry_lua = value
        .get("entry")
        .and_then(|entry| entry.get("lua"))
        .and_then(toml::Value::as_str)
        .ok_or(ModManifestError::MissingEntry)?;
    if !is_safe_relative_file(entry_lua) {
        return Err(ModManifestError::InvalidEntryPath);
    }

    Ok(LuaModManifest {
        id,
        name,
        uses_plugins,
        entry_lua: entry_lua.replace('\\', "/"),
    })
}

impl LuaMod {
    pub fn uses_plugin(&self, plugin_id: &str) -> bool {
        self.manifest
            .uses_plugins
            .iter()
            .any(|id| id.eq_ignore_ascii_case(plugin_id))
    }

    pub fn is_zip(&self) -> bool {
        matches!(self.source, ModSource::Zip(_))
    }

    pub fn source_path(&self) -> &Path {
        match &self.source {
            ModSource::Directory(path) | ModSource::Zip(path) => path,
        }
    }

    pub fn read_entry_script(&self) -> std::io::Result<String> {
        match &self.source {
            ModSource::Directory(root) => {
                fs::read_to_string(root.join(self.manifest.entry_lua.replace('/', "\\")))
            }
            ModSource::Zip(path) => read_zip_text(path, &self.manifest.entry_lua),
        }
    }
}

fn collect_directory_mod(path: &Path, mods: &mut Vec<LuaMod>) {
    let manifest_path = path.join("mod.toml");
    let Ok(text) = fs::read_to_string(manifest_path) else {
        return;
    };
    let Ok(manifest) = parse_mod_manifest(&text) else {
        return;
    };
    mods.push(LuaMod {
        manifest,
        source: ModSource::Directory(path.to_path_buf()),
    });
}

fn collect_zip_mod(path: &Path, mods: &mut Vec<LuaMod>) {
    let Ok(text) = read_zip_text(path, "mod.toml") else {
        return;
    };
    let Ok(manifest) = parse_mod_manifest(&text) else {
        return;
    };
    mods.push(LuaMod {
        manifest,
        source: ModSource::Zip(path.to_path_buf()),
    });
}

fn read_zip_text(path: &Path, entry_name: &str) -> std::io::Result<String> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut entry = archive.by_name(entry_name)?;
    let mut text = String::new();
    entry.read_to_string(&mut text)?;
    Ok(text)
}

fn is_zip_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
}

fn sanitize_id(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn is_safe_relative_file(path: &str) -> bool {
    let path = Path::new(path);
    !path.is_absolute()
        && path.components().all(|component| {
            matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
        && path.file_name().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lua_mod_manifest() {
        let manifest = parse_mod_manifest(
            r#"
                [mod]
                id = "aura_zoro"
                name = "Zoro Aura"

                [uses]
                plugins = ["weapon_aura"]

                [entry]
                lua = "mod.lua"
            "#,
        )
        .expect("manifest");

        assert_eq!(manifest.id, "aura_zoro");
        assert_eq!(manifest.name, "Zoro Aura");
        assert_eq!(manifest.uses_plugins, ["weapon_aura"]);
        assert_eq!(manifest.entry_lua, "mod.lua");
    }

    #[test]
    fn rejects_parent_entry_path() {
        let error = parse_mod_manifest(
            r#"
                [mod]
                id = "bad"

                [entry]
                lua = "../bad.lua"
            "#,
        )
        .expect_err("bad path");

        assert_eq!(error, ModManifestError::InvalidEntryPath);
    }
}
