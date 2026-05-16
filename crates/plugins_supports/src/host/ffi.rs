use std::{
    ffi::{c_char, c_void, CString},
    path::Path,
};

use oppw4_plugin_api::{
    optional_cstr, Oppw4LogEntry, Oppw4PluginApi, Oppw4VirtualReplacement, OPPW4_PLUGIN_API_VERSION,
};

use super::logs;

pub(crate) fn build_api(game_root: &Path, game_root_utf8: &CString) -> Oppw4PluginApi {
    let _ = game_root;
    Oppw4PluginApi {
        version: OPPW4_PLUGIN_API_VERSION,
        host_context: std::ptr::null_mut(),
        game_root_utf8: game_root_utf8.as_ptr(),
        log: Some(host_log),
        clear_virtual_replacements: Some(host_clear_virtual_replacements),
        register_virtual_replacement: Some(host_register_virtual_replacement),
        commit_virtual_replacements: Some(host_commit_virtual_replacements),
    }
}

pub(crate) fn cstring_lossy(value: &str) -> CString {
    let bytes = value
        .as_bytes()
        .iter()
        .copied()
        .filter(|byte| *byte != 0)
        .collect::<Vec<_>>();
    CString::new(bytes).unwrap_or_else(|_| CString::new("").expect("empty cstring"))
}

unsafe extern "system" fn host_log(_host_context: *mut c_void, entry: *const Oppw4LogEntry) {
    let Some(entry) = entry.as_ref() else {
        return;
    };
    let Some(plugin_id) = optional_cstr(entry.plugin_id) else {
        return;
    };
    let Some(message) = optional_cstr(entry.message) else {
        return;
    };
    logs::write(plugin_id, message);
}

unsafe extern "system" fn host_clear_virtual_replacements(
    _host_context: *mut c_void,
    plugin_id: *const c_char,
) -> i32 {
    oppw4_hooks::clear_virtual_replacements(plugin_id)
}

unsafe extern "system" fn host_register_virtual_replacement(
    _host_context: *mut c_void,
    replacement: *const Oppw4VirtualReplacement,
) -> i32 {
    oppw4_hooks::register_virtual_replacement(replacement)
}

unsafe extern "system" fn host_commit_virtual_replacements(
    _host_context: *mut c_void,
    plugin_id: *const c_char,
) -> i32 {
    oppw4_hooks::commit_virtual_replacements(plugin_id)
}
