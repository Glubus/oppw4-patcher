use std::ffi::{c_char, c_void, CStr};

use crate::{
    ffi::{HostPluginModVisitorFn, HostPluginModZipVisitorFn},
    helpers::optional_cstr,
    structs::{
        Oppw4ActiveCharacter, Oppw4FileProvider, Oppw4GameStatus, Oppw4LogEntry, Oppw4LuaModule,
        Oppw4PluginApi, Oppw4PluginModEntry, PluginModInfo,
    },
};

impl Oppw4PluginApi {
    pub fn log_line(&self, plugin_id: &CStr, message: &CStr) {
        let Some(log) = self.log else {
            return;
        };
        let entry = Oppw4LogEntry {
            plugin_id: plugin_id.as_ptr(),
            message: message.as_ptr(),
        };
        unsafe { log(self.host_context, &entry) };
    }

    pub fn module_base(&self) -> usize {
        let Some(module_base) = self.module_base else {
            return 0;
        };
        unsafe { module_base(self.host_context) }
    }

    pub fn read_memory(&self, address: usize, out: &mut [u8]) -> i32 {
        let Some(read) = self.read_memory else {
            return -1;
        };
        unsafe { read(self.host_context, address, out.as_mut_ptr(), out.len()) }
    }

    pub fn write_memory(&self, address: usize, bytes: &[u8]) -> i32 {
        let Some(write) = self.write_memory else {
            return -1;
        };
        unsafe { write(self.host_context, address, bytes.as_ptr(), bytes.len()) }
    }

    pub fn scan_memory(&self, pattern: &[u8], mask: &[u8]) -> usize {
        if pattern.len() != mask.len() {
            return 0;
        }
        let Some(scan) = self.scan_memory else {
            return 0;
        };
        unsafe {
            scan(
                self.host_context,
                pattern.as_ptr(),
                mask.as_ptr(),
                pattern.len(),
            )
        }
    }

    pub fn plugin_mod_zips(&self) -> Vec<String> {
        self.legacy_mod_paths()
    }

    pub fn legacy_mod_paths(&self) -> Vec<String> {
        let Some(for_each) = self.for_each_plugin_mod_zip else {
            return Vec::new();
        };
        let mut paths = Vec::new();
        unsafe {
            let _ = for_each(
                self.host_context,
                Some(collect_plugin_mod_zip),
                (&mut paths as *mut Vec<String>).cast(),
            );
        }
        paths
    }

    pub fn plugin_mods(&self) -> Vec<PluginModInfo> {
        let Some(for_each) = self.for_each_plugin_mod else {
            return Vec::new();
        };
        let mut entries = Vec::new();
        unsafe {
            let _ = for_each(
                self.host_context,
                Some(collect_plugin_mod),
                (&mut entries as *mut Vec<PluginModInfo>).cast(),
            );
        }
        entries
    }

    pub fn register_file_provider(&self, provider: &Oppw4FileProvider) -> i32 {
        let Some(register) = self.register_file_provider else {
            return -1;
        };
        unsafe { register(self.host_context, provider) }
    }

    pub fn game_status(&self) -> Option<Oppw4GameStatus> {
        let status = self.game_status?;
        let mut out = Oppw4GameStatus::default();
        let result = unsafe { status(self.host_context, &mut out) };
        (result == 0).then_some(out)
    }

    pub fn register_lua_module(&self, module: &Oppw4LuaModule) -> i32 {
        let Some(register) = self.register_lua_module else {
            return -1;
        };
        unsafe { register(self.host_context, module) }
    }

    pub fn active_character(&self) -> Option<Oppw4ActiveCharacter> {
        let active_character = self.active_character?;
        let mut out = Oppw4ActiveCharacter::default();
        let result = unsafe { active_character(self.host_context, &mut out) };
        (result == 0).then_some(out)
    }

    pub fn debug_enabled(&self) -> bool {
        let Some(debug_enabled) = self.debug_enabled else {
            return false;
        };
        unsafe { debug_enabled(self.host_context) != 0 }
    }
}

