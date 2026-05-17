use std::{ffi::CStr, fs, path::PathBuf};

use plugin_api::Oppw4PluginApi;

mod cycle;
mod fx;
mod plugin;

pub(crate) use cycle::{CycleConfig, CycleMode};
pub(crate) use fx::{FxConfig, TargetMode};
pub(crate) use plugin::{InstallMode, PluginConfig, StatusGate, TriggerMode};

pub(crate) fn load_plugin_config(api: &Oppw4PluginApi) -> PluginConfig {
    let root = unsafe { CStr::from_ptr(api.plugin_root_utf8) }
        .to_string_lossy()
        .into_owned();
    let path = PathBuf::from(root).join("config.toml");
    let Ok(text) = fs::read_to_string(path) else {
        return PluginConfig::default();
    };
    parse_plugin_config(&text).unwrap_or_default()
}

fn parse_plugin_config(text: &str) -> Option<PluginConfig> {
    let value = text.parse::<toml::Value>().ok()?;
    let config_type = value
        .get("config")
        .and_then(|config| config.get("type"))
        .and_then(toml::Value::as_str)?;
    if config_type != "fx_director" {
        return None;
    }

    let mut config = PluginConfig::default();
    if let Some(plugin) = value.get("plugin") {
        if let Some(mode) = plugin.get("install_mode").and_then(toml::Value::as_str) {
            config.install_mode = parse_install_mode(mode);
        }
    }
    if let Some(debug) = value.get("debug") {
        if let Some(observe_effect_ids) = debug
            .get("observe_effect_ids")
            .and_then(toml::Value::as_bool)
        {
            config.debug.observe_effect_ids = observe_effect_ids;
        }
        if let Some(observe_character_probe) = debug
            .get("observe_character_probe")
            .and_then(toml::Value::as_bool)
        {
            config.debug.observe_character_probe = observe_character_probe;
        }
    }
    Some(config)
}

fn parse_install_mode(value: &str) -> InstallMode {
    if value.eq_ignore_ascii_case("scan_only") {
        InstallMode::ScanOnly
    } else if value.eq_ignore_ascii_case("local_player_probe") {
        InstallMode::LocalPlayerProbe
    } else {
        InstallMode::Patch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_debug_config_is_separate_from_fx_definitions() {
        let config = parse_plugin_config(
            r#"
            [config]
            type = "fx_director"
            version = 1

            [debug]
            observe_effect_ids = true
            observe_character_probe = true
            "#,
        )
        .expect("config");

        assert!(config.debug.observe_effect_ids);
        assert!(config.debug.observe_character_probe);
        assert_eq!(config.install_mode, InstallMode::Patch);
    }
}
