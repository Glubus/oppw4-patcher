mod files;
mod game;
mod log;
mod lua;
mod memory;
mod mods;
mod paths;
mod r#unsafe;

use crate::abi::Oppw4PluginApi;

pub use files::FileService;
pub use game::GameService;
pub use log::LogService;
pub use lua::LuaService;
pub use memory::MemoryService;
pub use mods::ModService;
pub use paths::PathService;

#[derive(Clone, Copy)]
pub struct HostApi<'api> {
    abi: &'api Oppw4PluginApi,
}

#[derive(Clone, Copy)]
pub struct OwnedHostApi {
    abi: Oppw4PluginApi,
}

// SAFETY: `OwnedHostApi` is a copied host callback table plus opaque host pointers.
// The SDK never mutates the table. Thread-safety of the pointed-to host context is
// part of the host ABI contract because plugins may initialize worker threads and
// call host services from them.
unsafe impl Send for OwnedHostApi {}

// SAFETY: Shared references to `OwnedHostApi` only read immutable callback
// pointers and pass the opaque context back to the host. The host owns the context
// synchronization policy.
unsafe impl Sync for OwnedHostApi {}

impl<'api> HostApi<'api> {
    pub const fn new(abi: &'api Oppw4PluginApi) -> Self {
        Self { abi }
    }

    pub const fn abi(self) -> &'api Oppw4PluginApi {
        self.abi
    }

    pub const fn paths(self) -> PathService<'api> {
        PathService::new(self.abi)
    }

    pub const fn log(self) -> LogService<'api> {
        LogService::new(self.abi)
    }

    pub const fn memory(self) -> MemoryService<'api> {
        MemoryService::new(self.abi)
    }

    pub const fn mods(self) -> ModService<'api> {
        ModService::new(self.abi)
    }

    pub const fn files(self) -> FileService<'api> {
        FileService::new(self.abi)
    }

    pub const fn lua(self) -> LuaService<'api> {
        LuaService::new(self.abi)
    }

    pub const fn game(self) -> GameService<'api> {
        GameService::new(self.abi)
    }
}

impl OwnedHostApi {
    pub const fn new(abi: Oppw4PluginApi) -> Self {
        Self { abi }
    }

    pub const fn as_ref(&self) -> HostApi<'_> {
        HostApi::new(&self.abi)
    }

    pub const fn abi(self) -> Oppw4PluginApi {
        self.abi
    }

    pub const fn paths(&self) -> PathService<'_> {
        self.as_ref().paths()
    }

    pub const fn log(&self) -> LogService<'_> {
        self.as_ref().log()
    }

    pub const fn memory(&self) -> MemoryService<'_> {
        self.as_ref().memory()
    }

    pub const fn mods(&self) -> ModService<'_> {
        self.as_ref().mods()
    }

    pub const fn files(&self) -> FileService<'_> {
        self.as_ref().files()
    }

    pub const fn lua(&self) -> LuaService<'_> {
        self.as_ref().lua()
    }

    pub const fn game(&self) -> GameService<'_> {
        self.as_ref().game()
    }
}

impl<'api> From<&'api Oppw4PluginApi> for HostApi<'api> {
    fn from(abi: &'api Oppw4PluginApi) -> Self {
        Self::new(abi)
    }
}

impl From<Oppw4PluginApi> for OwnedHostApi {
    fn from(abi: Oppw4PluginApi) -> Self {
        Self::new(abi)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::{c_char, c_void, CStr},
        sync::{Mutex, OnceLock},
    };

    use crate::abi::{
        cstring_lossy, null_api, HostPluginModZipVisitorFn, Oppw4LogEntry, Oppw4PluginApi,
    };

    use super::{HostApi, OwnedHostApi};

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
    fn host_api_keeps_access_to_raw_abi_when_needed() {
        let abi = null_api();
        let host = HostApi::from(&abi);

        assert_eq!(host.abi().version, abi.version);
    }

    #[test]
    fn owned_host_api_can_be_copied_into_worker_state() {
        let owned = OwnedHostApi::from(Oppw4PluginApi {
            version: 42,
            ..null_api()
        });
        let copied = owned;

        assert_eq!(copied.abi().version, 42);
        assert_eq!(owned.as_ref().abi().version, 42);
    }

    #[test]
    fn host_log_service_forwards_plugin_id_and_message() {
        CAPTURED
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("capture lock")
            .clear();
        let api = Oppw4PluginApi {
            log: Some(capture_log),
            ..null_api()
        };

        HostApi::from(&api)
            .log()
            .write("skin_patcher", "hello")
            .expect("log write");

        assert_eq!(
            CAPTURED.get().unwrap().lock().unwrap().as_slice(),
            ["skin_patcher:hello"]
        );
    }

    #[test]
    fn host_mod_service_collects_legacy_paths() {
        let api = Oppw4PluginApi {
            for_each_plugin_mod_zip: Some(visit_mod_zips),
            ..null_api()
        };

        assert_eq!(
            HostApi::from(&api).mods().legacy_paths(),
            [
                r"D:\Game\OPPW4\plugins\skin_patcher\mods\a.zip",
                r"D:\Game\OPPW4\plugins\skin_patcher\mods\nested\b.zip",
            ]
        );
    }

    #[test]
    fn host_game_service_reports_debug_flag() {
        assert!(!HostApi::from(&null_api()).game().debug_enabled());
        let api = Oppw4PluginApi {
            debug_enabled: Some(debug_enabled),
            ..null_api()
        };

        assert!(HostApi::from(&api).game().debug_enabled());
    }

    #[allow(dead_code)]
    fn _keep_c_char(_: *const c_char) {}
}
