use std::ffi::{c_char, c_void};

use crate::structs::{
    Oppw4ActiveCharacter, Oppw4FileProvider, Oppw4GameStatus, Oppw4LogEntry, Oppw4LuaModule,
    Oppw4PluginModEntry,
};

pub const OPPW4_PLUGIN_API_VERSION: u32 = 11;
pub const OPPW4_PLUGIN_INIT_SYMBOL: &[u8] = b"oppw4_plugin_init\0";

pub const OPPW4_GAME_PHASE_UNKNOWN: u32 = 0;
pub const OPPW4_GAME_PHASE_BOOTING: u32 = 1;
pub const OPPW4_GAME_PHASE_RDB_LOADING: u32 = 2;
pub const OPPW4_GAME_PHASE_RDB_BIN_LOADING: u32 = 3;
pub const OPPW4_GAME_PHASE_DLC_CHARACTER_LOADING: u32 = 4;
pub const OPPW4_GAME_PHASE_VIRTUAL_RESOURCE_LOADING: u32 = 5;

pub const OPPW4_GAME_FLAG_DLC_CHARACTER_SEEN: u32 = 1 << 0;
pub const OPPW4_GAME_FLAG_VIRTUAL_RESOURCE_SEEN: u32 = 1 << 1;

pub const OPPW4_PLUGIN_MOD_FLAG_ZIP: u32 = 1 << 0;

pub type PluginInitFn = unsafe extern "system" fn(api: *const crate::Oppw4PluginApi) -> i32;
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
pub type HostPluginModVisitorFn =
    unsafe extern "system" fn(user_context: *mut c_void, entry: *const Oppw4PluginModEntry) -> i32;
pub type HostForEachPluginModFn = unsafe extern "system" fn(
    host_context: *mut c_void,
    visitor: Option<HostPluginModVisitorFn>,
    user_context: *mut c_void,
) -> i32;
pub type HostRegisterFileProviderFn =
    unsafe extern "system" fn(host_context: *mut c_void, provider: *const Oppw4FileProvider) -> i32;
pub type HostGameStatusFn =
    unsafe extern "system" fn(host_context: *mut c_void, out_status: *mut Oppw4GameStatus) -> i32;
pub type HostRegisterLuaModuleFn =
    unsafe extern "system" fn(host_context: *mut c_void, module: *const Oppw4LuaModule) -> i32;
pub type HostActiveCharacterFn =
    unsafe extern "system" fn(host_context: *mut c_void, out: *mut Oppw4ActiveCharacter) -> i32;
pub type HostDebugEnabledFn = unsafe extern "system" fn(host_context: *mut c_void) -> i32;

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

pub type Oppw4LuaRegisterFn =
    unsafe extern "system" fn(module_context: *mut c_void, lua_context: *mut c_void) -> i32;
