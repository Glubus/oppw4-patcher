use std::{
    collections::HashMap,
    ffi::c_void,
    io::SeekFrom,
    mem::size_of,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use oppw4_debug_tools::rdb_io::{
    classify_archive_file_name, hex_preview, is_interesting_create_path, TrackedArchiveKind,
};
use oppw4_rdb::{ReplacementSource, VirtualHandle, VirtualManager, VirtualReplacement};

use crate::{changes, log, win};

type Bool = i32;
type Dword = u32;
type Handle = *mut c_void;
type Lpcwstr = *const u16;
type Lpvoid = *mut c_void;
type Lpdword = *mut Dword;
type LargeInteger = i64;

const GENERIC_READ: Dword = 0x8000_0000;
const INVALID_HANDLE_VALUE: Handle = !0usize as Handle;
const FAKE_HANDLE_MASK: usize = 0xf000_0000_0000_0000;
const FAKE_HANDLE_BITS: usize = 0x1000_0000_0000_0000;
const FAKE_HANDLE_LAW_SLOT5_BITS: usize = 0x2000_0000_0000_0000;
const FILE_TYPE_DISK: Dword = 0x0000_0001;
const FILETIME_TICKS_PER_SECOND: u64 = 10_000_000;
const WINDOWS_TO_UNIX_EPOCH_SECONDS: u64 = 11_644_473_600;
const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;
const IMAGE_ORDINAL_FLAG64: u64 = 0x8000_0000_0000_0000;
const LAW_SLOT5_BIN_READ_PATCH_ENABLED: bool = true;

type CreateFileWFn =
    unsafe extern "system" fn(Lpcwstr, Dword, Dword, Lpvoid, Dword, Dword, Handle) -> Handle;
type ReadFileFn = unsafe extern "system" fn(Handle, Lpvoid, Dword, Lpdword, Lpvoid) -> Bool;
type CloseHandleFn = unsafe extern "system" fn(Handle) -> Bool;
type GetFileSizeExFn = unsafe extern "system" fn(Handle, *mut LargeInteger) -> Bool;
type GetFileTimeFn = unsafe extern "system" fn(Handle, Lpvoid, Lpvoid, Lpvoid) -> Bool;
type GetFileTypeFn = unsafe extern "system" fn(Handle) -> Dword;
type SetFilePointerExFn =
    unsafe extern "system" fn(Handle, LargeInteger, *mut LargeInteger, Dword) -> Bool;

#[repr(C)]
#[derive(Clone, Copy)]
struct FileTime {
    low_date_time: u32,
    high_date_time: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetSystemTimeAsFileTime(system_time_as_file_time: *mut FileTime);
}

static ORIGINALS: OnceLock<OriginalFunctions> = OnceLock::new();
static RUNTIME: OnceLock<Mutex<Option<VirtualManager>>> = OnceLock::new();
static LAW_SLOT5_RUNTIME: OnceLock<Mutex<Option<VirtualManager>>> = OnceLock::new();
static RDB_TRACKER: OnceLock<Mutex<RdbTracker>> = OnceLock::new();
static HASH_FILE_TRACKER: OnceLock<Mutex<HashFileTracker>> = OnceLock::new();
static VIRTUAL_SOURCES: OnceLock<Mutex<HashMap<VirtualSourceKey, ReplacementSource>>> =
    OnceLock::new();
static CREATE_FILE_LOGS: AtomicUsize = AtomicUsize::new(0);
static DATA_HIT_LOGS: AtomicUsize = AtomicUsize::new(0);
static DATA_PATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static INDEX_PATCH_LOGS: AtomicUsize = AtomicUsize::new(0);
static OPEN_VIRTUAL_LOGS: AtomicUsize = AtomicUsize::new(0);
static VIRTUAL_IO_LOGS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum VirtualRuntimeKind {
    Global,
    LawSlot5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct VirtualSourceKey {
    kind: VirtualRuntimeKind,
    raw_handle: u64,
}

#[derive(Clone, Copy)]
struct OriginalFunctions {
    create_file_w: Option<CreateFileWFn>,
    read_file: Option<ReadFileFn>,
    close_handle: Option<CloseHandleFn>,
    get_file_size_ex: Option<GetFileSizeExFn>,
    get_file_time: Option<GetFileTimeFn>,
    get_file_type: Option<GetFileTypeFn>,
    set_file_pointer_ex: Option<SetFilePointerExFn>,
}

impl OriginalFunctions {
    fn empty() -> Self {
        Self {
            create_file_w: None,
            read_file: None,
            close_handle: None,
            get_file_size_ex: None,
            get_file_time: None,
            get_file_type: None,
            set_file_pointer_ex: None,
        }
    }
}

#[repr(C)]
struct ImageImportDescriptor {
    original_first_thunk: u32,
    time_date_stamp: u32,
    forwarder_chain: u32,
    name: u32,
    first_thunk: u32,
}

#[derive(Debug, Default)]
struct RdbTracker {
    handles: HashMap<usize, TrackedRdb>,
    total_logs: usize,
}

#[derive(Debug, Default)]
struct HashFileTracker {
    handles: HashMap<usize, TrackedHashFile>,
    total_logs: usize,
}

#[derive(Debug)]
struct TrackedHashFile {
    path: String,
    read_logs: usize,
}

#[derive(Debug)]
struct TrackedRdb {
    archive_name: String,
    kind: TrackedArchiveKind,
    read_logs: usize,
}

impl RdbTracker {
    fn track_open(&mut self, handle: Handle, file_name: &str) {
        if handle == INVALID_HANDLE_VALUE {
            return;
        }
        let Some((archive_name, kind)) = tracked_archive_name(file_name) else {
            return;
        };
        self.handles.insert(
            handle as usize,
            TrackedRdb {
                archive_name,
                kind,
                read_logs: 0,
            },
        );
        log::write_line(format!(
            "Track {} {}: handle=0x{:x}",
            kind.label(),
            self.handles
                .get(&(handle as usize))
                .map(|tracked| tracked.archive_name.as_str())
                .unwrap_or("?"),
            handle as usize
        ));
    }

    fn untrack(&mut self, handle: Handle) {
        if let Some(tracked) = self.handles.remove(&(handle as usize)) {
            log::write_line(format!(
                "Close {} {}",
                tracked.kind.label(),
                tracked.archive_name
            ));
        }
    }

    fn read_event(&mut self, handle: Handle) -> Option<(TrackedRead, bool)> {
        let tracked = self.handles.get_mut(&(handle as usize))?;
        let should_log = tracked.read_logs < 64 && self.total_logs < 640;
        if should_log {
            tracked.read_logs += 1;
            self.total_logs += 1;
        }
        Some((
            TrackedRead {
                archive_name: tracked.archive_name.clone(),
                kind: tracked.kind,
            },
            should_log,
        ))
    }
}

impl HashFileTracker {
    fn track_open(&mut self, handle: Handle, path: &str) {
        if handle == INVALID_HANDLE_VALUE || !is_hash_data_file_path(path) {
            return;
        }
        self.handles.insert(
            handle as usize,
            TrackedHashFile {
                path: path.to_string(),
                read_logs: 0,
            },
        );
        log::write_line(format!(
            "HASH FILE OPEN path={path} handle=0x{:x} {}",
            handle as usize,
            format_hash_open_context()
        ));
    }

    fn untrack(&mut self, handle: Handle) {
        if let Some(tracked) = self.handles.remove(&(handle as usize)) {
            log::write_line(format!("HASH FILE CLOSE path={}", tracked.path));
        }
    }

    fn read_event(&mut self, handle: Handle) -> Option<(String, bool)> {
        let tracked = self.handles.get_mut(&(handle as usize))?;
        let should_log = tracked.read_logs < 32 && self.total_logs < 256;
        if should_log {
            tracked.read_logs += 1;
            self.total_logs += 1;
        }
        Some((tracked.path.clone(), should_log))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrackedRead {
    archive_name: String,
    kind: TrackedArchiveKind,
}

pub fn publish_replacements(replacements: Vec<VirtualReplacement>) {
    let count = replacements.len();
    let runtime = runtime_cell(VirtualRuntimeKind::Global);
    let Ok(mut guard) = runtime.lock() else {
        log::write_line("virtual runtime lock poisoned");
        return;
    };
    *guard = Some(VirtualManager::new(replacements));
    log::write_line(format!("virtual runtime published: {count} replacements"));
}

pub fn publish_law_slot5_replacements(replacements: Vec<VirtualReplacement>) {
    let count = replacements.len();
    let runtime = runtime_cell(VirtualRuntimeKind::LawSlot5);
    let Ok(mut guard) = runtime.lock() else {
        log::write_line("law slot5 virtual runtime lock poisoned");
        return;
    };
    *guard = Some(VirtualManager::new(replacements));
    log::write_line(format!(
        "law-slot5 virtual runtime published: {count} replacements"
    ));
}

pub fn install_main_module_hooks() {
    let module = win::main_module();
    if module.is_null() {
        log::write_line("hook install skipped: main module not found");
        return;
    }

    let originals = unsafe { install_iat_hooks(module as usize) };
    let installed = count_installed(originals);
    let _ = ORIGINALS.set(originals);
    log::write_line(format!("IAT hooks installed: {installed}"));
    changes::file_job_diag::install();
    changes::law_slot5_unlock::install();
    changes::law_slot5_model_manager::install();
}

unsafe fn install_iat_hooks(module: usize) -> OriginalFunctions {
    let mut originals = OriginalFunctions::empty();
    let Some((import_rva, import_size)) = import_directory(module) else {
        log::write_line("hook install skipped: import directory missing");
        return originals;
    };
    if import_rva == 0 || import_size == 0 {
        log::write_line("hook install skipped: empty import directory");
        return originals;
    }

    let mut descriptor = (module + import_rva as usize) as *const ImageImportDescriptor;
    while (*descriptor).name != 0 {
        patch_import_descriptor(module, &*descriptor, &mut originals);
        descriptor = descriptor.add(1);
    }
    originals
}

unsafe fn import_directory(module: usize) -> Option<(u32, u32)> {
    let dos = module as *const u8;
    if std::slice::from_raw_parts(dos, 2) != b"MZ" {
        return None;
    }
    let nt_offset = *(dos.add(0x3c) as *const u32) as usize;
    let nt = dos.add(nt_offset);
    if std::slice::from_raw_parts(nt, 4) != b"PE\0\0" {
        return None;
    }

    let optional_header = nt.add(24);
    let magic = *(optional_header as *const u16);
    if magic != 0x20b {
        return None;
    }

    let data_directories = optional_header.add(112);
    let import_directory = data_directories.add(IMAGE_DIRECTORY_ENTRY_IMPORT * 8);
    let rva = *(import_directory as *const u32);
    let size = *(import_directory.add(4) as *const u32);
    Some((rva, size))
}

unsafe fn patch_import_descriptor(
    module: usize,
    descriptor: &ImageImportDescriptor,
    originals: &mut OriginalFunctions,
) {
    let lookup_rva = if descriptor.original_first_thunk != 0 {
        descriptor.original_first_thunk
    } else {
        descriptor.first_thunk
    };
    if lookup_rva == 0 || descriptor.first_thunk == 0 {
        return;
    }

    let mut lookup = (module + lookup_rva as usize) as *const u64;
    let mut iat = (module + descriptor.first_thunk as usize) as *mut usize;
    while *lookup != 0 {
        let entry = *lookup;
        if entry & IMAGE_ORDINAL_FLAG64 == 0 {
            let import_by_name = (module + entry as usize) as *const u8;
            let name = c_string_at(import_by_name.add(2));
            patch_import_by_name(name, iat, originals);
        }
        lookup = lookup.add(1);
        iat = iat.add(1);
    }
}

unsafe fn patch_import_by_name(name: &str, iat: *mut usize, originals: &mut OriginalFunctions) {
    match name {
        "CreateFileW" => patch_slot(iat, hooked_create_file_w as usize, |original| {
            originals.create_file_w = Some(std::mem::transmute(original));
        }),
        "ReadFile" => patch_slot(iat, hooked_read_file as usize, |original| {
            originals.read_file = Some(std::mem::transmute(original));
        }),
        "CloseHandle" => patch_slot(iat, hooked_close_handle as usize, |original| {
            originals.close_handle = Some(std::mem::transmute(original));
        }),
        "GetFileSizeEx" => patch_slot(iat, hooked_get_file_size_ex as usize, |original| {
            originals.get_file_size_ex = Some(std::mem::transmute(original));
        }),
        "GetFileTime" => patch_slot(iat, hooked_get_file_time as usize, |original| {
            originals.get_file_time = Some(std::mem::transmute(original));
        }),
        "GetFileType" => patch_slot(iat, hooked_get_file_type as usize, |original| {
            originals.get_file_type = Some(std::mem::transmute(original));
        }),
        "SetFilePointerEx" => patch_slot(iat, hooked_set_file_pointer_ex as usize, |original| {
            originals.set_file_pointer_ex = Some(std::mem::transmute(original));
        }),
        _ => {}
    }
}

unsafe fn patch_slot<F>(slot: *mut usize, replacement: usize, assign_original: F)
where
    F: FnOnce(usize),
{
    if *slot == replacement {
        return;
    }
    let original = *slot;
    let mut old_protect = 0;
    if !win::make_memory_writable(slot.cast(), size_of::<usize>(), &mut old_protect) {
        return;
    }
    *slot = replacement;
    let _ = win::restore_memory_protection(slot.cast(), size_of::<usize>(), old_protect);
    assign_original(original);
}

unsafe fn c_string_at(ptr: *const u8) -> &'static str {
    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    let bytes = std::slice::from_raw_parts(ptr, len);
    std::str::from_utf8_unchecked(bytes)
}

fn count_installed(originals: OriginalFunctions) -> usize {
    [
        originals.create_file_w.is_some(),
        originals.read_file.is_some(),
        originals.close_handle.is_some(),
        originals.get_file_size_ex.is_some(),
        originals.get_file_time.is_some(),
        originals.get_file_type.is_some(),
        originals.set_file_pointer_ex.is_some(),
    ]
    .into_iter()
    .filter(|installed| *installed)
    .count()
}

unsafe extern "system" fn hooked_create_file_w(
    path: Lpcwstr,
    desired_access: Dword,
    share_mode: Dword,
    security_attributes: Lpvoid,
    creation_disposition: Dword,
    flags_and_attributes: Dword,
    template_file: Handle,
) -> Handle {
    let full_path = wide_path_to_string(path);
    if let Some(path) = full_path.as_deref() {
        changes::law_slot5_runtime::try_install("create_file");
        changes::law_slot5_runtime::update_activity_from_path(path);
        log_create_file_candidate(path, desired_access, creation_disposition);
    }
    let file_name = full_path
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .map(|name| name.to_string_lossy().into_owned());
    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.create_file_w)
    else {
        return INVALID_HANDLE_VALUE;
    };
    if desired_access == GENERIC_READ {
        if let Some(path) = full_path.as_deref() {
            if let Some(handle) = open_virtual_fake_handle(path) {
                return handle;
            }
        }
    }

    let alias_path = full_path
        .as_deref()
        .and_then(changes::law_slot5_runtime::slot5_dlc_alias_path);
    let alias_path_wide = alias_path.as_deref().map(wide_null_string);
    let original_path = alias_path_wide
        .as_ref()
        .map(|wide| wide.as_ptr())
        .unwrap_or(path);

    let handle = original(
        original_path,
        desired_access,
        share_mode,
        security_attributes,
        creation_disposition,
        flags_and_attributes,
        template_file,
    );
    if let Some(file_name) = file_name.as_deref() {
        with_rdb_tracker(|tracker| tracker.track_open(handle, file_name));
    }
    if let Some(path) = full_path.as_deref() {
        with_hash_file_tracker(|tracker| tracker.track_open(handle, path));
    }
    handle
}

unsafe extern "system" fn hooked_read_file(
    handle: Handle,
    buffer: Lpvoid,
    bytes_to_read: Dword,
    bytes_read: Lpdword,
    overlapped: Lpvoid,
) -> Bool {
    if let Some((virtual_handle, kind)) = virtual_handle_for_os_handle(handle) {
        return read_virtual_file(
            virtual_handle,
            kind,
            buffer,
            bytes_to_read,
            bytes_read,
            overlapped,
        );
    }

    let Some(original) = ORIGINALS.get().and_then(|originals| originals.read_file) else {
        return 0;
    };
    let tracked = tracked_read(handle, bytes_to_read, overlapped);
    log_hash_file_read(handle, bytes_to_read, overlapped);
    let result = original(handle, buffer, bytes_to_read, bytes_read, overlapped);
    if result != 0 {
        patch_tracked_read(tracked, buffer, bytes_to_read, bytes_read);
    }
    result
}

unsafe extern "system" fn hooked_close_handle(handle: Handle) -> Bool {
    if let Some((virtual_handle, kind)) = fake_to_handle(handle) {
        return close_virtual_file(virtual_handle, kind);
    }

    let Some(original) = ORIGINALS.get().and_then(|originals| originals.close_handle) else {
        return 0;
    };
    with_rdb_tracker(|tracker| tracker.untrack(handle));
    with_hash_file_tracker(|tracker| tracker.untrack(handle));
    original(handle)
}

unsafe extern "system" fn hooked_get_file_size_ex(handle: Handle, size: *mut LargeInteger) -> Bool {
    if let Some((virtual_handle, kind)) = virtual_handle_for_os_handle(handle) {
        return get_virtual_file_size(virtual_handle, kind, size);
    }

    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.get_file_size_ex)
    else {
        return 0;
    };
    original(handle, size)
}

unsafe extern "system" fn hooked_get_file_time(
    handle: Handle,
    creation_time: Lpvoid,
    last_access_time: Lpvoid,
    last_write_time: Lpvoid,
) -> Bool {
    if let Some((virtual_handle, kind)) = virtual_handle_for_os_handle(handle) {
        return unsafe {
            get_virtual_file_time(
                virtual_handle,
                kind,
                creation_time,
                last_access_time,
                last_write_time,
            )
        };
    }

    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.get_file_time)
    else {
        return 0;
    };
    original(handle, creation_time, last_access_time, last_write_time)
}

