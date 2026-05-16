use std::{
    mem,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
    sync::{Mutex, OnceLock},
    thread,
    time::Duration,
};

use oppw4_plugin_api::{
    Oppw4PluginApi, OPPW4_GAME_FLAG_DLC_CHARACTER_SEEN, OPPW4_GAME_FLAG_VIRTUAL_RESOURCE_SEEN,
};

use crate::{
    config::{AuraConfig, InstallMode, StatusGate, TargetMode, TriggerMode},
    log, memory,
};

const WEAPON_AURA_ID_PATTERN: &[u8] = &[0x44, 0x8b, 0x7b, 0x54, 0x41, 0x8b, 0x50, 0x04];
const WEAPON_AURA_ID_MASK: &[u8] = &[1; 8];
const AURA_DURATION_PATTERN: &[u8] = &[0xf3, 0x0f, 0x10, 0x9f, 0xe8, 0x02, 0x00, 0x00];
const AURA_DURATION_MASK: &[u8] = &[1; 8];
const LOCAL_PLAYER_PATTERN: &[u8] = &[0x48, 0x8b, 0x80, 0xd0, 0x02, 0x00, 0x00, 0xf3];
const LOCAL_PLAYER_MASK: &[u8] = &[1; 8];
const AURA_UPDATE_ENTRY_OFFSET_FROM_ID_SITE: usize = 0xc6;
const AURA_UPDATE_ENTRY_OVERWRITE_LEN: usize = 20;
const AURA_UPDATE_ENTRY_PREFIX: &[u8] = &[
    0x4c, 0x8b, 0xdc, 0x45, 0x89, 0x43, 0x18, 0x55, 0x56, 0x49, 0x8d, 0x6b, 0xa1, 0x48,
    0x81, 0xec, 0xf8, 0x00, 0x00, 0x00,
];

static INSTALL: OnceLock<Mutex<Option<InstallState>>> = OnceLock::new();
static DEFERRED_STARTED: OnceLock<()> = OnceLock::new();
static AURA_UPDATE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static AURA_UPDATE_DATA: OnceLock<AuraDataPtrs> = OnceLock::new();
const CAVE_ARENA_SIZE: usize = 0x10000;

#[derive(Clone, Copy)]
struct WorkerApi(Oppw4PluginApi);

unsafe impl Send for WorkerApi {}

impl WorkerApi {
    fn install_now(self, config: AuraConfig) -> Result<(), String> {
        install_now(&self.0, config)
    }

    fn load_config(self) -> Option<AuraConfig> {
        AuraConfig::load(&self.0)
    }
}

struct InstallState {
    data: AuraDataPtrs,
    _aura_update_hook: Option<InlineHook>,
}

#[derive(Clone, Copy)]
struct AuraDataPtrs {
    enabled: usize,
    force_effect_id: usize,
    effect_id: usize,
    local_player_filter: usize,
    local_player: usize,
    local_player_fx_owner: usize,
    speed: usize,
    timer: usize,
    loop_start: usize,
    loop_end: usize,
    id_hits: usize,
    id_forced_hits: usize,
    duration_hits: usize,
    duration_match_hits: usize,
    observed_effect_id: usize,
    observed_edx: usize,
    observed_aura_param: usize,
    observed_aura_owner: usize,
    observed_effect: usize,
}

pub(crate) fn install_deferred(api: Oppw4PluginApi, config: AuraConfig) -> i32 {
    if DEFERRED_STARTED.set(()).is_err() {
        log::write_line("weapon_aura deferred install already started");
        return 0;
    }
    let delay = config.install_delay_ms;
    let worker_api = WorkerApi(api);
    let builder = thread::Builder::new().name("oppw4_weapon_aura".to_string());
    match builder.spawn(move || {
        log::write_line(format!("weapon_aura deferred install sleeping {delay}ms"));
        thread::sleep(Duration::from_millis(delay));
        wait_for_status_gate(worker_api, config.wait_for);
        if config.trigger == TriggerMode::Hotkey {
            log::write_line(format!(
                "weapon_aura waiting for hotkey vk=0x{:02x} before patching",
                config.hotkey_vk
            ));
            wait_for_hotkey(config.hotkey_vk);
            log::write_line("weapon_aura hotkey pressed, starting patch attempt");
        }
        if let Some(status) = worker_api.0.game_status() {
            log::write_line(format!(
                "weapon_aura game_status before install phase={} flags=0x{:x} file_opens={} seconds={}",
                status.phase, status.flags, status.observed_file_opens, status.seconds_since_host_start
            ));
        }
        match worker_api.install_now(config) {
            Ok(()) => log::write_line("weapon_aura deferred install completed"),
            Err(error) => log::write_line(format!("weapon_aura deferred install failed: {error}")),
        }
    }) {
        Ok(_) => 0,
        Err(error) => {
            log::write_line(format!("weapon_aura deferred thread spawn failed: {error}"));
            -10
        }
    }
}

