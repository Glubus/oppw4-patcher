use std::{
    ffi::{c_void, CString},
    path::{Path, PathBuf},
};

use oppw4_plugin_api::{
    optional_cstr, HostPluginModVisitorFn, HostPluginModZipVisitorFn, Oppw4FileProvider,
    Oppw4GameStatus, Oppw4LogEntry, Oppw4PluginApi, Oppw4PluginModEntry,
    OPPW4_PLUGIN_API_VERSION, OPPW4_PLUGIN_MOD_FLAG_ZIP,
};

use super::{logs, mods};

pub(crate) struct ApiContext {
    plugin_id: String,
    plugin_mods_root: PathBuf,
}

impl ApiContext {
    pub(crate) fn new(plugin_id: String, plugin_mods_root: PathBuf) -> Self {
        Self {
            plugin_id,
            plugin_mods_root,
        }
    }
}

pub(crate) fn build_api(
    game_root: &Path,
    game_root_utf8: &CString,
    plugin_root_utf8: &CString,
    plugin_mods_root_utf8: &CString,
    context: &ApiContext,
) -> Oppw4PluginApi {
    let _ = game_root;
    Oppw4PluginApi {
        version: OPPW4_PLUGIN_API_VERSION,
        host_context: (context as *const ApiContext).cast_mut().cast(),
        game_root_utf8: game_root_utf8.as_ptr(),
        plugin_root_utf8: plugin_root_utf8.as_ptr(),
        plugin_mods_root_utf8: plugin_mods_root_utf8.as_ptr(),
        log: Some(host_log),
        module_base: Some(host_module_base),
        read_memory: Some(host_read_memory),
        write_memory: Some(host_write_memory),
        scan_memory: Some(host_scan_memory),
        for_each_plugin_mod_zip: Some(host_for_each_plugin_mod_zip),
        for_each_plugin_mod: Some(host_for_each_plugin_mod),
        register_file_provider: Some(host_register_file_provider),
        game_status: Some(host_game_status),
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

unsafe extern "system" fn host_module_base(_host_context: *mut c_void) -> usize {
    oppw4_hooks::module_base()
}

unsafe extern "system" fn host_register_file_provider(
    _host_context: *mut c_void,
    provider: *const Oppw4FileProvider,
) -> i32 {
    oppw4_hooks::register_file_provider(provider)
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

unsafe extern "system" fn host_game_status(
    _host_context: *mut c_void,
    out_status: *mut Oppw4GameStatus,
) -> i32 {
    let Some(out_status) = out_status.as_mut() else {
        return -1;
    };
    *out_status = oppw4_hooks::game_status();
    0
}

unsafe extern "system" fn host_for_each_plugin_mod_zip(
    host_context: *mut c_void,
    visitor: Option<HostPluginModZipVisitorFn>,
    user_context: *mut c_void,
) -> i32 {
    let Some(context) = host_context.cast::<ApiContext>().as_ref() else {
        return -1;
    };
    let Some(visitor) = visitor else {
        return -2;
    };
    for path in mods::list_legacy_paths(&context.plugin_mods_root) {
        let path = cstring_lossy(&path.to_string_lossy());
        let result = visitor(user_context, path.as_ptr());
        if result != 0 {
            return result;
        }
    }
    0
}

unsafe extern "system" fn host_for_each_plugin_mod(
    host_context: *mut c_void,
    visitor: Option<HostPluginModVisitorFn>,
    user_context: *mut c_void,
) -> i32 {
    let Some(context) = host_context.cast::<ApiContext>().as_ref() else {
        return -1;
    };
    let Some(visitor) = visitor else {
        return -2;
    };

    for mod_entry in oppw4_lua_support::discover_mods(&context.plugin_mods_root) {
        if !mod_entry.uses_plugin(&context.plugin_id) {
            continue;
        }
        let id = cstring_lossy(&mod_entry.manifest.id);
        let name = cstring_lossy(&mod_entry.manifest.name);
        let source_path = cstring_lossy(&mod_entry.source_path().to_string_lossy());
        let entry_lua = cstring_lossy(&mod_entry.manifest.entry_lua);
        let entry = Oppw4PluginModEntry {
            id: id.as_ptr(),
            name: name.as_ptr(),
            source_path_utf8: source_path.as_ptr(),
            entry_lua_utf8: entry_lua.as_ptr(),
            flags: if mod_entry.is_zip() {
                OPPW4_PLUGIN_MOD_FLAG_ZIP
            } else {
                0
            },
        };
        let result = visitor(user_context, &entry);
        if result != 0 {
            return result;
        }
    }
    0
}
