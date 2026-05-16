use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        OnceLock,
    },
    time::Instant,
};

use oppw4_plugin_api::{
    Oppw4GameStatus, OPPW4_GAME_FLAG_DLC_CHARACTER_SEEN,
    OPPW4_GAME_FLAG_VIRTUAL_RESOURCE_SEEN, OPPW4_GAME_PHASE_BOOTING,
    OPPW4_GAME_PHASE_DLC_CHARACTER_LOADING, OPPW4_GAME_PHASE_RDB_BIN_LOADING,
    OPPW4_GAME_PHASE_RDB_LOADING, OPPW4_GAME_PHASE_VIRTUAL_RESOURCE_LOADING,
};

use crate::log;

static START: OnceLock<Instant> = OnceLock::new();
static PHASE: AtomicU32 = AtomicU32::new(OPPW4_GAME_PHASE_BOOTING);
static FLAGS: AtomicU32 = AtomicU32::new(0);
static FILE_OPENS: AtomicU32 = AtomicU32::new(0);
static DLC_MARKERS: AtomicU32 = AtomicU32::new(0);
static VIRTUAL_MARKERS: AtomicU32 = AtomicU32::new(0);

pub fn mark_file_open(path: &str) {
    START.get_or_init(Instant::now);
    FILE_OPENS.fetch_add(1, Ordering::Relaxed);

    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rdb.bin") || lower.contains(".rdb.bin") {
        advance_phase(OPPW4_GAME_PHASE_RDB_BIN_LOADING, path);
    } else if lower.contains("file\\dlc\\dlc_character_") {
        mark_event(
            OPPW4_GAME_FLAG_DLC_CHARACTER_SEEN,
            &DLC_MARKERS,
            "dlc_character_seen",
            path,
        );
    } else if lower.contains("data\\0x") && lower.ends_with(".file") {
        mark_event(
            OPPW4_GAME_FLAG_VIRTUAL_RESOURCE_SEEN,
            &VIRTUAL_MARKERS,
            "virtual_resource_seen",
            path,
        );
    } else if lower.ends_with(".rdb") {
        advance_phase(OPPW4_GAME_PHASE_RDB_LOADING, path);
    }
}

pub fn game_status() -> Oppw4GameStatus {
    let start = START.get_or_init(Instant::now);
    Oppw4GameStatus {
        phase: PHASE.load(Ordering::Relaxed),
        flags: FLAGS.load(Ordering::Relaxed),
        observed_file_opens: FILE_OPENS.load(Ordering::Relaxed),
        seconds_since_host_start: start.elapsed().as_secs().min(u32::MAX as u64) as u32,
    }
}

fn mark_event(flag: u32, counter: &AtomicU32, marker: &str, path: &str) {
    FLAGS.fetch_or(flag, Ordering::Relaxed);
    let index = counter.fetch_add(1, Ordering::Relaxed);
    if index < 12 {
        log::write_line(format!(
            "game_status marker {marker} count={} phase={} flags=0x{:x} after {} file opens path={}",
            index + 1,
            phase_name(PHASE.load(Ordering::Relaxed)),
            FLAGS.load(Ordering::Relaxed),
            FILE_OPENS.load(Ordering::Relaxed),
            path
        ));
    } else if index == 12 {
        log::write_line(format!("game_status marker {marker} logs suppressed"));
    }
}

fn advance_phase(next: u32, path: &str) {
    let mut current = PHASE.load(Ordering::Relaxed);
    while next > current {
        match PHASE.compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {
                log::write_line(format!(
                    "game_status phase {} -> {} after {} file opens path={}",
                    phase_name(current),
                    phase_name(next),
                    FILE_OPENS.load(Ordering::Relaxed),
                    path
                ));
                return;
            }
            Err(actual) => current = actual,
        }
    }
}

fn phase_name(phase: u32) -> &'static str {
    match phase {
        OPPW4_GAME_PHASE_BOOTING => "booting",
        OPPW4_GAME_PHASE_RDB_LOADING => "rdb_loading",
        OPPW4_GAME_PHASE_RDB_BIN_LOADING => "rdb_bin_loading",
        OPPW4_GAME_PHASE_DLC_CHARACTER_LOADING => "dlc_character_loading",
        OPPW4_GAME_PHASE_VIRTUAL_RESOURCE_LOADING => "virtual_resource_loading",
        _ => "unknown",
    }
}
