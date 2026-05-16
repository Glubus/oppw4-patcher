use std::{ffi::c_void, sync::OnceLock};

use oppw4_plugin_api::{cstring_lossy, HostLogFn, Oppw4LogEntry, Oppw4PluginApi};

#[derive(Clone, Copy)]
struct Logger {
    host_context: usize,
    log: HostLogFn,
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn initialize(api: &Oppw4PluginApi) {
    let Some(log) = api.log else {
        return;
    };
    let _ = LOGGER.set(Logger {
        host_context: api.host_context as usize,
        log,
    });
}

pub fn write_line(message: impl AsRef<str>) {
    let Some(logger) = LOGGER.get().copied() else {
        return;
    };
    let plugin_id = cstring_lossy("weapon_aura");
    let message = cstring_lossy(message.as_ref());
    let entry = Oppw4LogEntry {
        plugin_id: plugin_id.as_ptr(),
        message: message.as_ptr(),
    };
    unsafe { (logger.log)(logger.host_context as *mut c_void, &entry) };
}
