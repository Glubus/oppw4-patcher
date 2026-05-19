use std::{
    ffi::{c_char, c_void, CStr},
    path::Path,
};

use plugin_sdk::{cstring_lossy, HostApi, Oppw4FileProvider, PluginError};

use crate::{constants::*, log, state};

pub(crate) fn register(host: HostApi<'_>) -> i32 {
    let plugin_id = cstring_lossy(PLUGIN_ID);
    let provider = Oppw4FileProvider {
        plugin_id: plugin_id.as_ptr(),
        provider_context: std::ptr::null_mut(),
        open_path: Some(provider_open_path),
        read: Some(provider_read),
        close: Some(provider_close),
        size: Some(provider_size),
        file_time: None,
        seek: Some(provider_seek),
        patch_read: None,
    };
    let result = match host.files().register_provider(&provider) {
        Ok(()) => 0,
        Err(PluginError::HostCallFailed { code, .. }) => code,
        Err(_) => -1,
    };
    log::write(
        host,
        format!("moveset_patcher file provider result={result}"),
    );
    result
}

unsafe extern "system" fn provider_open_path(
    _context: *mut c_void,
    path_utf8: *const c_char,
    out_handle: *mut u64,
) -> i32 {
    if path_utf8.is_null() || out_handle.is_null() {
        return -1;
    }
    let path = CStr::from_ptr(path_utf8).to_string_lossy();
    if !is_linkdata_a_path(&path) {
        return 0;
    }
    match state::with_mut(|state| state.open()) {
        Some(Ok(Some(handle))) => {
            log::write_global(format!(
                "moveset_patcher virtual LINKDATA_A opened path={path} handle={handle}"
            ));
            *out_handle = handle;
            1
        }
        Some(Ok(None)) => {
            log::write_global(format!(
                "moveset_patcher LINKDATA_A pass-through no patches path={path}"
            ));
            0
        }
        Some(Err(error)) => {
            log::write_global(format!(
                "moveset_patcher LINKDATA_A virtual open failed path={path} error={error}"
            ));
            0
        }
        None => 0,
    }
}

unsafe extern "system" fn provider_read(
    _context: *mut c_void,
    handle: u64,
    buffer: *mut u8,
    bytes_to_read: u32,
    requested_offset: i64,
    out_bytes_read: *mut u32,
) -> i32 {
    state::with_mut(|state| {
        state.read(
            handle,
            buffer,
            bytes_to_read,
            requested_offset,
            out_bytes_read,
        )
    })
    .unwrap_or(0)
}

unsafe extern "system" fn provider_close(_context: *mut c_void, handle: u64) -> i32 {
    state::with_mut(|state| state.close(handle)).unwrap_or(0)
}

unsafe extern "system" fn provider_size(
    _context: *mut c_void,
    _handle: u64,
    out_size: *mut u64,
) -> i32 {
    if out_size.is_null() {
        return -1;
    }
    state::with_mut(|state| state.size(out_size)).unwrap_or(0)
}

unsafe extern "system" fn provider_seek(
    _context: *mut c_void,
    handle: u64,
    distance: i64,
    move_method: u32,
    out_position: *mut u64,
) -> i32 {
    state::with_mut(|state| state.seek(handle, distance, move_method, out_position)).unwrap_or(0)
}

fn is_linkdata_a_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case(LINKDATA_A_NAME))
}
