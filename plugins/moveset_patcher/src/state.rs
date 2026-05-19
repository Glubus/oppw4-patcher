use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use crate::{constants::LINKDATA_A_NAME, hex, linkdata};

static STATE: OnceLock<Mutex<ProviderState>> = OnceLock::new();

pub(crate) fn initialize(game_root: PathBuf, mods_root: PathBuf) -> Result<(), String> {
    let base_path = game_root.join("LINKDATA").join("CMN").join(LINKDATA_A_NAME);
    let mut state = ProviderState::new(base_path);
    state.load_legacy_entry_patches(&mods_root)?;
    STATE
        .set(Mutex::new(state))
        .map_err(|_| "moveset state already initialized".to_string())
}

pub(crate) fn with_mut<T>(action: impl FnOnce(&mut ProviderState) -> T) -> Option<T> {
    let state = STATE.get()?;
    let mut guard = state.lock().ok()?;
    Some(action(&mut guard))
}

pub(crate) struct ProviderState {
    base_path: PathBuf,
    edits: BTreeMap<usize, Vec<u8>>,
    file: Option<VirtualFile>,
}

impl ProviderState {
    fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            edits: BTreeMap::new(),
            file: None,
        }
    }

    pub(crate) fn set_entry_patch(&mut self, entry: usize, payload: Vec<u8>) {
        self.edits.insert(entry, payload);
        self.file = None;
    }

    pub(crate) fn edit_count(&self) -> usize {
        self.edits.len()
    }

    pub(crate) fn open(&mut self) -> Result<Option<u64>, String> {
        if self.edits.is_empty() {
            return Ok(None);
        }
        self.ensure_file()?;
        Ok(self.file.as_mut().map(VirtualFile::open))
    }

    pub(crate) fn read(
        &mut self,
        handle: u64,
        buffer: *mut u8,
        bytes_to_read: u32,
        requested_offset: i64,
        out_bytes_read: *mut u32,
    ) -> i32 {
        let Some(file) = self.file.as_mut() else {
            return 0;
        };
        file.read(
            handle,
            buffer,
            bytes_to_read,
            requested_offset,
            out_bytes_read,
        )
    }

    pub(crate) fn close(&mut self, handle: u64) -> i32 {
        self.file
            .as_mut()
            .and_then(|file| file.positions.remove(&handle))
            .map(|_| 1)
            .unwrap_or(0)
    }

    pub(crate) fn size(&mut self, out_size: *mut u64) -> i32 {
        let Some(file) = self.file.as_ref() else {
            return 0;
        };
        unsafe {
            *out_size = file.bytes.len() as u64;
        }
        1
    }

    pub(crate) fn seek(
        &mut self,
        handle: u64,
        distance: i64,
        move_method: u32,
        out_position: *mut u64,
    ) -> i32 {
        let Some(file) = self.file.as_mut() else {
            return 0;
        };
        file.seek(handle, distance, move_method, out_position)
    }

    fn ensure_file(&mut self) -> Result<(), String> {
        if self.file.is_some() {
            return Ok(());
        }
        let base = fs::read(&self.base_path).map_err(|error| {
            format!(
                "base read failed path={} error={error}",
                self.base_path.display()
            )
        })?;
        let patched = linkdata::rebuild_raw_with_edits(&base, &self.edits)?;
        self.file = Some(VirtualFile::new(patched));
        Ok(())
    }

    fn load_legacy_entry_patches(&mut self, mods_root: &Path) -> Result<(), String> {
        let patch_root = mods_root.join("LINKDATA_A");
        let Ok(entries) = fs::read_dir(&patch_root) else {
            return Ok(());
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(index) = patch_entry_index(&path) else {
                continue;
            };
            self.set_entry_patch(index, hex::read_payload(&path)?);
        }
        Ok(())
    }
}

fn patch_entry_index(path: &Path) -> Option<usize> {
    let stem = path.file_stem()?.to_string_lossy();
    let raw = stem
        .strip_prefix("entry_")
        .or_else(|| stem.strip_prefix("entry-"))
        .unwrap_or(&stem);
    raw.parse::<usize>().ok()
}

pub(crate) struct VirtualFile {
    bytes: Vec<u8>,
    next_handle: u64,
    positions: BTreeMap<u64, usize>,
}

impl VirtualFile {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            next_handle: 1,
            positions: BTreeMap::new(),
        }
    }

    fn open(&mut self) -> u64 {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.positions.insert(handle, 0);
        handle
    }

    fn read(
        &mut self,
        handle: u64,
        buffer: *mut u8,
        bytes_to_read: u32,
        requested_offset: i64,
        out_bytes_read: *mut u32,
    ) -> i32 {
        if buffer.is_null() {
            return -1;
        }
        let Some(position) = self.positions.get_mut(&handle) else {
            return 0;
        };
        if requested_offset >= 0 {
            *position = requested_offset as usize;
        }
        let start = (*position).min(self.bytes.len());
        let end = (start + bytes_to_read as usize).min(self.bytes.len());
        let read = end.saturating_sub(start);
        unsafe {
            std::ptr::copy_nonoverlapping(self.bytes[start..end].as_ptr(), buffer, read);
            if !out_bytes_read.is_null() {
                *out_bytes_read = read as u32;
            }
        }
        *position = end;
        1
    }

    fn seek(
        &mut self,
        handle: u64,
        distance: i64,
        move_method: u32,
        out_position: *mut u64,
    ) -> i32 {
        let Some(position) = self.positions.get_mut(&handle) else {
            return 0;
        };
        let base = match move_method {
            0 => 0i64,
            1 => *position as i64,
            2 => self.bytes.len() as i64,
            _ => return 0,
        };
        let new_position = base.saturating_add(distance);
        if new_position < 0 {
            return 0;
        }
        *position = new_position as usize;
        unsafe {
            if !out_position.is_null() {
                *out_position = *position as u64;
            }
        }
        1
    }
}
