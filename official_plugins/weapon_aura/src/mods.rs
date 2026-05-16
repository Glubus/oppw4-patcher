use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use mlua::Lua;
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
        oppw4_lua_support::register_module(lua, "weapon_aura", table.clone())?;
        oppw4_lua_support::register_module(lua, "aura", table)
    });
    match result {
        Ok(()) => log::write_line(format!("weapon_aura lua mod applied id={id}")),
        Err(error) => log::write_line(format!("weapon_aura lua mod failed id={id} error={error:?}")),
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
            wait_config.lock().expect("aura config lock").wait_for =
                match wait_for.as_str() {
                    "none" => StatusGate::None,
                    "virtual_resource" => StatusGate::VirtualResource,
                    _ => StatusGate::DlcCharacter,
                };
            Ok(())
        })?,
    )?;

    Ok(table)
}
