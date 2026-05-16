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
        log_create_file_candidate(path, desired_access, creation_disposition);
        update_law_custom_slot_activity_from_path(path);
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
    if let Some(path) = full_path.as_deref() {
        maybe_dump_costume_table(path, "pre_open");
    }

    let alias_path = if DUPLICATE_LAW_VARIANT_SLOT_ENABLED.load(Ordering::Relaxed) {
        full_path
            .as_deref()
            .and_then(law_extra_slot_probe_alias_path)
    } else {
        None
    };
    let alias_wide = alias_path.as_ref().map(|path| wide_null_string(path));
    if let (Some(requested), Some(alias)) = (full_path.as_deref(), alias_path.as_deref()) {
        log::write_line(format!(
            "Law extra slot DLC alias requested={requested} alias={alias}"
        ));
    }
    let open_path = alias_wide
        .as_ref()
        .map(|path| path.as_ptr())
        .unwrap_or(path);

    let handle = original(
        open_path,
        desired_access,
        share_mode,
        security_attributes,
        creation_disposition,
        flags_and_attributes,
        template_file,
    );
    let last_error = if handle == INVALID_HANDLE_VALUE {
        unsafe { GetLastError() }
    } else {
        0
    };
    if let Some(path) = full_path.as_deref() {
        maybe_dump_costume_table(path, "post_open");
        log_dlc_stack_result(
            path,
            desired_access,
            creation_disposition,
            handle,
            last_error,
        );
    }
    if let Some(file_name) = file_name.as_deref() {
        with_rdb_tracker(|tracker| tracker.track_open(handle, file_name));
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
    if let Some((virtual_handle, runtime_kind)) = virtual_handle_for_os_handle(handle) {
        return read_virtual_file(
            virtual_handle,
            runtime_kind,
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
    let result = original(handle, buffer, bytes_to_read, bytes_read, overlapped);
    if result != 0 {
        patch_tracked_read(tracked, buffer, bytes_to_read, bytes_read);
    }
    result
}

unsafe extern "system" fn hooked_close_handle(handle: Handle) -> Bool {
    if let Some((virtual_handle, runtime_kind)) = fake_virtual_handle(handle) {
        return close_virtual_file(virtual_handle, runtime_kind);
    }

    let Some(original) = ORIGINALS.get().and_then(|originals| originals.close_handle) else {
        return 0;
    };
    with_rdb_tracker(|tracker| tracker.untrack(handle));
    original(handle)
}

unsafe extern "system" fn hooked_get_file_size_ex(handle: Handle, size: *mut LargeInteger) -> Bool {
    if let Some((virtual_handle, runtime_kind)) = virtual_handle_for_os_handle(handle) {
        return get_virtual_file_size(virtual_handle, runtime_kind, size);
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
    if let Some((virtual_handle, runtime_kind)) = virtual_handle_for_os_handle(handle) {
        return unsafe {
            get_virtual_file_time(
                virtual_handle,
                runtime_kind,
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
    if let Some((virtual_handle, _runtime_kind)) = virtual_handle_for_os_handle(handle) {
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
    if let Some((virtual_handle, runtime_kind)) = virtual_handle_for_os_handle(handle) {
        return seek_virtual_file(
            virtual_handle,
            runtime_kind,
            distance,
            new_pointer,
            move_method,
        );
    }

    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.set_file_pointer_ex)
    else {
        return 0;
    };
    original(handle, distance, new_pointer, move_method)
}

unsafe extern "system" fn hooked_get_file_attributes_w(path: Lpcwstr) -> Dword {
    let full_path = wide_path_to_string(path);
    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.get_file_attributes_w)
    else {
        return u32::MAX;
    };
    let attributes = original(path);
    if let Some(path) = full_path.as_deref() {
        let result = if attributes == u32::MAX {
            format!("missing error={}", GetLastError())
        } else {
            format!("ok attrs=0x{attributes:x}")
        };
        log_dlc_discovery_event("GetFileAttributesW", path, result, None);
    }
    attributes
}

unsafe extern "system" fn hooked_get_file_attributes_ex_w(
    path: Lpcwstr,
    info_level: i32,
    file_information: Lpvoid,
) -> Bool {
    let full_path = wide_path_to_string(path);
    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.get_file_attributes_ex_w)
    else {
        return 0;
    };
    let result = original(path, info_level, file_information);
    if let Some(path) = full_path.as_deref() {
        let summary = if result == 0 {
            format!("missing error={}", GetLastError())
        } else {
            "ok".to_string()
        };
        log_dlc_discovery_event("GetFileAttributesExW", path, summary, None);
    }
    result
}

unsafe extern "system" fn hooked_find_first_file_w(
    path: Lpcwstr,
    find_file_data: *mut Win32FindDataW,
) -> Handle {
    let full_path = wide_path_to_string(path);
    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.find_first_file_w)
    else {
        return INVALID_HANDLE_VALUE;
    };
    let handle = original(path, find_file_data);
    if let Some(path) = full_path.as_deref() {
        log_find_first_result("FindFirstFileW", path, handle, find_file_data.cast());
    }
    handle
}

unsafe extern "system" fn hooked_find_first_file_ex_w(
    path: Lpcwstr,
    info_level: i32,
    find_file_data: Lpvoid,
    search_op: i32,
    search_filter: Lpvoid,
    additional_flags: Dword,
) -> Handle {
    let full_path = wide_path_to_string(path);
    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.find_first_file_ex_w)
    else {
        return INVALID_HANDLE_VALUE;
    };
    let handle = original(
        path,
        info_level,
        find_file_data,
        search_op,
        search_filter,
        additional_flags,
    );
    if let Some(path) = full_path.as_deref() {
        log_find_first_result("FindFirstFileExW", path, handle, find_file_data);
    }
    handle
}

unsafe extern "system" fn hooked_find_next_file_w(
    handle: Handle,
    find_file_data: *mut Win32FindDataW,
) -> Bool {
    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.find_next_file_w)
    else {
        return 0;
    };
    let result = original(handle, find_file_data);
    if let Some(pattern) = find_pattern(handle) {
        let summary = if result == 0 {
            format!("done error={}", GetLastError())
        } else {
            "ok".to_string()
        };
        let found_name = (result != 0)
            .then(|| find_data_name(find_file_data))
            .flatten();
        log_dlc_discovery_event("FindNextFileW", &pattern, summary, found_name.as_deref());
    }
    result
}

unsafe extern "system" fn hooked_find_close(handle: Handle) -> Bool {
    forget_find_pattern(handle);
    let Some(original) = ORIGINALS.get().and_then(|originals| originals.find_close) else {
        return 0;
    };
    original(handle)
}

unsafe extern "system" fn hooked_get_proc_address(
    module: Handle,
    proc_name: *const c_char,
) -> *mut c_void {
    let Some(original) = ORIGINALS
        .get()
        .and_then(|originals| originals.get_proc_address)
    else {
        return std::ptr::null_mut();
    };
    let proc = original(module, proc_name);
    let Some(name) = narrow_proc_name(proc_name) else {
        return proc;
    };
    let Some(symbol) = steam_apps_probe_symbol(name) else {
        return proc;
    };
    let Some(replacement) = steam_apps_symbol_replacement(symbol) else {
        return proc;
    };
    if proc.is_null() {
        log_steam_apps_probe_event(format_args!(
            "GetProcAddress SteamApps symbol={name} result=null"
        ));
        return proc;
    }

    remember_dynamic_steam_apps_symbol(symbol, proc);
    log_steam_apps_probe_event(format_args!(
        "GetProcAddress SteamApps symbol={name} original=0x{:x} wrapper=0x{:x}",
        proc as usize, replacement as usize
    ));
    replacement
}

unsafe extern "system" fn hooked_steam_api_steam_apps_v006() -> *mut c_void {
    let Some(original) = steam_apps_interface_original("v006") else {
        return std::ptr::null_mut();
    };
    let apps = original();
    install_steam_apps_vtable_probe(apps, "v006");
    apps
}

unsafe extern "system" fn hooked_steam_api_steam_apps_v007() -> *mut c_void {
    let Some(original) = steam_apps_interface_original("v007") else {
        return std::ptr::null_mut();
    };
    let apps = original();
    install_steam_apps_vtable_probe(apps, "v007");
    apps
}

unsafe extern "system" fn hooked_steam_api_steam_apps_v008() -> *mut c_void {
    let Some(original) = steam_apps_interface_original("v008") else {
        return std::ptr::null_mut();
    };
    let apps = original();
    install_steam_apps_vtable_probe(apps, "v008");
    apps
}

unsafe extern "system" fn hooked_steam_apps_flat_b_is_subscribed_app(
    this: *mut c_void,
    app_id: u32,
) -> bool {
    let Some(original) = steam_apps_flat_original(SteamAppsBoolMethod::IsSubscribedApp) else {
        return false;
    };
    let result = original(this, app_id);
    log_steam_apps_check(
        "flat",
        SteamAppsBoolMethod::IsSubscribedApp,
        this,
        app_id,
        result,
    );
    result
}

unsafe extern "system" fn hooked_steam_apps_flat_b_is_dlc_installed(
    this: *mut c_void,
    app_id: u32,
) -> bool {
    let Some(original) = steam_apps_flat_original(SteamAppsBoolMethod::IsDlcInstalled) else {
        return false;
    };
    let result = original(this, app_id);
    log_steam_apps_check(
        "flat",
        SteamAppsBoolMethod::IsDlcInstalled,
        this,
        app_id,
        result,
    );
    result
}

unsafe extern "system" fn hooked_steam_apps_vtable_b_is_subscribed_app(
    this: *mut c_void,
    app_id: u32,
) -> bool {
    let Some(original) = steam_apps_vtable_original(SteamAppsBoolMethod::IsSubscribedApp) else {
        return false;
    };
    let result = original(this, app_id);
    log_steam_apps_check(
        "vtable",
        SteamAppsBoolMethod::IsSubscribedApp,
        this,
        app_id,
        result,
    );
    result
}

unsafe extern "system" fn hooked_steam_apps_vtable_b_is_dlc_installed(
    this: *mut c_void,
    app_id: u32,
) -> bool {
    let Some(original) = steam_apps_vtable_original(SteamAppsBoolMethod::IsDlcInstalled) else {
        return false;
    };
    let result = original(this, app_id);
    log_steam_apps_check(
        "vtable",
        SteamAppsBoolMethod::IsDlcInstalled,
        this,
        app_id,
        result,
    );
    result
}

unsafe fn narrow_proc_name(proc_name: *const c_char) -> Option<&'static str> {
    if proc_name.is_null() || (proc_name as usize) <= u16::MAX as usize {
        return None;
    }
    CStr::from_ptr(proc_name).to_str().ok()
}