unsafe extern "system" fn hooked_get_file_type(handle: Handle) -> Dword {
    if let Some((virtual_handle, _kind)) = virtual_handle_for_os_handle(handle) {
        log_virtual_io(format_args!(
            "Virtual TYPE handle=0x{:x} type=disk",
            virtual_handle.as_raw()
        ));
        return FILE_TYPE_DISK;
    }

    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.get_file_type)
    else {
        return 0;
    };
    original(handle)
}

unsafe extern "system" fn hooked_set_file_pointer_ex(
    handle: Handle,
    distance: LargeInteger,
    new_pointer: *mut LargeInteger,
    move_method: Dword,
) -> Bool {
    if let Some((virtual_handle, kind)) = virtual_handle_for_os_handle(handle) {
        return seek_virtual_file(virtual_handle, kind, distance, new_pointer, move_method);
    }

    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.set_file_pointer_ex)
    else {
        return 0;
    };
    original(handle, distance, new_pointer, move_method)
}

fn open_virtual_fake_handle(path: &str) -> Option<Handle> {
    if changes::law_slot5_runtime::slot5_active() {
        if let Some(handle) = open_virtual_fake_handle_with_kind(path, VirtualRuntimeKind::LawSlot5)
        {
            return Some(handle);
        }
    }
    open_virtual_fake_handle_with_kind(path, VirtualRuntimeKind::Global)
}

