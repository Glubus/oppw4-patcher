use std::{
    ffi::{c_char, c_void, CStr, CString},
    ptr,
};

pub const OPPW4_PLUGIN_API_VERSION: u32 = 3;
pub const OPPW4_PLUGIN_INIT_SYMBOL: &[u8] = b"oppw4_plugin_init\0";

pub type PluginInitFn = unsafe extern "system" fn(api: *const Oppw4PluginApi) -> i32;
pub type HostLogFn =
    unsafe extern "system" fn(host_context: *mut c_void, entry: *const Oppw4LogEntry);
pub type HostClearVirtualReplacementsFn =
    unsafe extern "system" fn(host_context: *mut c_void, plugin_id: *const c_char) -> i32;
pub type HostRegisterVirtualReplacementFn = unsafe extern "system" fn(
    host_context: *mut c_void,
    replacement: *const Oppw4VirtualReplacement,
) -> i32;
pub type HostCommitVirtualReplacementsFn =
    unsafe extern "system" fn(host_context: *mut c_void, plugin_id: *const c_char) -> i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Oppw4PluginApi {
    pub version: u32,
    pub host_context: *mut c_void,
    pub game_root_utf8: *const c_char,
    pub log: Option<HostLogFn>,
    pub clear_virtual_replacements: Option<HostClearVirtualReplacementsFn>,
    pub register_virtual_replacement: Option<HostRegisterVirtualReplacementFn>,
    pub commit_virtual_replacements: Option<HostCommitVirtualReplacementsFn>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Oppw4LogEntry {
    pub plugin_id: *const c_char,
    pub message: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Oppw4ByteSlice {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oppw4ReplacementSourceKind {
    File = 0,
    ZipEntry = 1,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oppw4ReplacementMode {
    Virtual = 0,
    Internal = 1,
    External = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Oppw4VirtualReplacement {
    pub plugin_id: *const c_char,
    pub archive_name: *const c_char,
    pub file_name: *const c_char,
    pub source_kind: Oppw4ReplacementSourceKind,
    pub source_path: *const c_char,
    pub source_entry_name: *const c_char,
    pub mode: Oppw4ReplacementMode,
    pub has_mod_size: i32,
    pub mod_size: u64,
    pub hash: u32,
    pub rdb_block_offset: u64,
    pub original_data_offset: u32,
    pub has_original_bin_offset: i32,
    pub original_bin_offset: u32,
    pub has_original_bin_size: i32,
    pub original_bin_size: u32,
    pub has_virtual_bin_offset: i32,
    pub virtual_bin_offset: u64,
    pub has_rdb_tail_offset: i32,
    pub rdb_tail_offset: u64,
    pub original_tail: *const c_char,
    pub virtual_prefix: Oppw4ByteSlice,
}

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

    pub fn clear_virtual_replacements(&self, plugin_id: &CStr) -> i32 {
        let Some(clear) = self.clear_virtual_replacements else {
            return -1;
        };
        unsafe { clear(self.host_context, plugin_id.as_ptr()) }
    }

    pub fn register_virtual_replacement(&self, replacement: &Oppw4VirtualReplacement) -> i32 {
        let Some(register) = self.register_virtual_replacement else {
            return -1;
        };
        unsafe { register(self.host_context, replacement) }
    }

    pub fn commit_virtual_replacements(&self, plugin_id: &CStr) -> i32 {
        let Some(commit) = self.commit_virtual_replacements else {
            return -1;
        };
        unsafe { commit(self.host_context, plugin_id.as_ptr()) }
    }
}

pub fn cstring_lossy(value: impl AsRef<str>) -> CString {
    let bytes = value
        .as_ref()
        .as_bytes()
        .iter()
        .copied()
        .filter(|byte| *byte != 0)
        .collect::<Vec<_>>();
    CString::new(bytes).unwrap_or_else(|_| CString::new("").expect("empty cstring"))
}

pub unsafe fn optional_cstr<'a>(value: *const c_char) -> Option<&'a CStr> {
    (!value.is_null()).then(|| CStr::from_ptr(value))
}

pub const fn null_api() -> Oppw4PluginApi {
    Oppw4PluginApi {
        version: OPPW4_PLUGIN_API_VERSION,
        host_context: ptr::null_mut(),
        game_root_utf8: ptr::null(),
        log: None,
        clear_virtual_replacements: None,
        register_virtual_replacement: None,
        commit_virtual_replacements: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn api_log_line_forwards_plugin_id_and_message() {
        CAPTURED
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("capture lock")
            .clear();
        let api = Oppw4PluginApi {
            version: OPPW4_PLUGIN_API_VERSION,
            host_context: ptr::null_mut(),
            game_root_utf8: ptr::null(),
            log: Some(capture_log),
            clear_virtual_replacements: None,
            register_virtual_replacement: None,
            commit_virtual_replacements: None,
        };
        let plugin = cstring_lossy("skin_patcher");
        let message = cstring_lossy("hello");

        api.log_line(&plugin, &message);

        assert_eq!(
            CAPTURED.get().unwrap().lock().unwrap().as_slice(),
            ["skin_patcher:hello"]
        );
    }
}
