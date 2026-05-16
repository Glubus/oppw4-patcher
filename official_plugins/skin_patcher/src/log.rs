use std::sync::OnceLock;

use oppw4_plugin_api::{cstring_lossy, Oppw4PluginApi};

const PLUGIN_ID: &str = "skin_patcher";

static API: OnceLock<usize> = OnceLock::new();

pub fn initialize(api: *const Oppw4PluginApi) {
    let _ = API.set(api as usize);
}

pub fn write_line(message: impl AsRef<str>) {
    let Some(api) = API
        .get()
        .and_then(|api| unsafe { (*api as *const Oppw4PluginApi).as_ref() })
    else {
        return;
    };
    let plugin_id = cstring_lossy(PLUGIN_ID);
    let message = cstring_lossy(message.as_ref());
    api.log_line(&plugin_id, &message);
}