unsafe extern "system" fn collect_plugin_mod_zip(
    user_context: *mut c_void,
    path_utf8: *const c_char,
) -> i32 {
    let Some(paths) = user_context.cast::<Vec<String>>().as_mut() else {
        return -1;
    };
    let Some(path) = optional_cstr(path_utf8) else {
        return -2;
    };
    paths.push(path.to_string_lossy().into_owned());
    0
}

unsafe extern "system" fn collect_plugin_mod(
    user_context: *mut c_void,
    entry: *const Oppw4PluginModEntry,
) -> i32 {
    let Some(entries) = user_context.cast::<Vec<PluginModInfo>>().as_mut() else {
        return -1;
    };
    let Some(entry) = entry.as_ref() else {
        return -2;
    };
    let Some(id) = optional_cstr(entry.id) else {
        return -3;
    };
    let Some(name) = optional_cstr(entry.name) else {
        return -4;
    };
    let Some(source_path) = optional_cstr(entry.source_path_utf8) else {
        return -5;
    };
    let Some(entry_lua) = optional_cstr(entry.entry_lua_utf8) else {
        return -6;
    };
    entries.push(PluginModInfo {
        id: id.to_string_lossy().into_owned(),
        name: name.to_string_lossy().into_owned(),
        source_path: source_path.to_string_lossy().into_owned(),
        entry_lua: entry_lua.to_string_lossy().into_owned(),
        flags: entry.flags,
    });
    0
}

#[allow(dead_code)]
fn _keep_visitor_types(_: Option<HostPluginModZipVisitorFn>, _: Option<HostPluginModVisitorFn>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cstring_lossy, null_api, HostPluginModZipVisitorFn};
    use std::sync::{Mutex, OnceLock};

    static CAPTURED: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

    unsafe extern "system" fn capture_log(_host_context: *mut c_void, entry: *const Oppw4LogEntry) {
        let entry = &*entry;
        let plugin_id = CStr::from_ptr(entry.plugin_id).to_string_lossy();
        let message = CStr::from_ptr(entry.message).to_string_lossy();
        CAPTURED
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("capture lock")
            .push(format!("{plugin_id}:{message}"));
    }

    unsafe extern "system" fn visit_mod_zips(
        _host_context: *mut c_void,
        visitor: Option<HostPluginModZipVisitorFn>,
        user_context: *mut c_void,
    ) -> i32 {
        let Some(visitor) = visitor else {
            return -1;
        };
        let a = cstring_lossy(r"D:\Game\OPPW4\plugins\skin_patcher\mods\a.zip");
        let b = cstring_lossy(r"D:\Game\OPPW4\plugins\skin_patcher\mods\nested\b.zip");
        if visitor(user_context, a.as_ptr()) != 0 {
            return -2;
        }
        visitor(user_context, b.as_ptr())
    }

    unsafe extern "system" fn debug_enabled(_host_context: *mut c_void) -> i32 {
        1
    }

    #[test]
    fn api_log_line_forwards_plugin_id_and_message() {
        CAPTURED
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("capture lock")
            .clear();
        let api = Oppw4PluginApi {
            log: Some(capture_log),
            ..null_api()
        };
        let plugin = cstring_lossy("skin_patcher");
        let message = cstring_lossy("hello");

        api.log_line(&plugin, &message);

        assert_eq!(
            CAPTURED.get().unwrap().lock().unwrap().as_slice(),
            ["skin_patcher:hello"]
        );
    }

    #[test]
    fn api_collects_plugin_mod_zip_paths_from_host() {
        let api = Oppw4PluginApi {
            for_each_plugin_mod_zip: Some(visit_mod_zips),
            ..null_api()
        };

        assert_eq!(
            api.plugin_mod_zips(),
            [
                r"D:\Game\OPPW4\plugins\skin_patcher\mods\a.zip",
                r"D:\Game\OPPW4\plugins\skin_patcher\mods\nested\b.zip",
            ]
        );
    }

    #[test]
    fn api_reports_debug_flag_from_host() {
        assert!(!null_api().debug_enabled());
        let api = Oppw4PluginApi {
            debug_enabled: Some(debug_enabled),
            ..null_api()
        };

        assert!(api.debug_enabled());
    }
}
