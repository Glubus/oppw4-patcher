use std::{
    ffi::{c_char, c_void, CString},
    path::Path,
};

use oppw4_plugin_api::{
    optional_cstr, Oppw4LogEntry, Oppw4PluginApi, Oppw4VirtualReplacement, OPPW4_PLUGIN_API_VERSION,
};

use super::logs;

pub(crate) fn build_api(
    game_root: &Path,
    game_root_utf8: &CString,
    plugin_root_utf8: &CString,
    plugin_mods_root_utf8: &CString,
) -> Oppw4PluginApi {
    let _ = game_root;
    Oppw4PluginApi {
        version: OPPW4_PLUGIN_API_VERSION,
        host_context: std::ptr::null_mut(),
        game_root_utf8: game_root_utf8.as_ptr(),
        plugin_root_utf8: plugin_root_utf8.as_ptr(),
        plugin_mods_root_utf8: plugin_mods_root_utf8.as_ptr(),
        log: Some(host_log),
        clear_virtual_replacements: Some(host_clear_virtual_replacements),
        register_virtual_replacement: Some(host_register_virtual_replacement),
        commit_virtual_replacements: Some(host_commit_virtual_replacements),
        module_base: Some(host_module_base),
        read_memory: Some(host_read_memory),
        write_memory: Some(host_write_memory),
        scan_memory: Some(host_scan_memory),
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

unsafe extern "system" fn host_module_base(_host_context: *mut c_void) -> usize {
    oppw4_hooks::module_base()
}

unsafe extern "system" fn host_read_memory(
    _host_context: *mut c_void,
    address: usize,
    out: *mut u8,
    len: usize,
) -> i32 {
    oppw4_hooks::read_memory(address, out, len)
}

unsafe extern "system" fn host_write_memory(
    _host_context: *mut c_void,
    address: usize,
    bytes: *const u8,
    len: usize,
) -> i32 {
    oppw4_hooks::write_memory(address, bytes, len)
}

unsafe extern "system" fn host_scan_memory(
    _host_context: *mut c_void,
    pattern: *const u8,
    mask: *const u8,
    len: usize,
) -> usize {
    oppw4_hooks::scan_memory(pattern, mask, len)
}
