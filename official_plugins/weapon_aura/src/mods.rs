use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use mlua::{Lua, Table, Value};
use oppw4_character::Character;
use oppw4_lua_support::{LuaMod, ModSource};
use oppw4_plugin_api::{Oppw4PluginApi, PluginModInfo, OPPW4_PLUGIN_MOD_FLAG_ZIP};

use crate::{
    config::{AuraConfig, InstallMode, StatusGate, TargetMode, TriggerMode},
    log,
};

pub(crate) fn load_config(api: &Oppw4PluginApi) -> AuraConfig {
    let config = AuraConfig::load(api).unwrap_or_default();
    let mods = api.plugin_mods();
    if mods.is_empty() {
        log::write_line("weapon_aura lua mods: none");
        return config;
    }

    let shared = Arc::new(Mutex::new(config));
    log::write_line(format!("weapon_aura lua mods: {}", mods.len()));
    for mod_info in mods {
        run_mod(mod_info, Arc::clone(&shared));
    }
    let config = *shared.lock().expect("aura config lock");
    config
}

fn run_mod(mod_info: PluginModInfo, config: Arc<Mutex<AuraConfig>>) {
    let mod_entry = lua_mod_from_info(&mod_info);
    let id = mod_entry.manifest.id.clone();
    let result = oppw4_lua_support::run_lua_mod(&mod_entry, |lua| {
        let table = aura_module(lua, Arc::clone(&config))?;
        let character = character_module(lua, Arc::clone(&config))?;
        oppw4_lua_support::register_module(lua, "weapon_aura", table.clone())?;
        oppw4_lua_support::register_module(lua, "aura", table)?;
        oppw4_lua_support::register_module(lua, "character", character)
    });
    match result {
        Ok(()) => log::write_line(format!("weapon_aura lua mod applied id={id}")),
        Err(error) => log::write_line(format!(
            "weapon_aura lua mod failed id={id} error={error:?}"
        )),
    }
}

fn lua_mod_from_info(info: &PluginModInfo) -> LuaMod {
    LuaMod {
        manifest: oppw4_lua_support::LuaModManifest {
            id: info.id.clone(),
            name: info.name.clone(),
            uses_plugins: vec!["weapon_aura".to_string()],
            entry_lua: info.entry_lua.clone(),
        },
        source: if info.flags & OPPW4_PLUGIN_MOD_FLAG_ZIP != 0 {
            ModSource::Zip(PathBuf::from(&info.source_path))
        } else {
            ModSource::Directory(PathBuf::from(&info.source_path))
        },
    }
}

fn aura_module(lua: &Lua, config: Arc<Mutex<AuraConfig>>) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;

    let enabled_config = Arc::clone(&config);
    table.set(
        "enable",
        lua.create_function(move |_, enabled: Option<bool>| {
            enabled_config.lock().expect("aura config lock").enabled = enabled.unwrap_or(true);
            Ok(())
        })?,
    )?;

    let disabled_config = Arc::clone(&config);
    table.set(
        "disable",
        lua.create_function(move |_, ()| {
            disabled_config.lock().expect("aura config lock").enabled = false;
            Ok(())
        })?,
    )?;

    let effect_config = Arc::clone(&config);
    table.set(
        "effect_id",
        lua.create_function(move |_, effect_id: u32| {
            effect_config.lock().expect("aura config lock").effect_id = effect_id;
            Ok(())
        })?,
    )?;

    let force_config = Arc::clone(&config);
    table.set(
        "force_effect_id",
        lua.create_function(move |_, force: bool| {
            force_config
                .lock()
                .expect("aura config lock")
                .force_effect_id = force;
            Ok(())
        })?,
    )?;

    let target_config = Arc::clone(&config);
    table.set(
        "target",
        lua.create_function(move |_, target: String| {
            target_config.lock().expect("aura config lock").target =
                if target.eq_ignore_ascii_case("local_player") {
                    TargetMode::LocalPlayer
                } else {
                    TargetMode::All
                };
            Ok(())
        })?,
    )?;

    let trigger_config = Arc::clone(&config);
    table.set(
        "trigger",
        lua.create_function(move |_, trigger: String| {
            trigger_config.lock().expect("aura config lock").trigger =
                if trigger.eq_ignore_ascii_case("auto") {
                    TriggerMode::Auto
                } else {
                    TriggerMode::Hotkey
                };
            Ok(())
        })?,
    )?;

    let mode_config = Arc::clone(&config);
    table.set(
        "install_mode",
        lua.create_function(move |_, mode: String| {
            mode_config.lock().expect("aura config lock").install_mode =
                if mode.eq_ignore_ascii_case("patch") {
                    InstallMode::Patch
                } else {
                    InstallMode::ScanOnly
                };
            Ok(())
        })?,
    )?;

    let timing_config = Arc::clone(&config);
    table.set(
        "timing",
        lua.create_function(move |_, (speed, loop_start, loop_end): (f32, f32, f32)| {
            let mut config = timing_config.lock().expect("aura config lock");
            config.animation_speed = speed;
            config.loop_start = loop_start;
            config.loop_end = loop_end;
            Ok(())
        })?,
    )?;

    let wait_config = Arc::clone(&config);
    table.set(
        "wait_for",
        lua.create_function(move |_, wait_for: String| {
            wait_config.lock().expect("aura config lock").wait_for = match wait_for.as_str() {
                "none" => StatusGate::None,
                "virtual_resource" => StatusGate::VirtualResource,
                _ => StatusGate::DlcCharacter,
            };
            Ok(())
        })?,
    )?;

    Ok(table)
}