fn open_virtual_fake_handle_with_kind(path: &str, kind: VirtualRuntimeKind) -> Option<Handle> {
    let (virtual_handle, replacement) = with_manager(kind, |manager| {
        let (virtual_handle, replacement) = open_virtual_handle_for_path(manager, path)?;
        Some((virtual_handle, replacement))
    })
    .flatten()?;
    let handle = returned_virtual_handle(virtual_handle, kind);
    remember_virtual_source(virtual_handle, kind, replacement.source.clone());
    log_open_virtual(path, handle, &replacement);
    Some(handle)
}

fn open_virtual_handle_for_path(
    manager: &mut VirtualManager,
    path: &str,
) -> Option<(VirtualHandle, VirtualReplacement)> {
    manager
        .open_by_path_fragment_with_replacement(
            Path::new(path).file_name()?.to_string_lossy().as_ref(),
        )
        .ok()
        .flatten()
        .or_else(|| {
            manager
                .open_by_path_fragment_with_replacement(path)
                .ok()
                .flatten()
        })
}

unsafe fn read_virtual_file(
    handle: VirtualHandle,
    kind: VirtualRuntimeKind,
    buffer: Lpvoid,
    bytes_to_read: Dword,
    bytes_read: Lpdword,
    overlapped: Lpvoid,
) -> Bool {
    if bytes_to_read == 0 {
        if !bytes_read.is_null() {
            *bytes_read = 0;
        }
        log_virtual_io(format_args!(
            "Virtual READ handle=0x{:x} offset=unchanged request=0x0 read=0x0",
            handle.as_raw()
        ));
        return 1;
    }
    if buffer.is_null() {
        return 0;
    }
    let requested_offset = if overlapped.is_null() {
        None
    } else {
        let offset = read_overlapped_offset(overlapped);
        if offset < 0 {
            return 0;
        }
        Some(offset as u64)
    };
    let buffer = std::slice::from_raw_parts_mut(buffer.cast::<u8>(), bytes_to_read as usize);
    let Some(result) = with_manager(kind, |manager| {
        if let Some(offset) = requested_offset {
            if manager.seek(handle, SeekFrom::Start(offset)).is_err() {
                return None;
            }
        }
        manager.read(handle, buffer).ok()
    }) else {
        return 0;
    };
    let Some(read) = result else {
        return 0;
    };
    if !bytes_read.is_null() {
        *bytes_read = read as Dword;
    }
    let h_event = if overlapped.is_null() {
        0
    } else {
        read_overlapped_event(overlapped)
    };
    let preview_len = read.min(16);
    let preview = hex_preview(&buffer[..preview_len]);
    log_virtual_io(format_args!(
        "Virtual READ handle=0x{:x} offset={} request=0x{:x} read=0x{:x} hEvent=0x{h_event:x} first={preview}",
        handle.as_raw(),
        requested_offset
            .map(|offset| format!("0x{offset:x}"))
            .unwrap_or_else(|| "fp".to_string()),
        bytes_to_read,
        read
    ));
    1
}

