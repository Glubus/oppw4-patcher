mod config;
mod hooks;
mod log;
mod memory;
mod mods;

use oppw4_plugin_api::{Oppw4PluginApi, OPPW4_PLUGIN_API_VERSION};

#[no_mangle]
pub unsafe extern "system" fn oppw4_plugin_init(api: *const Oppw4PluginApi) -> i32 {
    let Some(api) = api.as_ref() else {
        return -1;
    };
    if api.version != OPPW4_PLUGIN_API_VERSION {
        return -2;
    }
    log::initialize(api);
    let config = mods::load_config(api);
    log::write_line(format!(
        "weapon_aura init enabled={} mode={:?} trigger={:?} hotkey_vk=0x{:02x} target={:?} effect_id={} force_effect_id={} observe_effect_ids={} observe_character_probe={} speed={} loop_start={} loop_end={} install_delay_ms={} wait_for={:?} refresh_interval_ms={}",
        config.enabled,
        config.install_mode,
        config.trigger,
        config.hotkey_vk,
        config.target,
        config.effect_id,
        config.force_effect_id,
        config.observe_effect_ids,
        config.observe_character_probe,
        config.animation_speed,
        config.loop_start,
        config.loop_end,
        config.install_delay_ms,
        config.wait_for,
        config.refresh_interval_ms
    ));
    hooks::install_deferred(*api, config)
}

#[no_mangle]
pub extern "system" fn oppw4_weapon_aura_set_enabled(enabled: i32) -> i32 {
    hooks::set_enabled(enabled != 0)
}

#[no_mangle]
pub extern "system" fn oppw4_weapon_aura_set_effect_id(effect_id: u32) -> i32 {
    hooks::set_effect_id(effect_id)
}

#[no_mangle]
pub extern "system" fn oppw4_weapon_aura_set_timing(
    animation_speed: f32,
    loop_start: f32,
    loop_end: f32,
) -> i32 {
    hooks::set_timing(animation_speed, loop_start, loop_end)
}