fn steam_apps_symbol_replacement(symbol: SteamAppsSymbol) -> Option<*mut c_void> {
    let replacement = match symbol {
        SteamAppsSymbol::InterfaceVersion("v006") => hooked_steam_api_steam_apps_v006 as usize,
        SteamAppsSymbol::InterfaceVersion("v007") => hooked_steam_api_steam_apps_v007 as usize,
        SteamAppsSymbol::InterfaceVersion("v008") => hooked_steam_api_steam_apps_v008 as usize,
        SteamAppsSymbol::InterfaceVersion(_) => return None,
        SteamAppsSymbol::BoolMethod(SteamAppsBoolMethod::IsSubscribedApp) => {
            hooked_steam_apps_flat_b_is_subscribed_app as usize
        }
        SteamAppsSymbol::BoolMethod(SteamAppsBoolMethod::IsDlcInstalled) => {
            hooked_steam_apps_flat_b_is_dlc_installed as usize
        }
    };
    Some(replacement as *mut c_void)
}

fn remember_dynamic_steam_apps_symbol(symbol: SteamAppsSymbol, original: *mut c_void) {
    match symbol {
        SteamAppsSymbol::InterfaceVersion(version) => {
            if let Some(cell) = steam_apps_interface_dynamic_original_cell(version) {
                cell.store(original as usize, Ordering::Release);
            }
        }
        SteamAppsSymbol::BoolMethod(method) => {
            steam_apps_flat_dynamic_original_cell(method)
                .store(original as usize, Ordering::Release);
        }
    }
}