fn log_open_virtual(path: &str, returned_handle: Handle, replacement: &VirtualReplacement) {
    let index = OPEN_VIRTUAL_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 80 {
        let prefix_len = replacement
            .virtual_prefix
            .as_ref()
            .map(|prefix| prefix.len())
            .unwrap_or_default();
        log::write_line(format!(
            "Open virtual {path} handle=0x{:x} file={} mode={:?} hash=0x{:08x} prefix=0x{prefix_len:x} mod_size=0x{:x} source={}",
            returned_handle as usize,
            replacement.file_name,
            replacement.mode,
            replacement.hash,
            replacement.mod_size.unwrap_or_default(),
            replacement.source.display_name()
        ));
    } else if index == 80 {
        log::write_line("Open virtual logs suppressed".to_string());
    }
}

fn close_virtual_file(handle: VirtualHandle, kind: VirtualRuntimeKind) -> Bool {
    let closed = with_manager(kind, |manager| manager.close(handle))
        .filter(|closed| *closed)
        .map(|_| 1)
        .unwrap_or(0);
    if closed != 0 {
        forget_virtual_source(handle, kind);
    }
    log_virtual_io(format_args!(
        "Virtual CLOSE handle=0x{:x} closed={closed}",
        handle.as_raw()
    ));
    closed
}

