mod config;
mod hooks;
mod log;
mod memory;
mod mods;

use plugin_sdk::{plugin_abi_from_raw, HostApi, Oppw4PluginApi, OwnedHostApi};

#[no_mangle]
pub unsafe extern "system" fn oppw4_plugin_init(api: *const Oppw4PluginApi) -> i32 {
    let api = match plugin_abi_from_raw(api) {
        Ok(api) => api,
        Err(error) => return error.code(),
    };
    let host = HostApi::from(api);
    log::initialize(host);
    let shared_config = mods::load_config(host);
    mods::register_lua_modules(host, shared_config.clone());
    let plugin = shared_config.lock().expect("fx state lock").plugin_config();
    log::write_line(format!(
        "fx_director init trigger={:?} hotkey_vk=0x{:02x} observe_effect_ids={} observe_character_probe={} install_delay_ms={} wait_for={:?} refresh_interval_ms={}",
        plugin.trigger,
        plugin.hotkey_vk,
        plugin.debug.observe_effect_ids,
        plugin.debug.observe_character_probe,
        plugin.install_delay_ms,
        plugin.wait_for,
        plugin.refresh_interval_ms
    ));
    hooks::install_deferred(OwnedHostApi::from(*api), shared_config)
}

pub extern "system" fn oppw4_fx_director_set_enabled(enabled: i32) -> i32 {
    hooks::set_enabled(enabled != 0)
}

#[no_mangle]
pub extern "system" fn oppw4_fx_director_set_effect_id(effect_id: u32) -> i32 {
    hooks::set_effect_id(effect_id)
}

#[no_mangle]
pub extern "system" fn oppw4_fx_director_set_timing(
    animation_speed: f32,
    loop_start: f32,
    loop_end: f32,
) -> i32 {
    hooks::set_timing(animation_speed, loop_start, loop_end)
}
