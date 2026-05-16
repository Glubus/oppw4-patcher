use std::{ffi::CStr, fs, path::PathBuf};

use oppw4_plugin_api::Oppw4PluginApi;

#[derive(Clone, Copy, Debug)]
pub(crate) struct AuraConfig {
    pub(crate) enabled: bool,
    pub(crate) install_mode: InstallMode,
    pub(crate) trigger: TriggerMode,
    pub(crate) hotkey_vk: i32,
    pub(crate) target: TargetMode,
    pub(crate) effect_id: u32,
    pub(crate) force_effect_id: bool,
    pub(crate) observe_effect_ids: bool,
    pub(crate) observe_character_probe: bool,
    pub(crate) animation_speed: f32,
    pub(crate) loop_start: f32,
    pub(crate) loop_end: f32,
    pub(crate) install_delay_ms: u64,
    pub(crate) wait_for: StatusGate,
    pub(crate) refresh_interval_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InstallMode {
    ScanOnly,
    Patch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetMode {
    All,
    LocalPlayer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriggerMode {
    Auto,
    Hotkey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StatusGate {
    None,
    VirtualResource,
    DlcCharacter,
}

impl Default for AuraConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            install_mode: InstallMode::ScanOnly,
            trigger: TriggerMode::Hotkey,
            hotkey_vk: 0x77,
            target: TargetMode::All,
            effect_id: 2830,
            force_effect_id: true,
            observe_effect_ids: false,
            observe_character_probe: false,
            animation_speed: 0.6,
            loop_start: 0.1,
            loop_end: 1.9,
            install_delay_ms: 8000,
            wait_for: StatusGate::DlcCharacter,
            refresh_interval_ms: 500,
        }
    }
}

impl AuraConfig {
    pub(crate) fn load(api: &Oppw4PluginApi) -> Option<Self> {
        let root = unsafe { CStr::from_ptr(api.plugin_root_utf8) }
            .to_string_lossy()
            .into_owned();
        let path = PathBuf::from(root).join("config.toml");
        let text = fs::read_to_string(path).ok()?;
        let mut config = Self::default();
        config.apply_toml_text(&text).ok()?;
        Some(config)
    }

    pub(crate) fn apply_toml_text(&mut self, text: &str) -> Result<(), ()> {
        let value = text.parse::<toml::Value>().map_err(|_| ())?;
        let aura = value
            .get("aura")
            .and_then(toml::Value::as_table)
            .ok_or(())?;
        if let Some(enabled) = aura.get("enabled").and_then(toml::Value::as_bool) {
            self.enabled = enabled;
        }
        if let Some(mode) = aura.get("install_mode").and_then(toml::Value::as_str) {
            self.install_mode = match mode {
                "patch" => InstallMode::Patch,
                _ => InstallMode::ScanOnly,
            };
        }
        if let Some(trigger) = aura.get("trigger").and_then(toml::Value::as_str) {
            self.trigger = match trigger {
                "auto" => TriggerMode::Auto,
                _ => TriggerMode::Hotkey,
            };
        }
        if let Some(hotkey) = aura.get("hotkey_vk").and_then(toml::Value::as_integer) {
            self.hotkey_vk = hotkey.clamp(1, 255) as i32;
        }
        if let Some(target) = aura.get("target").and_then(toml::Value::as_str) {
            self.target = match target {
                "local_player" => TargetMode::LocalPlayer,
                _ => TargetMode::All,
            };
        }
        if let Some(effect_id) = aura.get("effect_id").and_then(toml::Value::as_integer) {
            self.effect_id = effect_id.max(0) as u32;
        }
        if let Some(force_effect_id) = aura.get("force_effect_id").and_then(toml::Value::as_bool) {
            self.force_effect_id = force_effect_id;
        }
        if let Some(observe_effect_ids) = aura
            .get("observe_effect_ids")
            .and_then(toml::Value::as_bool)
        {
            self.observe_effect_ids = observe_effect_ids;
        }
        if let Some(observe_character_probe) = aura
            .get("observe_character_probe")
            .and_then(toml::Value::as_bool)
        {
            self.observe_character_probe = observe_character_probe;
        }
        if let Some(speed) = aura.get("animation_speed").and_then(toml::Value::as_float) {
            self.animation_speed = speed as f32;
        }
        if let Some(loop_start) = aura.get("loop_start").and_then(toml::Value::as_float) {
            self.loop_start = loop_start as f32;
        }
        if let Some(loop_end) = aura.get("loop_end").and_then(toml::Value::as_float) {
            self.loop_end = loop_end as f32;
        }
        if let Some(delay) = aura.get("install_delay_ms").and_then(toml::Value::as_integer) {
            self.install_delay_ms = delay.max(0) as u64;
        }
        if let Some(wait_for) = aura.get("wait_for").and_then(toml::Value::as_str) {
            self.wait_for = match wait_for {
                "none" => StatusGate::None,
                "virtual_resource" => StatusGate::VirtualResource,
                _ => StatusGate::DlcCharacter,
            };
        }
        if let Some(interval) = aura
            .get("refresh_interval_ms")
            .and_then(toml::Value::as_integer)
        {
            self.refresh_interval_ms = interval.max(0) as u64;
        }
        Ok(())
    }
}