unsafe fn steam_apps_interface_original(version: &str) -> Option<SteamApiSteamAppsFn> {
    let imported = ORIGINALS.get().and_then(|originals| match version {
        "v006" => originals.steam_apps_v006,
        "v007" => originals.steam_apps_v007,
        "v008" => originals.steam_apps_v008,
        _ => None,
    });
    if imported.is_some() {
        return imported;
    }

    let address = steam_apps_interface_dynamic_original_cell(version)?.load(Ordering::Acquire);
    (address != 0).then(|| std::mem::transmute(address))
}

unsafe fn steam_apps_flat_original(method: SteamAppsBoolMethod) -> Option<SteamAppsBoolAppIdFn> {
    let imported = ORIGINALS.get().and_then(|originals| match method {
        SteamAppsBoolMethod::IsSubscribedApp => originals.steam_apps_b_is_subscribed_app,
        SteamAppsBoolMethod::IsDlcInstalled => originals.steam_apps_b_is_dlc_installed,
    });
    if imported.is_some() {
        return imported;
    }

    let address = steam_apps_flat_dynamic_original_cell(method).load(Ordering::Acquire);
    (address != 0).then(|| std::mem::transmute(address))
}

fn steam_apps_interface_dynamic_original_cell(version: &str) -> Option<&'static AtomicUsize> {
    match version {
        "v006" => Some(&STEAM_APPS_V006_DYNAMIC_ORIGINAL),
        "v007" => Some(&STEAM_APPS_V007_DYNAMIC_ORIGINAL),
        "v008" => Some(&STEAM_APPS_V008_DYNAMIC_ORIGINAL),
        _ => None,
    }
}