unsafe fn get_virtual_file_size(
    handle: VirtualHandle,
    kind: VirtualRuntimeKind,
    size: *mut LargeInteger,
) -> Bool {
    if size.is_null() {
        return 0;
    }
    let Some(result) = with_manager(kind, |manager| manager.size(handle).ok()) else {
        return 0;
    };
    let Some(file_size) = result else {
        return 0;
    };
    *size = file_size as LargeInteger;
    log_virtual_io(format_args!(
        "Virtual SIZE handle=0x{:x} size=0x{file_size:x}",
        handle.as_raw()
    ));
    1
}

unsafe fn get_virtual_file_time(
    handle: VirtualHandle,
    kind: VirtualRuntimeKind,
    creation_time: Lpvoid,
    last_access_time: Lpvoid,
    last_write_time: Lpvoid,
) -> Bool {
    if let Some((creation, access, write)) = virtual_file_times(handle, kind) {
        write_file_time(creation_time, creation);
        write_file_time(last_access_time, access);
        write_file_time(last_write_time, write);
        log_virtual_io(format_args!(
            "Virtual TIME handle=0x{:x} source=mod",
            handle.as_raw()
        ));
        return 1;
    }

    let mut now = FileTime {
        low_date_time: 0,
        high_date_time: 0,
    };
    GetSystemTimeAsFileTime(&mut now);
    write_file_time(creation_time, now);
    write_file_time(last_access_time, now);
    write_file_time(last_write_time, now);
    log_virtual_io(format_args!(
        "Virtual TIME handle=0x{:x} source=system",
        handle.as_raw()
    ));
    1
}

fn virtual_file_times(
    handle: VirtualHandle,
    kind: VirtualRuntimeKind,
) -> Option<(FileTime, FileTime, FileTime)> {
    let source = virtual_source(handle, kind)?;
    let write = source.modified_time().and_then(system_time_to_file_time)?;
    Some((write, write, write))
}

fn system_time_to_file_time(time: SystemTime) -> Option<FileTime> {
    let duration = time.duration_since(UNIX_EPOCH).ok()?;
    let seconds = duration
        .as_secs()
        .checked_add(WINDOWS_TO_UNIX_EPOCH_SECONDS)?;
    let ticks = seconds
        .checked_mul(FILETIME_TICKS_PER_SECOND)?
        .checked_add((duration.subsec_nanos() / 100) as u64)?;
    Some(FileTime {
        low_date_time: ticks as u32,
        high_date_time: (ticks >> 32) as u32,
    })
}

unsafe fn write_file_time(target: Lpvoid, value: FileTime) {
    if !target.is_null() {
        *target.cast::<FileTime>() = value;
    }
}

unsafe fn seek_virtual_file(
    handle: VirtualHandle,
    kind: VirtualRuntimeKind,
    distance: LargeInteger,
    new_pointer: *mut LargeInteger,
    move_method: Dword,
) -> Bool {
    let position = match move_method {
        0 => SeekFrom::Start(distance.max(0) as u64),
        1 => SeekFrom::Current(distance),
        2 => SeekFrom::End(distance),
        _ => return 0,
    };
    let Some(result) = with_manager(kind, |manager| manager.seek(handle, position).ok()) else {
        return 0;
    };
    let Some(position) = result else {
        return 0;
    };
    if !new_pointer.is_null() {
        *new_pointer = position as LargeInteger;
    }
    log_virtual_io(format_args!(
        "Virtual SEEK handle=0x{:x} method={move_method} distance=0x{distance:x} new=0x{position:x}",
        handle.as_raw()
    ));
    1
}

fn with_manager<T>(
    kind: VirtualRuntimeKind,
    action: impl FnOnce(&mut VirtualManager) -> T,
) -> Option<T> {
    let runtime = runtime_cell(kind);
    let mut guard = runtime.lock().ok()?;
    let manager = guard.as_mut()?;
    Some(action(manager))
}

