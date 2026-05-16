use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use mlua::{Lua, Table};

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptMod {
    pub zip_path: PathBuf,
    pub metadata: ScriptModMetadata,
    pub script: String,
    pub actions: Vec<ScriptAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptModMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: String,
    pub entry: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptAction {
    WeaponAura {
        target: AuraTarget,
        aura_id: u32,
    },
    WeaponAuraDuration {
        aura_id: u32,
        step: f32,
        reset_at: f32,
        reset_to: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuraTarget {
    LocalPlayer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptModError {
    Io(String),
    Zip(String),
    MissingEntry(String),
    InvalidMetadata(String),
    InvalidScript(String),
}

pub fn load_script_mod_zip(zip_path: impl AsRef<Path>) -> Result<ScriptMod, ScriptModError> {
    let zip_path = zip_path.as_ref();
    let file = File::open(zip_path).map_err(io_error)?;
    let mut archive = zip::ZipArchive::new(file).map_err(zip_error)?;
    let metadata_text = read_zip_text(&mut archive, "mod.toml")?;
    let metadata = parse_metadata(&metadata_text)?;
    let script = read_zip_text(&mut archive, &metadata.entry)?;
    let actions = run_script_actions(&script)?;

    Ok(ScriptMod {
        zip_path: zip_path.to_path_buf(),
        metadata,
        script,
        actions,
    })
}

pub fn discover_script_mods(root: impl AsRef<Path>) -> Vec<Result<ScriptMod, ScriptModError>> {
    let mut zip_paths = Vec::new();
    collect_zip_paths(root.as_ref(), &mut zip_paths);
    zip_paths.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
    zip_paths
        .into_iter()
        .map(load_script_mod_zip)
        .filter(|result| match result {
            Ok(script_mod) => script_mod.metadata.kind == "script",
            Err(ScriptModError::MissingEntry(entry)) if entry == "mod.toml" => false,
            Err(_) => true,
        })
        .collect()
}

pub fn parse_metadata(text: &str) -> Result<ScriptModMetadata, ScriptModError> {
    let id = required_string_field(text, "id")?;
    let name = required_string_field(text, "name")?;
    let version = required_string_field(text, "version")?;
    let kind = required_string_field(text, "kind")?;
    let entry = required_string_field(text, "entry")?;
    if kind != "script" {
        return Err(ScriptModError::InvalidMetadata(format!(
            "unsupported mod kind: {kind}"
        )));
    }
    if entry.starts_with('/') || entry.contains("..") || entry.contains('\\') {
        return Err(ScriptModError::InvalidMetadata(format!(
            "unsafe script entry path: {entry}"
        )));
    }

    Ok(ScriptModMetadata {
        id,
        name,
        version,
        kind,
        entry,
    })
}

pub fn run_script_actions(text: &str) -> Result<Vec<ScriptAction>, ScriptModError> {
    let lua = Lua::new();
    let actions = Arc::new(Mutex::new(Vec::new()));

    install_mod_api(&lua)?;
    install_fx_api(&lua, Arc::clone(&actions))?;
    lua.load(text)
        .set_name("script-mod")
        .exec()
        .map_err(lua_error)?;

    actions
        .lock()
        .map(|actions| actions.clone())
        .map_err(|_| ScriptModError::InvalidScript("script action collector poisoned".into()))
}

fn install_mod_api(lua: &Lua) -> Result<(), ScriptModError> {
    let mod_table = lua.create_table().map_err(lua_error)?;
    let on_enable = lua
        .create_function(|_, (_self, callback): (Table, mlua::Function)| {
            callback.call::<()>(())
        })
        .map_err(lua_error)?;
    let on_disable = lua
        .create_function(|_, _callback: mlua::Function| Ok(()))
        .map_err(lua_error)?;
    mod_table.set("on_enable", on_enable).map_err(lua_error)?;
    mod_table.set("on_disable", on_disable).map_err(lua_error)?;
    lua.globals().set("mod", mod_table).map_err(lua_error)
}

fn install_fx_api(lua: &Lua, actions: Arc<Mutex<Vec<ScriptAction>>>) -> Result<(), ScriptModError> {
    let fx = lua.create_table().map_err(lua_error)?;

    let weapon_aura_actions = Arc::clone(&actions);
    let weapon_aura = lua
        .create_function(move |_, table: Table| {
            let target = match table.get::<String>("target")?.as_str() {
                "local_player" => AuraTarget::LocalPlayer,
                other => {
                    return Err(mlua::Error::external(format!(
                        "unsupported aura target: {other}"
                    )))
                }
            };
            let aura_id = table.get::<u32>("aura_id")?;
            weapon_aura_actions
                .lock()
                .map_err(|_| mlua::Error::external("script action collector poisoned"))?
                .push(ScriptAction::WeaponAura { target, aura_id });
            Ok(())
        })
        .map_err(lua_error)?;

    let duration_actions = Arc::clone(&actions);
    let weapon_aura_duration = lua
        .create_function(move |_, table: Table| {
            let aura_id = table.get::<u32>("aura_id")?;
            let step = table.get::<f32>("step")?;
            let reset_at = table.get::<f32>("reset_at")?;
            let reset_to = table.get::<f32>("reset_to")?;
            duration_actions
                .lock()
                .map_err(|_| mlua::Error::external("script action collector poisoned"))?
                .push(ScriptAction::WeaponAuraDuration {
                    aura_id,
                    step,
                    reset_at,
                    reset_to,
                });
            Ok(())
        })
        .map_err(lua_error)?;

    fx.set("weapon_aura", weapon_aura).map_err(lua_error)?;
    fx.set("weapon_aura_duration", weapon_aura_duration)
        .map_err(lua_error)?;
    lua.globals().set("fx", fx).map_err(lua_error)
}

fn read_zip_text(
    archive: &mut zip::ZipArchive<File>,
    entry_name: &str,
) -> Result<String, ScriptModError> {
    let mut entry = archive
        .by_name(entry_name)
        .map_err(|_| ScriptModError::MissingEntry(entry_name.to_string()))?;
    let mut text = String::new();
    entry.read_to_string(&mut text).map_err(io_error)?;
    Ok(text)
}

fn required_string_field(text: &str, field: &str) -> Result<String, ScriptModError> {
    parse_toml_string_field(text, field).ok_or_else(|| {
        ScriptModError::InvalidMetadata(format!("missing string field: {field}"))
    })
}

fn parse_toml_string_field(text: &str, field: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.split_once('#').map_or(line, |(before, _)| before).trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != field {
            continue;
        }
        return parse_quoted_string(value.trim());
    }
    None
}

fn parse_quoted_string(value: &str) -> Option<String> {
    let mut chars = value.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let mut output = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            output.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return Some(output);
        } else {
            output.push(ch);
        }
    }
    None
}

fn io_error(error: std::io::Error) -> ScriptModError {
    ScriptModError::Io(error.to_string())
}

fn zip_error(error: zip::result::ZipError) -> ScriptModError {
    ScriptModError::Zip(error.to_string())
}

fn lua_error(error: mlua::Error) -> ScriptModError {
    ScriptModError::InvalidScript(error.to_string())
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
            if entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("_oppw4")
            {
                continue;
            }
            collect_zip_paths(&path, paths);
        } else if file_type.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
        {
            paths.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use super::*;

    #[test]
    fn parses_script_mod_metadata() {
        let metadata = parse_metadata(
            r#"
id = "aura_zoro"
name = "Zoro Aura Test"
version = "0.1.0"
kind = "script"
entry = "scripts/main.lua"
"#,
        )
        .unwrap();

        assert_eq!(metadata.id, "aura_zoro");
        assert_eq!(metadata.entry, "scripts/main.lua");
    }

    #[test]
    fn collects_weapon_aura_actions_from_mlua() {
        let actions = run_script_actions(
            r#"
mod:on_enable(function()
  fx.weapon_aura({
    target = "local_player",
    aura_id = 2830
  })
  fx.weapon_aura_duration({
    aura_id = 2830,
    step = 0.6,
    reset_at = 1.9,
    reset_to = 0.1
  })
end)
"#,
        )
        .unwrap();

        assert_eq!(
            actions,
            vec![
                ScriptAction::WeaponAura {
                    target: AuraTarget::LocalPlayer,
                    aura_id: 2830
                },
                ScriptAction::WeaponAuraDuration {
                    aura_id: 2830,
                    step: 0.6,
                    reset_at: 1.9,
                    reset_to: 0.1
                }
            ]
        );
    }

    #[test]
    fn loads_script_mod_zip() {
        let root = temp_root("script-mod-zip");
        let zip_path = root.join("aura_zoro.zip");
        write_zip(
            &zip_path,
            &[
                (
                    "mod.toml",
                    br#"id = "aura_zoro"
name = "Zoro Aura Test"
version = "0.1.0"
kind = "script"
entry = "scripts/main.lua"
"#,
                ),
                (
                    "scripts/main.lua",
                    br#"mod:on_enable(function()
  fx.weapon_aura({ target = "local_player", aura_id = 2830 })
  fx.weapon_aura_duration({ aura_id = 2830, step = 0.6, reset_at = 1.9, reset_to = 0.1 })
end)
"#,
                ),
            ],
        );

        let script_mod = load_script_mod_zip(&zip_path).unwrap();

        assert_eq!(script_mod.metadata.id, "aura_zoro");
        assert_eq!(script_mod.actions.len(), 2);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn discovers_script_mods_and_ignores_config_zips() {
        let root = temp_root("discover-script-mods");
        let mod_zip = root.join("aura_zoro.zip");
        let config_root = root.join("_oppw4");
        fs::create_dir_all(&config_root).unwrap();
        write_zip(
            &mod_zip,
            &[
                (
                    "mod.toml",
                    br#"id = "aura_zoro"
name = "Zoro Aura Test"
version = "0.1.0"
kind = "script"
entry = "scripts/main.lua"
"#,
                ),
                (
                    "scripts/main.lua",
                    br#"mod:on_enable(function()
  fx.weapon_aura({ target = "local_player", aura_id = 2830 })
end)
"#,
                ),
            ],
        );
        write_zip(&config_root.join("ignored.zip"), &[("mod.toml", b"broken")]);

        let mods = discover_script_mods(&root);

        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].as_ref().unwrap().metadata.id, "aura_zoro");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_unsafe_entry_path() {
        let error = parse_metadata(
            r#"
id = "bad"
name = "Bad"
version = "0.1.0"
kind = "script"
entry = "../main.lua"
"#,
        )
        .unwrap_err();

        assert!(matches!(error, ScriptModError::InvalidMetadata(_)));
    }

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "oppw4-script-mods-{label}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = File::create(path).unwrap();
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