fn steam_apps_flat_dynamic_original_cell(method: SteamAppsBoolMethod) -> &'static AtomicUsize {
    match method {
        SteamAppsBoolMethod::IsSubscribedApp => &STEAM_APPS_B_IS_SUBSCRIBED_APP_DYNAMIC_ORIGINAL,
        SteamAppsBoolMethod::IsDlcInstalled => &STEAM_APPS_B_IS_DLC_INSTALLED_DYNAMIC_ORIGINAL,
    }
}

unsafe fn install_steam_apps_vtable_probe(apps: *mut c_void, version: &str) {
    if apps.is_null() {
        log_steam_apps_probe_event(format_args!("SteamApps {version} returned null"));
        return;
    }
    if STEAM_APPS_VTABLE_PATCHED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        return;
    }

    let vtable = *(apps as *const *mut usize);
    if vtable.is_null() {
        STEAM_APPS_VTABLE_PATCHED.store(false, Ordering::Release);
        log_steam_apps_probe_event(format_args!(
            "SteamApps probe skipped version={version} interface=0x{:x} reason=null_vtable",
            apps as usize
        ));
        return;
    }

    let subscribed = patch_steam_apps_vtable_method(
        vtable,
        SteamAppsBoolMethod::IsSubscribedApp,
        hooked_steam_apps_vtable_b_is_subscribed_app as usize,
    );
    let dlc = patch_steam_apps_vtable_method(
        vtable,
        SteamAppsBoolMethod::IsDlcInstalled,
        hooked_steam_apps_vtable_b_is_dlc_installed as usize,
    );
    if !subscribed || !dlc {
        STEAM_APPS_VTABLE_PATCHED.store(false, Ordering::Release);
    }
    log_steam_apps_probe_event(format_args!(
        "SteamApps probe installed version={version} interface=0x{:x} vtable=0x{:x} subscribed={subscribed} dlc={dlc}",
        apps as usize,
        vtable as usize
    ));
}

unsafe fn patch_steam_apps_vtable_method(
    vtable: *mut usize,
    method: SteamAppsBoolMethod,
    replacement: usize,
) -> bool {
    let slot = vtable.add(method.vtable_index());
    if *slot == replacement {
        return true;
    }
    let original = *slot;
    if original == 0 {
        return false;
    }
    let mut old_protect = 0;
    if !win::make_memory_writable(slot.cast(), size_of::<usize>(), &mut old_protect) {
        return false;
    }
    *slot = replacement;
    let restored = win::restore_memory_protection(slot.cast(), size_of::<usize>(), old_protect);
    if restored {
        steam_apps_vtable_original_cell(method).store(original, Ordering::Release);
    }
    restored
}

unsafe fn steam_apps_vtable_original(method: SteamAppsBoolMethod) -> Option<SteamAppsBoolAppIdFn> {
    let address = steam_apps_vtable_original_cell(method).load(Ordering::Acquire);
    (address != 0).then(|| std::mem::transmute(address))
}

fn steam_apps_vtable_original_cell(method: SteamAppsBoolMethod) -> &'static AtomicUsize {
    match method {
        SteamAppsBoolMethod::IsSubscribedApp => &STEAM_APPS_B_IS_SUBSCRIBED_APP_VTABLE_ORIGINAL,
        SteamAppsBoolMethod::IsDlcInstalled => &STEAM_APPS_B_IS_DLC_INSTALLED_VTABLE_ORIGINAL,
    }
}