fn runtime_cell(kind: VirtualRuntimeKind) -> &'static Mutex<Option<VirtualManager>> {
    match kind {
        VirtualRuntimeKind::Global => RUNTIME.get_or_init(|| Mutex::new(None)),
        VirtualRuntimeKind::LawSlot5 => LAW_SLOT5_RUNTIME.get_or_init(|| Mutex::new(None)),
    }
}

fn remember_virtual_source(
    handle: VirtualHandle,
    kind: VirtualRuntimeKind,
    source: ReplacementSource,
) {
    let sources = VIRTUAL_SOURCES.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = sources.lock() {
        guard.insert(virtual_source_key(handle, kind), source);
    }
}

fn forget_virtual_source(handle: VirtualHandle, kind: VirtualRuntimeKind) {
    let Some(sources) = VIRTUAL_SOURCES.get() else {
        return;
    };
    if let Ok(mut guard) = sources.lock() {
        guard.remove(&virtual_source_key(handle, kind));
    }
}

fn virtual_source(handle: VirtualHandle, kind: VirtualRuntimeKind) -> Option<ReplacementSource> {
    let sources = VIRTUAL_SOURCES.get()?;
    let guard = sources.lock().ok()?;
    guard.get(&virtual_source_key(handle, kind)).cloned()
}

fn virtual_source_key(handle: VirtualHandle, kind: VirtualRuntimeKind) -> VirtualSourceKey {
    VirtualSourceKey {
        kind,
        raw_handle: handle.as_raw(),
    }
}

unsafe fn tracked_read(
    handle: Handle,
    bytes_to_read: Dword,
    overlapped: Lpvoid,
) -> Option<(TrackedRead, LargeInteger)> {
    let Some((tracked, should_log)) =
        with_rdb_tracker(|tracker| tracker.read_event(handle)).flatten()
    else {
        return None;
    };
    let offset = if overlapped.is_null() {
        current_file_pointer(handle).unwrap_or(-1)
    } else {
        read_overlapped_offset(overlapped)
    };
    if should_log {
        log::write_line(format!(
            "{} READ {}: offset=0x{offset:x} bytes=0x{bytes_to_read:x} {}",
            tracked.kind.label(),
            tracked.archive_name,
            format_io_context()
        ));
    }
    Some((tracked, offset))
}

unsafe fn log_hash_file_read(handle: Handle, bytes_to_read: Dword, overlapped: Lpvoid) {
    let Some((path, should_log)) =
        with_hash_file_tracker(|tracker| tracker.read_event(handle)).flatten()
    else {
        return;
    };
    if !should_log {
        return;
    }
    let offset = if overlapped.is_null() {
        current_file_pointer(handle).unwrap_or(-1)
    } else {
        read_overlapped_offset(overlapped)
    };
    log::write_line(format!(
        "HASH FILE READ path={path} offset=0x{offset:x} bytes=0x{bytes_to_read:x} {}",
        format_io_context()
    ));
}

unsafe fn patch_tracked_read(
    tracked: Option<(TrackedRead, LargeInteger)>,
    buffer: Lpvoid,
    bytes_to_read: Dword,
    bytes_read: Lpdword,
) {
    let Some((tracked, offset)) = tracked else {
        return;
    };
    if offset < 0 || buffer.is_null() {
        return;
    }
    let actual_read = if bytes_read.is_null() {
        bytes_to_read as usize
    } else {
        *bytes_read as usize
    };
    if actual_read == 0 {
        return;
    }
    let buffer = std::slice::from_raw_parts_mut(buffer.cast::<u8>(), actual_read);
    match tracked.kind {
        TrackedArchiveKind::Index => {
            patch_index_external_flags(&tracked.archive_name, offset as u64, buffer);
        }
        TrackedArchiveKind::Data => {
            patch_data_read(&tracked.archive_name, offset as u64, buffer);
            log_data_read_hits(&tracked.archive_name, offset as u64, buffer.len());
        }
    }
}

fn patch_data_read(archive_name: &str, read_offset: u64, buffer: &mut [u8]) {
    let global_patched = with_manager(VirtualRuntimeKind::Global, |manager| {
        manager.patch_archive_read_if(archive_name, read_offset, buffer, |replacement| {
            allow_global_replacement(replacement)
        })
    })
    .and_then(Result::ok)
    .unwrap_or_default();
    let slot5_patched = if allow_law_slot5_data_patch(archive_name) {
        with_manager(VirtualRuntimeKind::LawSlot5, |manager| {
            manager.patch_archive_read(archive_name, read_offset, buffer)
        })
        .and_then(Result::ok)
        .unwrap_or_default()
    } else {
        0
    };
    let patched = global_patched + slot5_patched;
    if patched == 0 || DATA_PATCH_LOGS.load(Ordering::Relaxed) >= 80 {
        return;
    }
    let index = DATA_PATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 80 {
        log::write_line(format!(
            "RDB BIN PATCH {archive_name}: read=0x{read_offset:x}+0x{:x} bytes={patched} law_slot5_active={} mode={}",
            buffer.len(),
            changes::law_slot5_runtime::slot5_active(),
            if slot5_patched > 0 {
                "law-slot5-virtual-offset"
            } else {
                "global-virtual-offset"
            }
        ));
    }
}