fn wait_for_status_gate(api: WorkerApi, gate: StatusGate) {
    let Some(required_flag) = status_gate_flag(gate) else {
        log::write_line("weapon_aura status gate disabled");
        return;
    };
    log::write_line(format!("weapon_aura waiting for status gate {gate:?}"));
    let mut last_log_second = u32::MAX;
    loop {
        let Some(status) = api.0.game_status() else {
            log::write_line("weapon_aura status gate skipped: host game_status unavailable");
            return;
        };
        if status.flags & required_flag != 0 {
            log::write_line(format!(
                "weapon_aura status gate {gate:?} passed phase={} flags=0x{:x} file_opens={} seconds={}",
                status.phase, status.flags, status.observed_file_opens, status.seconds_since_host_start
            ));
            return;
        }
        if status.seconds_since_host_start != last_log_second
            && status.seconds_since_host_start % 5 == 0
        {
            log::write_line(format!(
                "weapon_aura status gate {gate:?} pending phase={} flags=0x{:x} file_opens={} seconds={}",
                status.phase, status.flags, status.observed_file_opens, status.seconds_since_host_start
            ));
            last_log_second = status.seconds_since_host_start;
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn status_gate_flag(gate: StatusGate) -> Option<u32> {
    match gate {
        StatusGate::None => None,
        StatusGate::VirtualResource => Some(OPPW4_GAME_FLAG_VIRTUAL_RESOURCE_SEEN),
        StatusGate::DlcCharacter => Some(OPPW4_GAME_FLAG_DLC_CHARACTER_SEEN),
    }
}

fn wait_for_hotkey(vk: i32) {
    loop {
        let state = unsafe { GetAsyncKeyState(vk) };
        if state & 1 != 0 {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[link(name = "user32")]
extern "system" {
    fn GetAsyncKeyState(v_key: i32) -> i16;
}

fn install_now(api: &Oppw4PluginApi, config: AuraConfig) -> Result<(), String> {
    if !config.enabled {
        log::write_line("weapon_aura disabled by config");
        return Ok(());
    }

    let needs_local_player = config.target == TargetMode::LocalPlayer || config.observe_character_probe;
    let local_player_site = if needs_local_player {
        log::write_line("weapon_aura scanning LocalPlayerHook signature");
        let site = api.scan_memory(LOCAL_PLAYER_PATTERN, LOCAL_PLAYER_MASK);
        if site == 0 {
            return Err("local player signature not found".to_string());
        }
        log::write_line(format!(
            "weapon_aura LocalPlayerHook signature site=0x{site:x}"
        ));
        Some(site)
    } else {
        log::write_line("weapon_aura target=all and character probe disabled: LocalPlayerHook skipped");
        None
    };

    log::write_line("weapon_aura scanning TestWeaponAuraV2 signature");
    let id_site = api.scan_memory(WEAPON_AURA_ID_PATTERN, WEAPON_AURA_ID_MASK);
    if id_site == 0 {
        return Err("weapon aura id signature not found".to_string());
    }
    log::write_line(format!(
        "weapon_aura TestWeaponAuraV2 signature site=0x{id_site:x}"
    ));

    let duration_site = if config.force_effect_id {
        log::write_line("weapon_aura scanning TestWeaponAuraDuration signature");
        let site = api.scan_memory(AURA_DURATION_PATTERN, AURA_DURATION_MASK);
        if site == 0 {
            return Err("aura duration signature not found".to_string());
        }
        log::write_line(format!(
            "weapon_aura TestWeaponAuraDuration signature site=0x{site:x}"
        ));
        Some(site)
    } else {
        log::write_line("weapon_aura observer mode: TestWeaponAuraDuration skipped");
        None
    };

    if config.install_mode == InstallMode::ScanOnly {
        log::write_line("weapon_aura scan_only mode: signatures found, patching skipped");
        return Ok(());
    }

    log::write_line("weapon_aura allocating cave arena");
    let mut arena = CaveArena::new(id_site)?;
    log::write_line(format!(
        "weapon_aura cave arena base=0x{:x} size=0x{:x}",
        arena.base, CAVE_ARENA_SIZE
    ));
    log::write_line("weapon_aura allocating shared data");
    let data = build_shared_data(&mut arena, config)?;
    log::write_line(format!(
        "weapon_aura shared data enabled=0x{:x} force_effect_id=0x{:x} effect_id=0x{:x} local_player_filter=0x{:x} local_player=0x{:x} local_player_fx_owner=0x{:x}",
        data.enabled, data.force_effect_id, data.effect_id, data.local_player_filter, data.local_player, data.local_player_fx_owner
    ));
    let local_cave = if let Some(local_player_site) = local_player_site {
        log::write_line("weapon_aura building LocalPlayerHook cave");
        let cave = build_local_player_cave(&mut arena, local_player_site + 7, data)?;
        log::write_line(format!(
            "weapon_aura LocalPlayerHook cave=0x{:x}",
            cave.entry
        ));
        Some((local_player_site, cave))
    } else {
        None
    };
    let aura_update_hook = if config.force_effect_id || config.observe_effect_ids {
        log::write_line("weapon_aura installing function hook for aura update");
        let function_entry = id_site
            .checked_sub(AURA_UPDATE_ENTRY_OFFSET_FROM_ID_SITE)
            .ok_or_else(|| "weapon aura id site before expected function entry".to_string())?;
        verify_aura_update_entry(api, function_entry)?;
        let _ = AURA_UPDATE_DATA.set(data);
        let hook = unsafe {
            InlineHook::install(
                api,
                &mut arena,
                function_entry,
                aura_update_detour as usize,
                AURA_UPDATE_ENTRY_OVERWRITE_LEN,
            )?
        };
        AURA_UPDATE_ORIGINAL.store(hook.trampoline, Ordering::Release);
        log::write_line(format!(
            "weapon_aura aura update hook entry=0x{function_entry:x} trampoline=0x{:x}",
            hook.trampoline
        ));
        Some(hook)
    } else {
        log::write_line("weapon_aura aura update hook skipped: passthrough mode");
        None
    };
    let duration_cave = if let Some(duration_site) = duration_site {
        log::write_line("weapon_aura building TestWeaponAuraDuration cave");
        let cave = build_duration_cave(
            &mut arena,
            duration_site + AURA_DURATION_PATTERN.len(),
            data,
        )?;
        log::write_line(format!(
            "weapon_aura TestWeaponAuraDuration cave=0x{:x}",
            cave.entry
        ));
        Some((duration_site, cave))
    } else {
        None
    };

    unsafe {
        if let Some((local_player_site, local_cave)) = local_cave {
            log::write_line("weapon_aura patching LocalPlayerHook jump");
            patch_jump(api, local_player_site, local_cave.entry, 7)?;
        }
        if let Some((duration_site, duration_cave)) = duration_cave {
            log::write_line("weapon_aura patching TestWeaponAuraDuration jump");
            patch_jump(
                api,
                duration_site,
                duration_cave.entry,
                AURA_DURATION_PATTERN.len(),
            )?;
        }
    }

    let state = InstallState {
        data,
        _aura_update_hook: aura_update_hook,
    };
    let install = INSTALL.get_or_init(|| Mutex::new(None));
    *install.lock().map_err(|_| "install lock poisoned")? = Some(state);

    log::write_line(format!(
        "weapon_aura hooks installed target={:?} local_player_site={} aura_update_site=0x{id_site:x} duration_site={}",
        config.target,
        local_player_site
            .map(|site| format!("0x{site:x}"))
            .unwrap_or_else(|| "skipped".to_string()),
        duration_site
            .map(|site| format!("0x{site:x}"))
            .unwrap_or_else(|| "skipped".to_string())
    ));
    start_config_reloader(WorkerApi(*api), data, config);
    if config.force_effect_id || config.observe_effect_ids {
        start_observed_id_logger(data);
        start_counter_logger(data);
    } else {
        log::write_line("weapon_aura passthrough mode: counters disabled");
    }
    if config.observe_character_probe {
        start_character_probe_logger(data);
    } else {
        log::write_line("weapon_aura character probe disabled");
    }
    Ok(())
}

pub(crate) fn set_enabled(enabled: bool) -> i32 {
    with_state(|state| {
        write_u32(state.data.enabled, enabled as u32);
    })
}

pub(crate) fn set_effect_id(effect_id: u32) -> i32 {
    with_state(|state| {
        write_u32(state.data.effect_id, effect_id);
    })
}

#[allow(dead_code)]
pub(crate) fn set_force_effect_id(force: bool) -> i32 {
    with_state(|state| {
        write_u32(state.data.force_effect_id, force as u32);
    })
}

pub(crate) fn set_timing(animation_speed: f32, loop_start: f32, loop_end: f32) -> i32 {
    with_state(|state| {
        write_f32(state.data.speed, animation_speed);
        write_f32(state.data.loop_start, loop_start);
        write_f32(state.data.loop_end, loop_end);
    })
}

fn with_state(action: impl FnOnce(&InstallState)) -> i32 {
    let Some(install) = INSTALL.get() else {
        return -1;
    };
    let Ok(guard) = install.lock() else {
        return -2;
    };
    let Some(state) = guard.as_ref() else {
        return -3;
    };
    action(state);
    0
}

fn start_counter_logger(data: AuraDataPtrs) {
    let _ = thread::Builder::new()
        .name("oppw4_weapon_aura_counters".to_string())
        .spawn(move || {
            for _ in 0..12 {
                thread::sleep(Duration::from_secs(5));
                log::write_line(format!(
                    "weapon_aura counters id_hits={} id_forced={} duration_hits={} duration_match={}",
                    read_u32(data.id_hits),
                    read_u32(data.id_forced_hits),
                    read_u32(data.duration_hits),
                    read_u32(data.duration_match_hits)
                ));
            }
        });
}

fn start_observed_id_logger(data: AuraDataPtrs) {
    let _ = thread::Builder::new()
        .name("oppw4_weapon_aura_observed".to_string())
        .spawn(move || {
            let mut last_effect_id = u32::MAX;
            let mut last_edx = u32::MAX;
            loop {
                thread::sleep(Duration::from_millis(250));
                let effect_id = read_u32(data.observed_effect_id);
                let edx = read_u32(data.observed_edx);
                if effect_id == 0 && edx == 0 {
                    continue;
                }
                if effect_id != last_effect_id || edx != last_edx {
                    log::write_line(format!(
                        "weapon_aura observed original_effect_id={} edx={}",
                        effect_id, edx
                    ));
                    last_effect_id = effect_id;
                    last_edx = edx;
                }
            }
        });
}

fn start_character_probe_logger(data: AuraDataPtrs) {
    let _ = thread::Builder::new()
        .name("oppw4_weapon_aura_character_probe".to_string())
        .spawn(move || {
            let mut last = String::new();
            loop {
                thread::sleep(Duration::from_secs(2));
                let local_player = read_usize(data.local_player);
                let local_owner = read_usize(data.local_player_fx_owner);
                let aura_param = read_usize(data.observed_aura_param);
                let aura_owner = read_usize(data.observed_aura_owner);
                let effect = read_usize(data.observed_effect);
                if local_player == 0 && aura_param == 0 {
                    continue;
                }
                let snapshot = character_probe_snapshot(
                    local_player,
                    local_owner,
                    aura_param,
                    aura_owner,
                    effect,
                );
                if snapshot != last {
                    log::write_line(format!("weapon_aura character_probe {snapshot}"));
                    last = snapshot;
                }
            }
        });
}

fn character_probe_snapshot(
    local_player: usize,
    local_owner: usize,
    aura_param: usize,
    aura_owner: usize,
    effect: usize,
) -> String {
    format!(
        "local={} owner={} aura={} aura_owner={} effect={} lp_u32=[{}] owner_u32=[{}] aura_u32=[{}] effect_u32=[{}] lp_ptrs=[{}] owner_ptrs=[{}] aura_ptrs=[{}]",
        fmt_ptr(local_player),
        fmt_ptr(local_owner),
        fmt_ptr(aura_param),
        fmt_ptr(aura_owner),
        fmt_ptr(effect),
        probe_u32_fields(local_player, &[0x0, 0x10, 0x24, 0x28, 0x30, 0xd0, 0xd8, 0xdc, 0x118, 0x158, 0x160, 0x280, 0x290]),
        probe_u32_fields(local_owner, &[0x0, 0x10, 0x24, 0x28, 0x30, 0x54, 0xd0, 0xd8, 0xdc, 0x118, 0x158, 0x280, 0x290]),
        probe_u32_fields(aura_param, &[0x0, 0x10, 0x24, 0x28, 0x30, 0xd0, 0xd8, 0xdc, 0x280, 0x290]),
        probe_u32_fields(effect, &[0x0, 0x10, 0x30, 0x54, 0x58, 0x5c, 0x60]),
        probe_ptr_fields(local_player, &[0x90, 0x118, 0x2a0, 0x2c0, 0x2d0, 0x2d8, 0x460]),
        probe_ptr_fields(local_owner, &[0x90, 0x118, 0x2a0, 0x2c0, 0x2d0, 0x2d8]),
        probe_ptr_fields(aura_param, &[0x28, 0x38, 0x410, 0x458, 0x2d0, 0x2d8])
    )
}

fn fmt_ptr(value: usize) -> String {
    if value == 0 {
        "0".to_string()
    } else {
        format!("0x{value:x}")
    }
}

fn probe_u32_fields(base: usize, offsets: &[usize]) -> String {
    if base == 0 {
        return "none".to_string();
    }
    offsets
        .iter()
        .map(|offset| format!("+0x{offset:x}:{}", unsafe {
            read_unaligned_u32(base + offset)
        }))
        .collect::<Vec<_>>()
        .join(",")
}

fn probe_ptr_fields(base: usize, offsets: &[usize]) -> String {
    if base == 0 {
        return "none".to_string();
    }
    offsets
        .iter()
        .map(|offset| format!("+0x{offset:x}:{}", fmt_ptr(unsafe {
            read_unaligned_usize(base + offset)
        })))
        .collect::<Vec<_>>()
        .join(",")
}

fn start_config_reloader(api: WorkerApi, data: AuraDataPtrs, initial: AuraConfig) {
    let interval = initial.refresh_interval_ms;
    if interval == 0 {
        log::write_line("weapon_aura live config refresh disabled");
        return;
    }

    let _ = thread::Builder::new()
        .name("oppw4_weapon_aura_config".to_string())
        .spawn(move || {
            let mut current = initial;
            log::write_line(format!(
                "weapon_aura live config refresh every {interval}ms"
            ));
            loop {
                thread::sleep(Duration::from_millis(interval));
                let Some(next) = api.load_config() else {
                    continue;
                };
                if next.enabled != current.enabled {
                    write_u32(data.enabled, next.enabled as u32);
                    log::write_line(format!("weapon_aura live enabled={}", next.enabled));
                }
                if next.effect_id != current.effect_id {
                    write_u32(data.effect_id, next.effect_id);
                    log::write_line(format!(
                        "weapon_aura live effect_id {} -> {}",
                        current.effect_id, next.effect_id
                    ));
                }
                if next.force_effect_id != current.force_effect_id {
                    write_u32(data.force_effect_id, next.force_effect_id as u32);
                    log::write_line(format!(
                        "weapon_aura live force_effect_id {} -> {}",
                        current.force_effect_id, next.force_effect_id
                    ));
                }
                if next.animation_speed != current.animation_speed
                    || next.loop_start != current.loop_start
                    || next.loop_end != current.loop_end
                {
                    write_f32(data.speed, next.animation_speed);
                    write_f32(data.loop_start, next.loop_start);
                    write_f32(data.loop_end, next.loop_end);
                    log::write_line(format!(
                        "weapon_aura live timing speed={} loop_start={} loop_end={}",
                        next.animation_speed, next.loop_start, next.loop_end
                    ));
                }
                current.enabled = next.enabled;
                current.effect_id = next.effect_id;
                current.force_effect_id = next.force_effect_id;
                current.animation_speed = next.animation_speed;
                current.loop_start = next.loop_start;
                current.loop_end = next.loop_end;
            }
        });
}

struct CodeCave {
    entry: usize,
}

struct InlineHook {
    trampoline: usize,
}

struct CaveArena {
    base: usize,
    cursor: usize,
    size: usize,
}

impl CaveArena {
    fn new(hint: usize) -> Result<Self, String> {
        let base = unsafe { memory::allocate_near_executable_block(hint, CAVE_ARENA_SIZE) }
            .ok_or_else(|| "VirtualAlloc failed for aura cave arena".to_string())?;
        Ok(Self {
            base,
            cursor: 0,
            size: CAVE_ARENA_SIZE,
        })
    }

    fn alloc(&mut self, bytes: &[u8], alignment: usize) -> Result<usize, String> {
        let address = self.reserve(bytes.len(), alignment)?;
        unsafe { memory::write_bytes(address, bytes) };
        Ok(address)
    }

    fn reserve(&mut self, len: usize, alignment: usize) -> Result<usize, String> {
        self.cursor = align_up(self.cursor, alignment.max(1));
        let end = self
            .cursor
            .checked_add(len)
            .ok_or_else(|| "aura cave arena offset overflow".to_string())?;
        if end > self.size {
            return Err(format!(
                "aura cave arena full requested=0x{:x} available=0x{:x}",
                len,
                self.size.saturating_sub(self.cursor)
            ));
        }
        let address = self.base + self.cursor;
        self.cursor = end;
        Ok(address)
    }

    fn write_at(&self, address: usize, bytes: &[u8]) {
        unsafe { memory::write_bytes(address, bytes) };
    }
}

impl InlineHook {
    unsafe fn install(
        api: &Oppw4PluginApi,
        arena: &mut CaveArena,
        site: usize,
        detour: usize,
        overwrite_len: usize,
    ) -> Result<Self, String> {
        if overwrite_len < 12 {
            return Err("inline hook overwrite_len must fit absolute jump".to_string());
        }
        let mut original = vec![0u8; overwrite_len];
        let result = api.read_memory(site, &mut original);
        if result != 0 {
            return Err(format!("read_memory failed site=0x{site:x} result={result}"));
        }

        let mut trampoline_code = original;
        emit_abs_jmp(&mut trampoline_code, site + overwrite_len);
        let trampoline = arena.alloc(&trampoline_code, 16)?;

        let mut patch = Vec::with_capacity(overwrite_len);
        emit_abs_jmp(&mut patch, detour);
        patch.resize(overwrite_len, 0x90);
        let result = api.write_memory(site, &patch);
        if result != 0 {
            return Err(format!("write_memory failed site=0x{site:x} result={result}"));
        }

        Ok(Self { trampoline })
    }
}

#[derive(Clone, Copy)]
struct Rel32Patch {
    instruction_offset: usize,
    immediate_offset: usize,
    instruction_len: usize,
}

fn build_shared_data(arena: &mut CaveArena, config: AuraConfig) -> Result<AuraDataPtrs, String> {
    let mut bytes = Vec::new();
    append_data(&mut bytes, config);
    let base = arena.alloc(&bytes, 8)?;
    Ok(data_ptrs(base, 0))
}

fn build_local_player_cave(
    arena: &mut CaveArena,
    return_address: usize,
    data: AuraDataPtrs,
) -> Result<CodeCave, String> {
    let mut code = Vec::new();
    code.extend_from_slice(&[0x48, 0x8b, 0x80, 0xd0, 0x02, 0x00, 0x00]);
    code.extend_from_slice(&[0x48, 0x85, 0xc0]);
    let jump_back_null = emit_jz(&mut code);
    code.extend_from_slice(&[0x51, 0x52]);
    code.extend_from_slice(&[0x48, 0xb9]);
    code.extend_from_slice(&(data.local_player as u64).to_le_bytes());
    code.extend_from_slice(&[0x48, 0x89, 0x01]);
    code.extend_from_slice(&[0x48, 0x8d, 0x90, 0x60, 0x04, 0x00, 0x00]);
    code.extend_from_slice(&[0x48, 0xb9]);
    code.extend_from_slice(&(data.local_player_fx_owner as u64).to_le_bytes());
    code.extend_from_slice(&[0x48, 0x89, 0x11]);
    code.extend_from_slice(&[0x5a, 0x59]);
    let jump_back = emit_jmp(&mut code);

    let base = arena.reserve(code.len(), 16)?;
    patch_rel32_vec(&mut code, base, jump_back_null, return_address)?;
    patch_rel32_vec(&mut code, base, jump_back, return_address)?;
    arena.write_at(base, &code);
    Ok(CodeCave { entry: base })
}

fn build_duration_cave(
    arena: &mut CaveArena,
    return_address: usize,
    data: AuraDataPtrs,
) -> Result<CodeCave, String> {
    let mut code = Vec::new();
    let duration_hits_inc = emit_inc_rip_u32(&mut code);
    let enabled_cmp = emit_cmp_rip_u32(&mut code);
    let jump_original_disabled = emit_jz(&mut code);
    let force_cmp = emit_cmp_rip_u32(&mut code);
    let jump_original_not_forced = emit_jz(&mut code);
    let effect_cmp = emit_cmp_edx_rip(&mut code);
    let jump_original_mismatch = emit_jne(&mut code);
    let duration_match_hits_inc = emit_inc_rip_u32(&mut code);

    emit_mov_dword_rdi_disp_imm(&mut code, 0xe0, 1.0);
    emit_mov_dword_rdi_disp_imm(&mut code, 0xe4, 0.0);
    emit_mov_dword_rdi_disp_imm(&mut code, 0x2dc, 0.0);
    emit_mov_dword_rdi_disp_imm(&mut code, 0x2e8, 2.0);
    emit_mov_dword_rdi_disp_imm(&mut code, 0x2d0, 0.0);
    emit_mov_dword_rdi_disp_imm(&mut code, 0x2d4, 0.0);
    emit_mov_dword_rdi_disp_imm(&mut code, 0x2d8, 0.0);

    let timer_load = emit_movss_xmm2_rip(&mut code);
    let speed_load = emit_movss_xmm0_rip(&mut code);
    code.extend_from_slice(&[0xf3, 0x0f, 0x58, 0xd0]);
    let timer_store = emit_movss_rip_xmm2(&mut code);
    code.extend_from_slice(&[0xf3, 0x0f, 0x11, 0x97, 0xe4, 0x02, 0x00, 0x00]);
    emit_mov_dword_rdi_disp_imm(&mut code, 0x2ec, 1.0);
    let loop_end_load = emit_movss_xmm0_rip(&mut code);
    code.extend_from_slice(&[0x0f, 0x2e, 0xd0]);
    let jump_original_not_done = emit_jbe(&mut code);
    let loop_start_load = emit_movss_xmm0_rip(&mut code);
    let timer_reset = emit_movss_rip_xmm0(&mut code);

    let original_label = code.len();
    code.extend_from_slice(AURA_DURATION_PATTERN);
    let jump_back = emit_jmp(&mut code);

    let base = arena.reserve(code.len(), 16)?;
    patch_disp32_vec(&mut code, base, duration_hits_inc, data.duration_hits)?;
    patch_disp32_vec(&mut code, base, enabled_cmp, data.enabled)?;
    patch_rel32_vec(&mut code, base, jump_original_disabled, base + original_label)?;
    patch_disp32_vec(&mut code, base, force_cmp, data.force_effect_id)?;
    patch_rel32_vec(&mut code, base, jump_original_not_forced, base + original_label)?;
    patch_disp32_vec(&mut code, base, effect_cmp, data.effect_id)?;
    patch_rel32_vec(&mut code, base, jump_original_mismatch, base + original_label)?;
    patch_disp32_vec(
        &mut code,
        base,
        duration_match_hits_inc,
        data.duration_match_hits,
    )?;
    patch_disp32_vec(&mut code, base, timer_load, data.timer)?;
    patch_disp32_vec(&mut code, base, speed_load, data.speed)?;
    patch_disp32_vec(&mut code, base, timer_store, data.timer)?;
    patch_disp32_vec(&mut code, base, loop_end_load, data.loop_end)?;
    patch_rel32_vec(&mut code, base, jump_original_not_done, base + original_label)?;
    patch_disp32_vec(&mut code, base, loop_start_load, data.loop_start)?;
    patch_disp32_vec(&mut code, base, timer_reset, data.timer)?;
    patch_rel32_vec(&mut code, base, jump_back, return_address)?;
    arena.write_at(base, &code);
    Ok(CodeCave { entry: base })
}

unsafe fn patch_jump(
    api: &Oppw4PluginApi,
    site: usize,
    target: usize,
    overwrite_len: usize,
) -> Result<(), String> {
    let mut patch = vec![0x90; overwrite_len];
    patch[0] = 0xe9;
    let rel = rel32(site, 5, target)?;
    patch[1..5].copy_from_slice(&rel.to_le_bytes());
    let result = api.write_memory(site, &patch);
    if result != 0 {
        return Err(format!("write_memory failed site=0x{site:x} result={result}"));
    }
    Ok(())
}

fn verify_aura_update_entry(api: &Oppw4PluginApi, function_entry: usize) -> Result<(), String> {
    let mut bytes = vec![0u8; AURA_UPDATE_ENTRY_PREFIX.len()];
    let result = api.read_memory(function_entry, &mut bytes);
    if result != 0 {
        return Err(format!(
            "read_memory failed function_entry=0x{function_entry:x} result={result}"
        ));
    }
    if bytes != AURA_UPDATE_ENTRY_PREFIX {
        return Err(format!(
            "aura update function prefix mismatch entry=0x{function_entry:x}"
        ));
    }
    Ok(())
}

type AuraUpdateFn = unsafe extern "system" fn(usize, usize, u32);

unsafe extern "system" fn aura_update_detour(param_1: usize, param_2: usize, param_3: u32) {
    let original = AURA_UPDATE_ORIGINAL.load(Ordering::Acquire);
    if original == 0 {
        return;
    }
    let original: AuraUpdateFn = mem::transmute(original);
    let Some(data) = AURA_UPDATE_DATA.get().copied() else {
        original(param_1, param_2, param_3);
        return;
    };

    let Some(effect_field) = aura_effect_id_field(param_1) else {
        original(param_1, param_2, param_3);
        return;
    };

    let natural_effect_id = ptr::read_unaligned(effect_field);
    write_u32(data.id_hits, read_u32(data.id_hits).wrapping_add(1));
    write_u32(data.observed_effect_id, natural_effect_id);
    write_u32(data.observed_edx, param_3);
    write_usize(data.observed_aura_param, param_1);
    if let Some(owner) = aura_owner(param_1) {
        write_usize(data.observed_aura_owner, owner);
    }
    write_usize(data.observed_effect, effect_field as usize - 0x54);

    if read_u32(data.enabled) == 0 || read_u32(data.force_effect_id) == 0 {
        original(param_1, param_2, param_3);
        return;
    }

    let local_player_fx_owner = read_usize(data.local_player_fx_owner);
    if read_u32(data.local_player_filter) != 0 && local_player_fx_owner != 0 {
        if let Some(owner) = aura_owner(param_1) {
            if owner != local_player_fx_owner {
                original(param_1, param_2, param_3);
                return;
            }
        }
    }

    let forced_effect_id = read_u32(data.effect_id);
    write_u32(data.id_forced_hits, read_u32(data.id_forced_hits).wrapping_add(1));
    ptr::write_unaligned(effect_field, forced_effect_id);
    original(param_1, param_2, param_3);
    ptr::write_unaligned(effect_field, natural_effect_id);
}

unsafe fn aura_owner(param_1: usize) -> Option<usize> {
    if param_1 == 0 {
        return None;
    }
    let owner = ptr::read_unaligned((param_1 + 0x2d8) as *const usize);
    (owner != 0).then_some(owner)
}

unsafe fn aura_effect_id_field(param_1: usize) -> Option<*mut u32> {
    let owner = aura_owner(param_1)?;
    let effect = ptr::read_unaligned((owner + 0x10) as *const usize);
    (effect != 0).then_some((effect + 0x54) as *mut u32)
}

fn append_data(code: &mut Vec<u8>, config: AuraConfig) -> usize {
    let offset = code.len();
    code.extend_from_slice(&(config.enabled as u32).to_le_bytes());
    code.extend_from_slice(&(config.force_effect_id as u32).to_le_bytes());
    code.extend_from_slice(&config.effect_id.to_le_bytes());
    code.extend_from_slice(&((config.target == TargetMode::LocalPlayer) as u32).to_le_bytes());
    code.extend_from_slice(&0u64.to_le_bytes());
    code.extend_from_slice(&0u64.to_le_bytes());
    code.extend_from_slice(&config.animation_speed.to_bits().to_le_bytes());
    code.extend_from_slice(&0.0f32.to_bits().to_le_bytes());
    code.extend_from_slice(&config.loop_start.to_bits().to_le_bytes());
    code.extend_from_slice(&config.loop_end.to_bits().to_le_bytes());
    code.extend_from_slice(&0u32.to_le_bytes());
    code.extend_from_slice(&0u32.to_le_bytes());
    code.extend_from_slice(&0u32.to_le_bytes());
    code.extend_from_slice(&0u32.to_le_bytes());
    code.extend_from_slice(&0u32.to_le_bytes());
    code.extend_from_slice(&0u32.to_le_bytes());
    code.extend_from_slice(&0u64.to_le_bytes());
    code.extend_from_slice(&0u64.to_le_bytes());
    code.extend_from_slice(&0u64.to_le_bytes());
    offset
}

fn data_ptrs(base: usize, offset: usize) -> AuraDataPtrs {
    AuraDataPtrs {
        enabled: base + offset,
        force_effect_id: base + offset + 4,
        effect_id: base + offset + 8,
        local_player_filter: base + offset + 12,
        local_player: base + offset + 16,
        local_player_fx_owner: base + offset + 24,
        speed: base + offset + 32,
        timer: base + offset + 36,
        loop_start: base + offset + 40,
        loop_end: base + offset + 44,
        id_hits: base + offset + 48,
        id_forced_hits: base + offset + 52,
        duration_hits: base + offset + 56,
        duration_match_hits: base + offset + 60,
        observed_effect_id: base + offset + 64,
        observed_edx: base + offset + 68,
        observed_aura_param: base + offset + 72,
        observed_aura_owner: base + offset + 80,
        observed_effect: base + offset + 88,
    }
}

fn write_u32(address: usize, value: u32) {
    unsafe { *(address as *mut u32) = value };
}

fn read_u32(address: usize) -> u32 {
    unsafe { *(address as *const u32) }
}

fn read_usize(address: usize) -> usize {
    unsafe { *(address as *const usize) }
}

fn write_usize(address: usize, value: usize) {
    unsafe { *(address as *mut usize) = value };
}

unsafe fn read_unaligned_u32(address: usize) -> u32 {
    ptr::read_unaligned(address as *const u32)
}

unsafe fn read_unaligned_usize(address: usize) -> usize {
    ptr::read_unaligned(address as *const usize)
}

fn write_f32(address: usize, value: f32) {
    unsafe { *(address as *mut f32) = value };
}

fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

fn emit_abs_jmp(code: &mut Vec<u8>, target: usize) {
    code.extend_from_slice(&[0x48, 0xb8]);
    code.extend_from_slice(&(target as u64).to_le_bytes());
    code.extend_from_slice(&[0xff, 0xe0]);
}

fn emit_cmp_rip_u32(code: &mut Vec<u8>) -> usize {
    code.extend_from_slice(&[0x83, 0x3d]);
    let disp = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    code.push(0x00);
    disp
}

fn emit_inc_rip_u32(code: &mut Vec<u8>) -> usize {
    code.extend_from_slice(&[0xff, 0x05]);
    let disp = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    disp
}

fn emit_cmp_edx_rip(code: &mut Vec<u8>) -> usize {
    code.extend_from_slice(&[0x3b, 0x15]);
    let disp = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    disp
}

fn emit_movss_xmm2_rip(code: &mut Vec<u8>) -> usize {
    code.extend_from_slice(&[0xf3, 0x0f, 0x10, 0x15]);
    let disp = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    disp
}

fn emit_movss_xmm0_rip(code: &mut Vec<u8>) -> usize {
    code.extend_from_slice(&[0xf3, 0x0f, 0x10, 0x05]);
    let disp = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    disp
}

fn emit_movss_rip_xmm2(code: &mut Vec<u8>) -> usize {
    code.extend_from_slice(&[0xf3, 0x0f, 0x11, 0x15]);
    let disp = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    disp
}

fn emit_movss_rip_xmm0(code: &mut Vec<u8>) -> usize {
    code.extend_from_slice(&[0xf3, 0x0f, 0x11, 0x05]);
    let disp = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    disp
}

fn emit_mov_dword_rdi_disp_imm(code: &mut Vec<u8>, disp: u32, value: f32) {
    code.extend_from_slice(&[0xc7, 0x87]);
    code.extend_from_slice(&disp.to_le_bytes());
    code.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn emit_jmp(code: &mut Vec<u8>) -> Rel32Patch {
    let instruction_offset = code.len();
    code.push(0xe9);
    let immediate_offset = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    Rel32Patch {
        instruction_offset,
        immediate_offset,
        instruction_len: 5,
    }
}

fn emit_jz(code: &mut Vec<u8>) -> Rel32Patch {
    let instruction_offset = code.len();
    code.extend_from_slice(&[0x0f, 0x84]);
    let immediate_offset = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    Rel32Patch {
        instruction_offset,
        immediate_offset,
        instruction_len: 6,
    }
}

fn emit_jne(code: &mut Vec<u8>) -> Rel32Patch {
    let instruction_offset = code.len();
    code.extend_from_slice(&[0x0f, 0x85]);
    let immediate_offset = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    Rel32Patch {
        instruction_offset,
        immediate_offset,
        instruction_len: 6,
    }
}

fn emit_jbe(code: &mut Vec<u8>) -> Rel32Patch {
    let instruction_offset = code.len();
    code.extend_from_slice(&[0x0f, 0x86]);
    let immediate_offset = code.len();
    code.extend_from_slice(&0i32.to_le_bytes());
    Rel32Patch {
        instruction_offset,
        immediate_offset,
        instruction_len: 6,
    }
}

fn patch_disp32_vec(
    code: &mut [u8],
    base: usize,
    disp_offset: usize,
    target: usize,
) -> Result<(), String> {
    let instruction_end = base + disp_offset + 4;
    let disp = checked_i32(target as isize - instruction_end as isize, "rip displacement")?;
    write_i32(code, disp_offset, disp)
}

fn patch_rel32_vec(
    code: &mut [u8],
    base: usize,
    patch: Rel32Patch,
    target: usize,
) -> Result<(), String> {
    let rel = rel32(
        base + patch.instruction_offset,
        patch.instruction_len,
        target,
    )?;
    write_i32(code, patch.immediate_offset, rel)
}

fn rel32(source: usize, instruction_len: usize, target: usize) -> Result<i32, String> {
    let rel = target as isize - (source + instruction_len) as isize;
    checked_i32(
        rel,
        &format!("jump target out of range source=0x{source:x} target=0x{target:x}"),
    )
}

fn checked_i32(value: isize, label: &str) -> Result<i32, String> {
    i32::try_from(value).map_err(|_| format!("{label}: value out of i32 range: {value}"))
}

fn write_i32(code: &mut [u8], offset: usize, value: i32) -> Result<(), String> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| "i32 patch offset overflow".to_string())?;
    let Some(slot) = code.get_mut(offset..end) else {
        return Err(format!(
            "i32 patch out of range offset=0x{offset:x} len=0x{:x}",
            code.len()
        ));
    };
    slot.copy_from_slice(&value.to_le_bytes());
    Ok(())
}
