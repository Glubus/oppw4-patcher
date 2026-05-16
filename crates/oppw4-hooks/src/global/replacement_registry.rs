use std::{
    ffi::{c_char, CStr},
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use oppw4_plugin_api::{Oppw4ReplacementMode, Oppw4ReplacementSourceKind, Oppw4VirtualReplacement};
use oppw4_rdb::{ReplacementMode, ReplacementSource, VirtualReplacement};

static PENDING: OnceLock<Mutex<Vec<VirtualReplacement>>> = OnceLock::new();

pub unsafe fn clear(_plugin_id: *const c_char) -> i32 {
    let pending = PENDING.get_or_init(|| Mutex::new(Vec::new()));
    let Ok(mut pending) = pending.lock() else {
        return -1;
    };
    pending.clear();
    0
}

pub unsafe fn register(replacement: *const Oppw4VirtualReplacement) -> i32 {
    let Some(replacement) = replacement.as_ref() else {
        return -1;
    };
    let replacement = match convert_replacement(replacement) {
        Ok(replacement) => replacement,
        Err(_) => return -2,
    };
    let pending = PENDING.get_or_init(|| Mutex::new(Vec::new()));
    let Ok(mut pending) = pending.lock() else {
        return -3;
    };
    pending.push(replacement);
    0
}

pub unsafe fn take_all(_plugin_id: *const c_char) -> Result<Vec<VirtualReplacement>, i32> {
    let pending = PENDING.get_or_init(|| Mutex::new(Vec::new()));
    let Ok(mut pending) = pending.lock() else {
        return Err(-1);
    };
    Ok(std::mem::take(&mut *pending))
}

unsafe fn convert_replacement(
    replacement: &Oppw4VirtualReplacement,
) -> Result<VirtualReplacement, ()> {
    let archive_name = required_string(replacement.archive_name)?;
    let file_name = required_string(replacement.file_name)?;
    let source_path = required_string(replacement.source_path)?;
    let source = match replacement.source_kind {
        Oppw4ReplacementSourceKind::File => ReplacementSource::File(PathBuf::from(source_path)),
        Oppw4ReplacementSourceKind::ZipEntry => ReplacementSource::ZipEntry {
            zip_path: PathBuf::from(source_path),
            entry_name: required_string(replacement.source_entry_name)?,
        },
    };
    let mode = match replacement.mode {
        Oppw4ReplacementMode::Virtual => ReplacementMode::Virtual,
        Oppw4ReplacementMode::Internal => ReplacementMode::Internal,
        Oppw4ReplacementMode::External => ReplacementMode::External,
    };

    Ok(VirtualReplacement {
        archive_name,
        file_name,
        source,
        mode,
        mod_size: flag_option_u64(replacement.has_mod_size, replacement.mod_size),
        hash: replacement.hash,
        rdb_block_offset: usize::try_from(replacement.rdb_block_offset).map_err(|_| ())?,
        original_data_offset: replacement.original_data_offset,
        original_bin_offset: flag_option_u32(
            replacement.has_original_bin_offset,
            replacement.original_bin_offset,
        ),
        original_bin_size: flag_option_u32(
            replacement.has_original_bin_size,
            replacement.original_bin_size,
        ),
        virtual_bin_offset: flag_option_u64(
            replacement.has_virtual_bin_offset,
            replacement.virtual_bin_offset,
        ),
        rdb_tail_offset: flag_option_u64(
            replacement.has_rdb_tail_offset,
            replacement.rdb_tail_offset,
        )
        .map(usize::try_from)
        .transpose()
        .map_err(|_| ())?,
        original_tail: optional_string(replacement.original_tail),
        virtual_prefix: optional_bytes(
            replacement.virtual_prefix.ptr,
            replacement.virtual_prefix.len,
        ),
    })
}

unsafe fn required_string(value: *const c_char) -> Result<String, ()> {
    if value.is_null() {
        return Err(());
    }
    Ok(CStr::from_ptr(value).to_string_lossy().into_owned())
}

unsafe fn optional_string(value: *const c_char) -> Option<String> {
    (!value.is_null()).then(|| CStr::from_ptr(value).to_string_lossy().into_owned())
}

unsafe fn optional_bytes(ptr: *const u8, len: usize) -> Option<Vec<u8>> {
    if ptr.is_null() || len == 0 {
        return None;
    }
    Some(std::slice::from_raw_parts(ptr, len).to_vec())
}

fn flag_option_u32(has_value: i32, value: u32) -> Option<u32> {
    (has_value != 0).then_some(value)
}

fn flag_option_u64(has_value: i32, value: u64) -> Option<u64> {
    (has_value != 0).then_some(value)
}