fn log_steam_apps_check(
    source: &str,
    method: SteamAppsBoolMethod,
    this: *mut c_void,
    app_id: u32,
    result: bool,
) {
    let index = STEAM_APPS_LOGS.fetch_add(1, Ordering::Relaxed);
    if index >= steam::STEAM_APPS_MAX_LOGS {
        if index == steam::STEAM_APPS_MAX_LOGS {
            log::write_line("SteamApps check logs suppressed".to_string());
        }
        return;
    }
    let stack = capture_stack_trace()
        .into_iter()
        .map(format_stack_frame)
        .collect::<Vec<_>>()
        .join(" ");
    log::write_line(format!(
        "SteamApps {source} {} app_id={app_id} result={result} this=0x{:x} frames={stack}",
        method.label(),
        this as usize
    ));
}

fn log_steam_apps_probe_event(args: std::fmt::Arguments<'_>) {
    let index = STEAM_APPS_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < steam::STEAM_APPS_MAX_LOGS {
        log::write_line(args.to_string());
    }
}

fn open_virtual_fake_handle(path: &str) -> Option<Handle> {
    if let Some((virtual_handle, replacement, runtime_kind)) =
        open_law_custom_slot_virtual_handle(path)
    {
        let handle = returned_virtual_handle_for_runtime(virtual_handle, runtime_kind);
        remember_virtual_source(runtime_kind, virtual_handle, replacement.source.clone());
        log_open_virtual(path, handle, runtime_kind, &replacement);
        return Some(handle);
    }

    let (virtual_handle, replacement) = with_manager(|manager| {
        let (virtual_handle, replacement) = open_virtual_handle_for_path(manager, path)?;
        Some((virtual_handle, replacement))
    })
    .flatten()?;
    let runtime_kind = VirtualRuntimeKind::Global;
    let handle = returned_virtual_handle_for_runtime(virtual_handle, runtime_kind);
    remember_virtual_source(runtime_kind, virtual_handle, replacement.source.clone());
    log_open_virtual(path, handle, runtime_kind, &replacement);
    Some(handle)
}

fn open_law_custom_slot_virtual_handle(
    path: &str,
) -> Option<(VirtualHandle, VirtualReplacement, VirtualRuntimeKind)> {
    if !law_custom_slot_path_matches(path) {
        return None;
    }

    let runtime_kind = law_custom_slot_open_runtime_kind(law_custom_slot_active());
    let (virtual_handle, replacement) = with_runtime_manager(runtime_kind, |manager| {
        let (virtual_handle, replacement) = open_virtual_handle_for_path(manager, path)?;
        Some((virtual_handle, replacement))
    })
    .flatten()?;
    Some((virtual_handle, replacement, runtime_kind))
}

fn law_custom_slot_open_runtime_kind(active: bool) -> VirtualRuntimeKind {
    if active || LAW_CUSTOM_SLOT_DORMANT_ASSETS_ALWAYS_ACTIVE {
        VirtualRuntimeKind::LawCustomSlot
    } else {
        VirtualRuntimeKind::LawCustomOriginal
    }
}