fn patch_index_external_flags(archive_name: &str, read_offset: u64, buffer: &mut [u8]) {
    let global_patched = with_manager(VirtualRuntimeKind::Global, |manager| {
        manager.patch_archive_index_external_flags_if(
            archive_name,
            read_offset,
            buffer,
            |replacement| allow_global_replacement(replacement),
        )
    })
    .unwrap_or_default();
    let slot5_patched = if allow_law_slot5_index_patch(archive_name) {
        with_manager(VirtualRuntimeKind::LawSlot5, |manager| {
            manager.patch_archive_index_external_flags(archive_name, read_offset, buffer)
        })
        .unwrap_or_default()
    } else {
        0
    };
    let patched = global_patched + slot5_patched;
    if patched == 0 || INDEX_PATCH_LOGS.load(Ordering::Relaxed) >= 80 {
        return;
    }
    let index = INDEX_PATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 80 {
        log::write_line(format!(
            "RDB INDEX EXTERNAL {archive_name}: read=0x{read_offset:x}+0x{:x} fields={patched} law_slot5_active={} mode={}",
            buffer.len(),
            changes::law_slot5_runtime::slot5_active(),
            if slot5_patched > 0 {
                "law-slot5-virtual-offset"
            } else {
                "global-virtual-offset"
            }
        ));
    }
}

fn allow_global_replacement(replacement: &VirtualReplacement) -> bool {
    !changes::law_slot5_assets::is_reserved_private_asset_name(&replacement.file_name)
}

fn allow_law_slot5_data_patch(archive_name: &str) -> bool {
    LAW_SLOT5_BIN_READ_PATCH_ENABLED
        && changes::law_slot5_runtime::slot5_active()
        && is_law_slot5_model_archive(archive_name)
}

fn allow_law_slot5_index_patch(archive_name: &str) -> bool {
    LAW_SLOT5_BIN_READ_PATCH_ENABLED && is_law_slot5_model_archive(archive_name)
}

fn is_law_slot5_model_archive(archive_name: &str) -> bool {
    matches!(archive_name, "CharacterEditor" | "MaterialEditor")
}

fn log_create_file_candidate(path: &str, desired_access: Dword, creation_disposition: Dword) {
    if CREATE_FILE_LOGS.load(Ordering::Relaxed) >= 320 {
        return;
    }
    if !is_interesting_create_path(path) {
        return;
    }
    let index = CREATE_FILE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 320 {
        log::write_line(format!(
            "CreateFileW path={path} access=0x{desired_access:x} disposition=0x{creation_disposition:x} {}",
            if is_hash_data_file_path(path) {
                format_io_context()
            } else {
                String::new()
            }
        ));
    }
}

