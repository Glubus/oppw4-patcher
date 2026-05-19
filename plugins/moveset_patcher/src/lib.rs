use plugin_sdk::{plugin_abi_from_raw, HostApi, Oppw4PluginApi};

mod constants;
mod hex;
mod linkdata;
mod log;
mod lua;
mod payload;
mod provider;
mod state;

#[no_mangle]
pub unsafe extern "system" fn oppw4_plugin_init(api: *const Oppw4PluginApi) -> i32 {
    let api = match plugin_abi_from_raw(api) {
        Ok(api) => api,
        Err(error) => return error.code(),
    };
    let host = HostApi::from(api);
    match initialize(host) {
        Ok(()) => 0,
        Err(error) => {
            log::write(host, format!("moveset_patcher init failed: {error}"));
            -1
        }
    }
}

fn initialize(host: HostApi<'_>) -> Result<(), String> {
    log::init(host);
    let game_root = host
        .paths()
        .game_root()
        .ok_or_else(|| "missing game root".to_string())?;
    let mods_root = host
        .paths()
        .mods_root()
        .ok_or_else(|| "missing mods root".to_string())?;
    state::initialize(game_root, mods_root)?;
    let edits = state::with_mut(|state| state.edit_count()).unwrap_or(0);
    log::write(
        host,
        format!("moveset_patcher initialized legacy_entry_patches={edits}"),
    );
    provider::register(host);
    lua::register(host);
    Ok(())
}