fn law_custom_slot_path_matches(path: &str) -> bool {
    with_runtime_manager(VirtualRuntimeKind::LawCustomSlot, |manager| {
        let file_match = Path::new(path)
            .file_name()
            .map(|name| manager.contains_path_fragment(name.to_string_lossy().as_ref()))
            .unwrap_or(false);
        file_match || manager.contains_path_fragment(path)
    })
    .unwrap_or(false)
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
    runtime_kind: VirtualRuntimeKind,
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
    let Some(result) = with_runtime_manager(runtime_kind, |manager| {
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

fn log_open_virtual(
    path: &str,
    returned_handle: Handle,
    runtime_kind: VirtualRuntimeKind,
    replacement: &VirtualReplacement,
) {
    let index = OPEN_VIRTUAL_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 80 {
        let prefix_len = replacement
            .virtual_prefix
            .as_ref()
            .map(|prefix| prefix.len())
            .unwrap_or_default();
        log::write_line(format!(
            "Open virtual {path} runtime={} handle=0x{:x} file={} mode={:?} hash=0x{:08x} prefix=0x{prefix_len:x} mod_size=0x{:x} source={}",
            runtime_kind.label(),
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

fn close_virtual_file(handle: VirtualHandle, runtime_kind: VirtualRuntimeKind) -> Bool {
    let closed = with_runtime_manager(runtime_kind, |manager| manager.close(handle))
        .filter(|closed| *closed)
        .map(|_| 1)
        .unwrap_or(0);
    if closed != 0 {
        forget_virtual_source(runtime_kind, handle);
    }
    log_virtual_io(format_args!(
        "Virtual CLOSE handle=0x{:x} closed={closed}",
        handle.as_raw()
    ));
    closed
}

unsafe fn get_virtual_file_size(
    handle: VirtualHandle,
    runtime_kind: VirtualRuntimeKind,
    size: *mut LargeInteger,
) -> Bool {
    if size.is_null() {
        return 0;
    }
    let Some(result) = with_runtime_manager(runtime_kind, |manager| manager.size(handle).ok())
    else {
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
    runtime_kind: VirtualRuntimeKind,
    creation_time: Lpvoid,
    last_access_time: Lpvoid,
    last_write_time: Lpvoid,
) -> Bool {
    if let Some((creation, access, write)) = virtual_file_times(handle, runtime_kind) {
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
    runtime_kind: VirtualRuntimeKind,
) -> Option<(FileTime, FileTime, FileTime)> {
    let source = virtual_source(runtime_kind, handle)?;
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
    runtime_kind: VirtualRuntimeKind,
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
    let Some(result) =
        with_runtime_manager(runtime_kind, |manager| manager.seek(handle, position).ok())
    else {
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

fn with_manager<T>(action: impl FnOnce(&mut VirtualManager) -> T) -> Option<T> {
    with_runtime_manager(VirtualRuntimeKind::Global, action)
}

fn with_runtime_manager<T>(
    runtime_kind: VirtualRuntimeKind,
    action: impl FnOnce(&mut VirtualManager) -> T,
) -> Option<T> {
    let runtime = match runtime_kind {
        VirtualRuntimeKind::Global => RUNTIME.get()?,
        VirtualRuntimeKind::LawCustomSlot => LAW_CUSTOM_SLOT_RUNTIME.get()?,
        VirtualRuntimeKind::LawCustomOriginal => LAW_CUSTOM_SLOT_ORIGINAL_RUNTIME.get()?,
    };
    let mut guard = runtime.lock().ok()?;
    let manager = guard.as_mut()?;
    Some(action(manager))
}

fn remember_virtual_source(
    runtime_kind: VirtualRuntimeKind,
    handle: VirtualHandle,
    source: ReplacementSource,
) {
    let sources = VIRTUAL_SOURCES.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = sources.lock() {
        guard.insert(virtual_source_key(runtime_kind, handle), source);
    }
}

fn forget_virtual_source(runtime_kind: VirtualRuntimeKind, handle: VirtualHandle) {
    let Some(sources) = VIRTUAL_SOURCES.get() else {
        return;
    };
    if let Ok(mut guard) = sources.lock() {
        guard.remove(&virtual_source_key(runtime_kind, handle));
    }
}

fn virtual_source(
    runtime_kind: VirtualRuntimeKind,
    handle: VirtualHandle,
) -> Option<ReplacementSource> {
    let sources = VIRTUAL_SOURCES.get()?;
    let guard = sources.lock().ok()?;
    guard
        .get(&virtual_source_key(runtime_kind, handle))
        .cloned()
}

fn virtual_source_key(runtime_kind: VirtualRuntimeKind, handle: VirtualHandle) -> u64 {
    ((runtime_kind.id() as u64) << 56) | handle.as_raw()
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
            "{} READ {}: offset=0x{offset:x} bytes=0x{bytes_to_read:x}",
            tracked.kind.label(),
            tracked.archive_name
        ));
    }
    Some((tracked, offset))
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
        TrackedFileKind::Index => {
            patch_index_external_flags(&tracked.archive_name, offset as u64, buffer);
        }
        TrackedFileKind::Data => {
            patch_data_read(&tracked.archive_name, offset as u64, buffer);
            log_data_read_hits(&tracked.archive_name, offset as u64, buffer.len());
        }
    }
}

fn patch_data_read(archive_name: &str, read_offset: u64, buffer: &mut [u8]) {
    let global_patched =
        with_manager(|manager| manager.patch_archive_read(archive_name, read_offset, buffer))
            .and_then(Result::ok)
            .unwrap_or_default();
    let slot_patched =
        if LAW_SLOT5_PRIVATE_ASSET_DATA_READ_PATCH_ENABLED && law_custom_slot_active() {
        with_runtime_manager(VirtualRuntimeKind::LawCustomSlot, |manager| {
            manager.patch_archive_original_read(archive_name, read_offset, buffer)
        })
        .and_then(Result::ok)
        .unwrap_or_default()
    } else {
        0
    };
    let patched = global_patched + slot_patched;
    if patched == 0 || DATA_PATCH_LOGS.load(Ordering::Relaxed) >= 80 {
        return;
    }
    let index = DATA_PATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 80 {
        log::write_line(format!(
            "RDB BIN PATCH {archive_name}: read=0x{read_offset:x}+0x{:x} bytes={patched} law_custom_active={} mode={}",
            buffer.len(),
            law_custom_slot_active(),
            if slot_patched > 0 {
                "law-custom-original-offset"
            } else {
                "global-virtual-offset"
            }
        ));
    }
}

fn patch_index_external_flags(archive_name: &str, read_offset: u64, buffer: &mut [u8]) {
    let global_patched = with_manager(|manager| {
        manager.patch_archive_index_external_flags(archive_name, read_offset, buffer)
    })
    .unwrap_or_default();
    let slot_patched = if law_custom_slot_index_patch_enabled() {
        with_runtime_manager(VirtualRuntimeKind::LawCustomSlot, |manager| {
            manager.patch_archive_index_external_flags(archive_name, read_offset, buffer)
        })
        .unwrap_or_default()
    } else {
        0
    };
    let patched = global_patched + slot_patched;
    if patched == 0 || !verbose_io_logs() || INDEX_PATCH_LOGS.load(Ordering::Relaxed) >= 80 {
        return;
    }
    let index = INDEX_PATCH_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 80 {
        log::write_line(format!(
            "RDB INDEX EXTERNAL {archive_name}: read=0x{read_offset:x}+0x{:x} fields={patched}",
            buffer.len()
        ));
    }
}

fn law_custom_slot_index_patch_enabled() -> bool {
    false
}

fn log_create_file_candidate(path: &str, desired_access: Dword, creation_disposition: Dword) {
    if !verbose_io_logs() {
        return;
    }
    if CREATE_FILE_LOGS.load(Ordering::Relaxed) >= 320 {
        return;
    }
    let lower = path.to_ascii_lowercase();
    let interesting = lower.contains("oppw4")
        || lower.contains("op4")
        || lower.contains(".rdb")
        || lower.contains(".g1")
        || lower.contains("0x");
    if !interesting {
        return;
    }
    let index = CREATE_FILE_LOGS.fetch_add(1, Ordering::Relaxed);
    if index < 320 {
        log::write_line(format!(
            "CreateFileW path={path} access=0x{desired_access:x} disposition=0x{creation_disposition:x}"
        ));
    }
}

fn maybe_dump_costume_table(path: &str, phase: &str) {
    if !DUMP_COSTUME_TABLE_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if COSTUME_TABLE_DUMP_DONE.load(Ordering::Relaxed) {
        return;
    }
    let Some(trigger) = dlc_stack_path_kind(path) else {
        return;
    };
    let attempt = COSTUME_TABLE_DUMP_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
    if attempt >= 32 {
        if attempt == 32 {
            log::write_line(format!(
                "Global costume table dump skipped phase={phase}: attempts exhausted"
            ));
        }
        return;
    }

    let mut memory = ProcessMemoryReader;
    match costume_table::dump_costume_table(win::main_module() as usize, &mut memory) {
        Ok(dump) => {
            COSTUME_TABLE_DUMP_DONE.store(true, Ordering::Relaxed);
            log_costume_table_dump(phase, trigger, &dump);
            log_law_menu_row_slot_probe(&dump);
            match costume_table::dump_costume_layout_table(win::main_module() as usize, &mut memory)
            {
                Ok(layout_dump) => {
                    log_costume_layout_table_dump(&layout_dump);
                    log_law_ram_slot_source(&dump, &layout_dump);
                    maybe_patch_law_linkdata_only_ram_variant699_metadata(&layout_dump);
                    maybe_patch_law_duplicate_variant_slot(&layout_dump);
                }
                Err(error) => log::write_line(format!(
                    "Global costume layout dump failed phase={phase} trigger={trigger} error={error:?}"
                )),
            }
        }
        Err(error) => {
            if attempt < 8 {
                log::write_line(format!(
                    "Global costume table dump not ready phase={phase} trigger={trigger} attempt={} error={error:?}",
                    attempt + 1
                ));
            }
        }
    }
}
