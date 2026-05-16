use oppw4_plugin_api::{cstring_lossy, Oppw4PluginApi, OPPW4_PLUGIN_API_VERSION};

#[no_mangle]
pub unsafe extern "system" fn oppw4_plugin_init(api: *const Oppw4PluginApi) -> i32 {
    let Some(api) = api.as_ref() else {
        return -1;
    };
    if api.version != OPPW4_PLUGIN_API_VERSION {
        return -2;
    }

    let plugin_id = cstring_lossy("skin_patcher");
    let message = cstring_lossy("skin_patcher plugin initialized");
    api.log_line(&plugin_id, &message);
    0
}