fn character_module(lua: &Lua, config: Arc<Mutex<AuraConfig>>) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    table.set(
        "all",
        lua.create_function(|lua, ()| {
            let characters = lua.create_table()?;
            for (index, character) in oppw4_character::all().iter().enumerate() {
                characters.set(index + 1, character_info_table(lua, character)?)?;
            }
            Ok(characters)
        })?,
    )?;

    let find_config = Arc::clone(&config);
    table.set(
        "find",
        lua.create_function(move |lua, name: String| {
            let Some(character) = oppw4_character::find(&name) else {
                return Ok(Value::Nil);
            };
            Ok(Value::Table(character_handle_table(
                lua,
                character,
                Arc::clone(&find_config),
            )?))
        })?,
    )?;

    let local_config = Arc::clone(&config);
    table.set(
        "local_player",
        lua.create_function(move |lua, ()| {
            local_player_handle_table(lua, Arc::clone(&local_config))
        })?,
    )?;

    Ok(table)
}

fn character_info_table(lua: &Lua, character: &Character) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("id", character.model_id)?;
    table.set("model_id", character.model_id)?;
    match character.playable_id {
        Some(playable_id) => table.set("playable_id", playable_id)?,
        None => table.set("playable_id", Value::Nil)?,
    }
    table.set("name", character.canonical.as_str())?;
    table.set("display_name", character.display_name.as_str())?;
    table.set("model", character.model_stem.as_str())?;
    Ok(table)
}

fn character_handle_table(
    lua: &Lua,
    character: &'static Character,
    config: Arc<Mutex<AuraConfig>>,
) -> mlua::Result<Table> {
    let table = character_info_table(lua, character)?;
    table.set("kind", "character")?;

    let add_fx_config = Arc::clone(&config);
    table.set(
        "add_fx",
        lua.create_function(move |lua, (this, options): (Table, Option<Table>)| {
            let handle = fx_handle_table(lua, Arc::clone(&add_fx_config))?;
            if let Some(options) = options {
                apply_fx_options(&add_fx_config, &options)?;
            }
            handle.set("character_id", this.get::<u16>("id")?)?;
            handle.set("character", this.get::<String>("name")?)?;
            Ok(handle)
        })?,
    )?;

    Ok(table)
}

fn local_player_handle_table(lua: &Lua, config: Arc<Mutex<AuraConfig>>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("id", -1)?;
    table.set("name", "local_player")?;
    table.set("display_name", "Local Player")?;
    table.set("kind", "local_player")?;

    let add_fx_config = Arc::clone(&config);
    table.set(
        "add_fx",
        lua.create_function(move |lua, (_this, options): (Table, Option<Table>)| {
            {
                let mut config = add_fx_config.lock().expect("aura config lock");
                config.target = TargetMode::LocalPlayer;
            }
            let handle = fx_handle_table(lua, Arc::clone(&add_fx_config))?;
            if let Some(options) = options {
                apply_fx_options(&add_fx_config, &options)?;
            }
            handle.set("character", "local_player")?;
            Ok(handle)
        })?,
    )?;

    Ok(table)
}

