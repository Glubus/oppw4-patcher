use std::{
    ffi::{c_char, c_void, CStr, CString},
    ptr,
};

pub const OPPW4_PLUGIN_API_VERSION: u32 = 6;
pub const OPPW4_PLUGIN_INIT_SYMBOL: &[u8] = b"oppw4_plugin_init\0";

pub type PluginInitFn = unsafe extern "system" fn(api: *const Oppw4PluginApi) -> i32;
pub type HostLogFn =
    unsafe extern "system" fn(host_context: *mut c_void, entry: *const Oppw4LogEntry);
pub type HostModuleBaseFn = unsafe extern "system" fn(host_context: *mut c_void) -> usize;
pub type HostReadMemoryFn = unsafe extern "system" fn(
    host_context: *mut c_void,
    address: usize,
    out: *mut u8,
    len: usize,
) -> i32;
pub type HostWriteMemoryFn = unsafe extern "system" fn(
    host_context: *mut c_void,
    address: usize,
    bytes: *const u8,
    len: usize,
) -> i32;
pub type HostScanMemoryFn = unsafe extern "system" fn(
    host_context: *mut c_void,
    pattern: *const u8,
    mask: *const u8,
    len: usize,
) -> usize;
pub type HostPluginModZipVisitorFn =
    unsafe extern "system" fn(user_context: *mut c_void, path_utf8: *const c_char) -> i32;
pub type HostForEachPluginModZipFn = unsafe extern "system" fn(
    host_context: *mut c_void,
    visitor: Option<HostPluginModZipVisitorFn>,
    user_context: *mut c_void,
) -> i32;
pub type HostRegisterFileProviderFn =
    unsafe extern "system" fn(host_context: *mut c_void, provider: *const Oppw4FileProvider) -> i32;

pub type Oppw4ProviderOpenPathFn = unsafe extern "system" fn(
    provider_context: *mut c_void,
    path_utf8: *const c_char,
    out_handle: *mut u64,
) -> i32;
pub type Oppw4ProviderReadFn = unsafe extern "system" fn(
    provider_context: *mut c_void,
    handle: u64,
    buffer: *mut u8,
    bytes_to_read: u32,
    requested_offset: i64,
    out_bytes_read: *mut u32,
) -> i32;
pub type Oppw4ProviderCloseFn =
    unsafe extern "system" fn(provider_context: *mut c_void, handle: u64) -> i32;
pub type Oppw4ProviderSizeFn = unsafe extern "system" fn(
    provider_context: *mut c_void,
    handle: u64,
    out_size: *mut u64,
) -> i32;
pub type Oppw4ProviderFileTimeFn = unsafe extern "system" fn(
    provider_context: *mut c_void,
    handle: u64,
    out_filetime: *mut u64,
) -> i32;
pub type Oppw4ProviderSeekFn = unsafe extern "system" fn(
    provider_context: *mut c_void,
    handle: u64,
    distance: i64,
    move_method: u32,
    out_position: *mut u64,
) -> i32;
pub type Oppw4ProviderPatchReadFn = unsafe extern "system" fn(
    provider_context: *mut c_void,
    path_utf8: *const c_char,
    os_handle: usize,
    read_offset: u64,
    buffer: *mut u8,
    len: usize,
) -> i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Oppw4PluginApi {
    pub version: u32,
    pub host_context: *mut c_void,
    pub game_root_utf8: *const c_char,
    pub plugin_root_utf8: *const c_char,
    pub plugin_mods_root_utf8: *const c_char,
    pub log: Option<HostLogFn>,
    pub module_base: Option<HostModuleBaseFn>,
    pub read_memory: Option<HostReadMemoryFn>,
    pub write_memory: Option<HostWriteMemoryFn>,
    pub scan_memory: Option<HostScanMemoryFn>,
    pub for_each_plugin_mod_zip: Option<HostForEachPluginModZipFn>,
    pub register_file_provider: Option<HostRegisterFileProviderFn>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Oppw4LogEntry {
    pub plugin_id: *const c_char,
    pub message: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Oppw4FileProvider {
    pub plugin_id: *const c_char,
    pub provider_context: *mut c_void,
    pub open_path: Option<Oppw4ProviderOpenPathFn>,
    pub read: Option<Oppw4ProviderReadFn>,
    pub close: Option<Oppw4ProviderCloseFn>,
    pub size: Option<Oppw4ProviderSizeFn>,
    pub file_time: Option<Oppw4ProviderFileTimeFn>,
    pub seek: Option<Oppw4ProviderSeekFn>,
    pub patch_read: Option<Oppw4ProviderPatchReadFn>,
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

    pub fn register_file_provider(&self, provider: &Oppw4FileProvider) -> i32 {
        let Some(register) = self.register_file_provider else {
            return -1;
        };
        unsafe { register(self.host_context, provider) }
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

pub const fn null_api() -> Oppw4PluginApi {
    Oppw4PluginApi {
        version: OPPW4_PLUGIN_API_VERSION,
        host_context: ptr::null_mut(),
        game_root_utf8: ptr::null(),
        plugin_root_utf8: ptr::null(),
        plugin_mods_root_utf8: ptr::null(),
        log: None,
        module_base: None,
        read_memory: None,
        write_memory: None,
        scan_memory: None,
        for_each_plugin_mod_zip: None,
        register_file_provider: None,
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
            plugin_root_utf8: ptr::null(),
            plugin_mods_root_utf8: ptr::null(),
            log: Some(capture_log),
            module_base: None,
            read_memory: None,
            write_memory: None,
            scan_memory: None,
            for_each_plugin_mod_zip: None,
            register_file_provider: None,
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
            version: OPPW4_PLUGIN_API_VERSION,
            host_context: ptr::null_mut(),
            game_root_utf8: ptr::null(),
            plugin_root_utf8: ptr::null(),
            plugin_mods_root_utf8: ptr::null(),
            log: None,
            module_base: None,
            read_memory: None,
            write_memory: None,
            scan_memory: None,
            for_each_plugin_mod_zip: Some(visit_mod_zips),
            register_file_provider: None,
        };

        assert_eq!(
            api.plugin_mod_zips(),
            [
                r"D:\Game\OPPW4\plugins\skin_patcher\mods\a.zip",
                r"D:\Game\OPPW4\plugins\skin_patcher\mods\nested\b.zip",
            ]
        );
    }
}
