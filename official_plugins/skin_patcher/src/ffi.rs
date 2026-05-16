use std::{ffi::CString, ptr};

use oppw4_plugin_api::{
    cstring_lossy, Oppw4ByteSlice, Oppw4PluginApi, Oppw4ReplacementMode,
    Oppw4ReplacementSourceKind, Oppw4VirtualReplacement,
};
use oppw4_rdb::{ReplacementMode, ReplacementSource, VirtualReplacement};

pub fn register_replacements(
    api: &Oppw4PluginApi,
    plugin_id: &CString,
    replacements: &[VirtualReplacement],
) -> i32 {
    let clear = api.clear_virtual_replacements(plugin_id);
    if clear != 0 {
        return clear;
    }

    for replacement in replacements {
        let owned = OwnedReplacement::new(plugin_id, replacement);
        let result = api.register_virtual_replacement(&owned.raw);
        if result != 0 {
            return result;
        }
    }

    api.commit_virtual_replacements(plugin_id)
}

struct OwnedReplacement {
    raw: Oppw4VirtualReplacement,
    _archive_name: CString,
    _file_name: CString,
    _source_path: CString,
    _source_entry_name: Option<CString>,
    _original_tail: Option<CString>,
}

impl OwnedReplacement {
    fn new(plugin_id: &CString, replacement: &VirtualReplacement) -> Self {
        let archive_name = cstring_lossy(&replacement.archive_name);
        let file_name = cstring_lossy(&replacement.file_name);
        let (source_kind, source_path, source_entry_name) = source_parts(&replacement.source);
        let original_tail = replacement.original_tail.as_ref().map(cstring_lossy);
        let virtual_prefix = replacement
            .virtual_prefix
            .as_ref()
            .map(|bytes| Oppw4ByteSlice {
                ptr: bytes.as_ptr(),
                len: bytes.len(),
            })
            .unwrap_or(Oppw4ByteSlice {
                ptr: ptr::null(),
                len: 0,
            });

        let raw = Oppw4VirtualReplacement {
            plugin_id: plugin_id.as_ptr(),
            archive_name: archive_name.as_ptr(),
            file_name: file_name.as_ptr(),
            source_kind,
            source_path: source_path.as_ptr(),
            source_entry_name: source_entry_name
                .as_ref()
                .map(|value| value.as_ptr())
                .unwrap_or(ptr::null()),
            mode: mode(replacement.mode),
            has_mod_size: replacement.mod_size.is_some() as i32,
            mod_size: replacement.mod_size.unwrap_or_default(),
            hash: replacement.hash,
            rdb_block_offset: replacement.rdb_block_offset as u64,
            original_data_offset: replacement.original_data_offset,
            has_original_bin_offset: replacement.original_bin_offset.is_some() as i32,
            original_bin_offset: replacement.original_bin_offset.unwrap_or_default(),
            has_original_bin_size: replacement.original_bin_size.is_some() as i32,
            original_bin_size: replacement.original_bin_size.unwrap_or_default(),
            has_virtual_bin_offset: replacement.virtual_bin_offset.is_some() as i32,
            virtual_bin_offset: replacement.virtual_bin_offset.unwrap_or_default(),
            has_rdb_tail_offset: replacement.rdb_tail_offset.is_some() as i32,
            rdb_tail_offset: replacement.rdb_tail_offset.unwrap_or_default() as u64,
            original_tail: original_tail
                .as_ref()
                .map(|value| value.as_ptr())
                .unwrap_or(ptr::null()),
            virtual_prefix,
        };

        Self {
            raw,
            _archive_name: archive_name,
            _file_name: file_name,
            _source_path: source_path,
            _source_entry_name: source_entry_name,
            _original_tail: original_tail,
        }
    }
}

fn source_parts(
    source: &ReplacementSource,
) -> (Oppw4ReplacementSourceKind, CString, Option<CString>) {
    match source {
        ReplacementSource::File(path) => (
            Oppw4ReplacementSourceKind::File,
            cstring_lossy(path.to_string_lossy()),
            None,
        ),
        ReplacementSource::ZipEntry {
            zip_path,
            entry_name,
        } => (
            Oppw4ReplacementSourceKind::ZipEntry,
            cstring_lossy(zip_path.to_string_lossy()),
            Some(cstring_lossy(entry_name)),
        ),
    }
}

fn mode(mode: ReplacementMode) -> Oppw4ReplacementMode {
    match mode {
        ReplacementMode::Virtual => Oppw4ReplacementMode::Virtual,
        ReplacementMode::Internal => Oppw4ReplacementMode::Internal,
        ReplacementMode::External => Oppw4ReplacementMode::External,
    }
}