fn log_data_read_hits(archive_name: &str, read_offset: u64, read_len: usize) {
    if DATA_HIT_LOGS.load(Ordering::Relaxed) >= 240 {
        return;
    }
    let hits = with_manager(VirtualRuntimeKind::Global, |manager| {
        manager
            .data_read_hits(archive_name, read_offset, read_len)
            .into_iter()
            .map(|replacement| {
                (
                    replacement.file_name.clone(),
                    replacement.hash,
                    replacement.original_bin_offset.unwrap_or_default(),
                    replacement.mod_size.unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    if hits.is_empty() {
        return;
    }
    for (file_name, hash, original_offset, mod_size) in hits {
        let index = DATA_HIT_LOGS.fetch_add(1, Ordering::Relaxed);
        if index >= 240 {
            return;
        }
        log::write_line(format!(
            "RDB BIN HIT {archive_name}: read=0x{read_offset:x}+0x{read_len:x} file={file_name} hash=0x{hash:08x} bin_offset=0x{original_offset:x} mod_size=0x{mod_size:x}"
        ));
    }
}

unsafe fn read_overlapped_offset(overlapped: Lpvoid) -> LargeInteger {
    let bytes = overlapped as *const u8;
    *(bytes.add(16) as *const LargeInteger)
}

unsafe fn read_overlapped_event(overlapped: Lpvoid) -> usize {
    let bytes = overlapped as *const u8;
    *(bytes.add(24) as *const usize)
}

unsafe fn current_file_pointer(handle: Handle) -> Option<LargeInteger> {
    let original = ORIGINALS
        .get()
        .and_then(|originals| originals.set_file_pointer_ex)?;
    let mut position = 0;
    if original(handle, 0, &mut position, 1) == 0 {
        return None;
    }
    Some(position)
}

fn with_rdb_tracker<T>(action: impl FnOnce(&mut RdbTracker) -> T) -> Option<T> {
    let tracker = RDB_TRACKER.get_or_init(|| Mutex::new(RdbTracker::default()));
    let mut guard = tracker.lock().ok()?;
    Some(action(&mut guard))
}

fn with_hash_file_tracker<T>(action: impl FnOnce(&mut HashFileTracker) -> T) -> Option<T> {
    let tracker = HASH_FILE_TRACKER.get_or_init(|| Mutex::new(HashFileTracker::default()));
    let mut guard = tracker.lock().ok()?;
    Some(action(&mut guard))
}

fn is_hash_data_file_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    (lower.contains("\\data\\0x") || lower.contains("/data/0x")) && lower.ends_with(".file")
}

fn format_game_callers() -> String {
    let module = win::main_module() as usize;
    if module == 0 {
        return "callers=none".to_string();
    }

    let callers = win::capture_stack_backtrace(2, 12)
        .into_iter()
        .filter_map(|address| {
            let rva = address.checked_sub(module)?;
            (rva < 0x0400_0000).then_some(format!("game+0x{rva:x}"))
        })
        .take(6)
        .collect::<Vec<_>>();
    if callers.is_empty() {
        "callers=none".to_string()
    } else {
        format!("callers={}", callers.join(","))
    }
}

fn format_io_context() -> String {
    let callers = format_game_callers();
    match changes::file_job_diag::current_job_summary() {
        Some(job) => format!("{callers} {job}"),
        None => callers,
    }
}

fn format_hash_open_context() -> String {
    let callers = format_game_callers();
    match changes::file_job_diag::current_hash_open_summary() {
        Some(job) => format!("{callers} {job}"),
        None => callers,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn handle_to_fake(handle: VirtualHandle, kind: VirtualRuntimeKind) -> Handle {
    let bits = match kind {
        VirtualRuntimeKind::Global => FAKE_HANDLE_BITS,
        VirtualRuntimeKind::LawSlot5 => FAKE_HANDLE_LAW_SLOT5_BITS,
    };
    (bits | handle.as_raw() as usize) as Handle
}

fn returned_virtual_handle(handle: VirtualHandle, kind: VirtualRuntimeKind) -> Handle {
    handle_to_fake(handle, kind)
}

fn fake_to_handle(handle: Handle) -> Option<(VirtualHandle, VirtualRuntimeKind)> {
    let raw = handle as usize;
    let kind = match raw & FAKE_HANDLE_MASK {
        FAKE_HANDLE_BITS => VirtualRuntimeKind::Global,
        FAKE_HANDLE_LAW_SLOT5_BITS => VirtualRuntimeKind::LawSlot5,
        _ => return None,
    };
    Some((
        VirtualHandle::from_raw((raw & !FAKE_HANDLE_MASK) as u64),
        kind,
    ))
}

fn virtual_handle_for_os_handle(handle: Handle) -> Option<(VirtualHandle, VirtualRuntimeKind)> {
    fake_to_handle(handle)
}

fn wide_null_string(path: &str) -> Vec<u16> {
    path.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_path_to_string(path: Lpcwstr) -> Option<String> {
    if path.is_null() {
        return None;
    }
    let mut len = 0;
    unsafe {
        while *path.add(len) != 0 {
            len += 1;
        }
        Some(String::from_utf16_lossy(std::slice::from_raw_parts(
            path, len,
        )))
    }
}

fn log_virtual_io(args: std::fmt::Arguments<'_>) {
    let index = VIRTUAL_IO_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 512 {
        log::write_line(args.to_string());
    } else if index == 512 {
        log::write_line("Virtual IO logs suppressed".to_string());
    }
}

fn tracked_archive_name(file_name: &str) -> Option<(String, TrackedArchiveKind)> {
    let tracked = classify_archive_file_name(file_name)?;
    Some((tracked.archive_name, tracked.kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_handle_round_trip_preserves_id() {
        let handle = VirtualHandle::from_raw(42);

        let round_trip = fake_to_handle(handle_to_fake(handle, VirtualRuntimeKind::Global));

        assert_eq!(
            round_trip.map(|(handle, kind)| (handle.as_raw(), kind)),
            Some((42, VirtualRuntimeKind::Global))
        );
    }

    #[test]
    fn law_slot5_fake_handle_round_trip_preserves_runtime() {
        let handle = VirtualHandle::from_raw(43);

        let round_trip = fake_to_handle(handle_to_fake(handle, VirtualRuntimeKind::LawSlot5));

        assert_eq!(
            round_trip.map(|(handle, kind)| (handle.as_raw(), kind)),
            Some((43, VirtualRuntimeKind::LawSlot5))
        );
    }

    #[test]
    fn normal_handles_are_not_virtual() {
        assert_eq!(fake_to_handle(std::ptr::null_mut()), None);
        assert_eq!(fake_to_handle(INVALID_HANDLE_VALUE), None);
    }

    #[test]
    fn virtual_open_returns_fake_handle_value() {
        let virtual_handle = VirtualHandle::from_raw(0x21);

        let returned = returned_virtual_handle(virtual_handle, VirtualRuntimeKind::Global);

        assert_eq!(
            fake_to_handle(returned).map(|(handle, kind)| (handle.as_raw(), kind)),
            Some((0x21, VirtualRuntimeKind::Global))
        );
    }

    #[test]
    fn extracts_file_name_from_wide_path() {
        let mut path: Vec<u16> = "D:\\Game\\OPPW4_PATCHER\\CharacterEditor\\MDLC038_Zoro_Wa.g1m"
            .encode_utf16()
            .collect();
        path.push(0);

        let file_name = wide_path_to_string(path.as_ptr()).and_then(|text| {
            Path::new(&text)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        });

        assert_eq!(file_name.as_deref(), Some("MDLC038_Zoro_Wa.g1m"));
    }

    #[test]
    fn tracks_rdb_and_rdb_bin_file_names() {
        assert_eq!(
            tracked_archive_name("CharacterEditor.rdb"),
            Some(("CharacterEditor".to_string(), TrackedArchiveKind::Index))
        );
        assert_eq!(
            tracked_archive_name("MaterialEditor.rdb.bin"),
            Some(("MaterialEditor".to_string(), TrackedArchiveKind::Data))
        );
        assert_eq!(
            tracked_archive_name("ScreenLayout.rdb.bin10"),
            Some(("ScreenLayout".to_string(), TrackedArchiveKind::Data))
        );
        assert_eq!(tracked_archive_name("not-an-archive.bin"), None);
    }

    #[test]
    fn law_slot5_rdb_patch_is_limited_to_model_archives() {
        assert!(is_law_slot5_model_archive("CharacterEditor"));
        assert!(is_law_slot5_model_archive("MaterialEditor"));
        assert!(!is_law_slot5_model_archive("ScreenLayout"));
        assert!(!is_law_slot5_model_archive("RRPreview"));
    }
}