fn fx_handle_table(lua: &Lua, config: Arc<Mutex<AuraConfig>>) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("kind", "fx")?;

    let enable_config = Arc::clone(&config);
    table.set(
        "enable",
        lua.create_function(move |_, (_this, enabled): (Table, Option<bool>)| {
            enable_config.lock().expect("aura config lock").enabled = enabled.unwrap_or(true);
            Ok(())
        })?,
    )?;

    let disable_config = Arc::clone(&config);
    table.set(
        "disable",
        lua.create_function(move |_, _this: Table| {
            disable_config.lock().expect("aura config lock").enabled = false;
            Ok(())
        })?,
    )?;

    let effect_config = Arc::clone(&config);
    table.set(
        "effect_id",
        lua.create_function(move |_, (_this, effect_id): (Table, u32)| {
            effect_config.lock().expect("aura config lock").effect_id = effect_id;
            Ok(())
        })?,
    )?;

    let target_config = Arc::clone(&config);
    table.set(
        "target",
        lua.create_function(move |_, (_this, target): (Table, String)| {
            target_config.lock().expect("aura config lock").target = parse_target(&target);
            Ok(())
        })?,
    )?;

    let mode_config = Arc::clone(&config);
    table.set(
        "install_mode",
        lua.create_function(move |_, (_this, mode): (Table, String)| {
            mode_config.lock().expect("aura config lock").install_mode = parse_install_mode(&mode);
            Ok(())
        })?,
    )?;

    let timing_config = Arc::clone(&config);
    table.set(
        "timing",
        lua.create_function(
            move |_, (_this, speed, loop_start, loop_end): (Table, f32, f32, f32)| {
                let mut config = timing_config.lock().expect("aura config lock");
                config.animation_speed = speed;
                config.loop_start = loop_start;
                config.loop_end = loop_end;
                Ok(())
            },
        )?,
    )?;

    Ok(table)
}

fn apply_fx_options(config: &Arc<Mutex<AuraConfig>>, options: &Table) -> mlua::Result<()> {
    let mut config = config.lock().expect("aura config lock");
    if let Some(enabled) = options.get::<Option<bool>>("enabled")? {
        config.enabled = enabled;
    }
    if let Some(effect_id) = options.get::<Option<u32>>("effect_id")? {
        config.effect_id = effect_id;
    }
    if let Some(force_effect_id) = options.get::<Option<bool>>("force_effect_id")? {
        config.force_effect_id = force_effect_id;
    }
    if let Some(target) = options.get::<Option<String>>("target")? {
        config.target = parse_target(&target);
    }
    if let Some(trigger) = options.get::<Option<String>>("trigger")? {
        config.trigger = parse_trigger(&trigger);
    }
    if let Some(mode) = options.get::<Option<String>>("install_mode")? {
        config.install_mode = parse_install_mode(&mode);
    }
    if let Some(wait_for) = options.get::<Option<String>>("wait_for")? {
        config.wait_for = parse_wait_for(&wait_for);
    }
    if let Some(speed) = options.get::<Option<f32>>("speed")? {
        config.animation_speed = speed;
    }
    if let Some(loop_start) = options.get::<Option<f32>>("loop_start")? {
        config.loop_start = loop_start;
    }
    if let Some(loop_end) = options.get::<Option<f32>>("loop_end")? {
        config.loop_end = loop_end;
    }
    Ok(())
}

fn parse_target(value: &str) -> TargetMode {
    if value.eq_ignore_ascii_case("local_player") {
        TargetMode::LocalPlayer
    } else {
        TargetMode::All
    }
}

fn parse_trigger(value: &str) -> TriggerMode {
    if value.eq_ignore_ascii_case("auto") {
        TriggerMode::Auto
    } else {
        TriggerMode::Hotkey
    }
}

fn parse_install_mode(value: &str) -> InstallMode {
    if value.eq_ignore_ascii_case("patch") {
        InstallMode::Patch
    } else {
        InstallMode::ScanOnly
    }
}

fn parse_wait_for(value: &str) -> StatusGate {
    match value {
        "none" => StatusGate::None,
        "virtual_resource" => StatusGate::VirtualResource,
        _ => StatusGate::DlcCharacter,
    }
}
