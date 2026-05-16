use std::{
    cell::Cell,
    mem::size_of,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use crate::{changes::internal_hook, log, win};

const FILE_JOB_EXECUTE_RVA: usize = 0x0a76830;
const FILE_JOB_EXECUTE_STOLEN_LEN: usize = 15;

type FileJobExecuteFn = unsafe extern "system" fn(usize, usize) -> u64;

static INSTALLED: AtomicBool = AtomicBool::new(false);
static FILE_JOB_EXECUTE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static HASH_OPEN_DIAG_LOGS: AtomicUsize = AtomicUsize::new(0);
static PREVIOUS_FILE_STATE: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static CURRENT_FILE_JOB: Cell<usize> = const { Cell::new(0) };
}

pub(crate) fn install() {
    if INSTALLED.swap(true, Ordering::AcqRel) {
        return;
    }

    let module = win::main_module() as usize;
    if module == 0 {
        log::write_line("file-job-diagnostic install skipped reason=main_module_missing");
        return;
    }

    let installed = unsafe {
        internal_hook::install_absolute_jump_hook(
            module,
            FILE_JOB_EXECUTE_RVA,
            FILE_JOB_EXECUTE_STOLEN_LEN,
            hooked_file_job_execute as usize,
            &FILE_JOB_EXECUTE_ORIGINAL,
            "file job execute diagnostic",
        )
    };
    log::write_line(format!("file-job-diagnostic hooks installed={installed}"));
}

pub(crate) fn current_job_summary() -> Option<String> {
    let job = CURRENT_FILE_JOB.with(Cell::get);
    if job == 0 {
        return None;
    }
    Some(format_job_summary(job, false))
}

pub(crate) fn current_hash_open_summary() -> Option<String> {
    let job = CURRENT_FILE_JOB.with(Cell::get);
    if job == 0 {
        return None;
    }
    let index = HASH_OPEN_DIAG_LOGS.fetch_add(1, Ordering::Relaxed);
    Some(format_job_summary(job, index < 15))
}

unsafe extern "system" fn hooked_file_job_execute(manager: usize, job: usize) -> u64 {
    let Some(original) = original_file_job_execute() else {
        return 0;
    };
    let previous = CURRENT_FILE_JOB.with(|current| {
        let previous = current.get();
        current.set(job);
        previous
    });
    let result = original(manager, job);
    CURRENT_FILE_JOB.with(|current| current.set(previous));
    result
}

fn original_file_job_execute() -> Option<FileJobExecuteFn> {
    let address = FILE_JOB_EXECUTE_ORIGINAL.load(Ordering::Acquire);
    (address != 0).then(|| unsafe { std::mem::transmute(address) })
}

fn format_job_summary(job: usize, verbose: bool) -> String {
    let op = read_u32(job.saturating_add(0x10)).unwrap_or(u32::MAX);
    let state = read_u32(job.saturating_add(0x14)).unwrap_or(u32::MAX);
    let queue = read_u32(job.saturating_add(0x20)).unwrap_or(u32::MAX);
    let path_owner = read_usize(job.saturating_add(0x18)).unwrap_or_default();
    let file_state = read_usize(job.saturating_add(0x30)).unwrap_or_default();
    let size_low = read_u64(job.saturating_add(0x40)).unwrap_or_default();
    let output = read_usize(job.saturating_add(0x58)).unwrap_or_default();
    let path = path_owner
        .checked_add(0x204)
        .and_then(|address| read_wide_string(address, 260))
        .unwrap_or_else(|| "?".to_string());
    let mut summary = format!(
        "job=0x{job:x} op={op} state={state} queue={queue} path=\"{path}\" path_owner=0x{path_owner:x} file_state=0x{file_state:x} size=0x{size_low:x} output=0x{output:x}"
    );
    if verbose {
        let previous = PREVIOUS_FILE_STATE.swap(file_state, Ordering::Relaxed);
        summary.push_str(&format!(
            " prev_file_state=0x{previous:x} wide_candidates={}",
            format_wide_candidates(path_owner)
        ));
        summary.push_str(&format!(
            " job_dump={}",
            format_memory_dump(job.saturating_add(0x18), 0x40)
        ));
        summary.push_str(&format!(
            " path_owner_dump={}",
            format_memory_dump(path_owner, 0x80)
        ));
        summary.push_str(&format!(
            " file_state_dump={}",
            format_memory_dump(file_state, 0x100)
        ));
    }
    summary
}

fn read_exact(address: usize, buffer: &mut [u8]) -> bool {
    if address == 0 || buffer.is_empty() {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(address as *const u8, buffer.as_mut_ptr(), buffer.len());
    }
    true
}

fn read_usize(address: usize) -> Option<usize> {
    let mut bytes = [0u8; size_of::<usize>()];
    read_exact(address, &mut bytes).then(|| usize::from_le_bytes(bytes))
}

fn read_u64(address: usize) -> Option<u64> {
    let mut bytes = [0u8; size_of::<u64>()];
    read_exact(address, &mut bytes).then(|| u64::from_le_bytes(bytes))
}

fn read_u32(address: usize) -> Option<u32> {
    let mut bytes = [0u8; size_of::<u32>()];
    read_exact(address, &mut bytes).then(|| u32::from_le_bytes(bytes))
}

fn read_wide_string(address: usize, max_units: usize) -> Option<String> {
    if address == 0 || max_units == 0 {
        return None;
    }
    let mut units = Vec::new();
    for index in 0..max_units {
        let mut bytes = [0u8; size_of::<u16>()];
        if !read_exact(
            address.checked_add(index.checked_mul(size_of::<u16>())?)?,
            &mut bytes,
        ) {
            return None;
        }
        let unit = u16::from_le_bytes(bytes);
        if unit == 0 {
            break;
        }
        units.push(unit);
    }
    Some(String::from_utf16_lossy(&units))
}

fn format_wide_candidates(base: usize) -> String {
    if base == 0 {
        return "[]".to_string();
    }

    let mut candidates = Vec::new();
    for offset in [
        0x00usize, 0x04, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x40, 0x80, 0x100, 0x180, 0x200,
        0x204, 0x208, 0x220, 0x240,
    ] {
        let Some(address) = base.checked_add(offset) else {
            continue;
        };
        let Some(text) = read_wide_string(address, 160) else {
            continue;
        };
        if !is_candidate_string(&text) {
            continue;
        }
        candidates.push(format!("+0x{offset:x}:\"{}\"", escape_log_text(&text)));
        if candidates.len() >= 8 {
            break;
        }
    }
    if candidates.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", candidates.join(","))
    }
}

fn is_candidate_string(text: &str) -> bool {
    if text.is_empty() || text.len() > 260 {
        return false;
    }
    let printable = text
        .chars()
        .filter(|ch| ch.is_ascii_graphic() || ch.is_ascii_whitespace())
        .count();
    printable == text.chars().count()
        && (text.len() >= 3
            || text.contains(':')
            || text.contains('\\')
            || text.contains('/')
            || text.contains("0x"))
}

fn escape_log_text(text: &str) -> String {
    text.chars()
        .take(120)
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn format_memory_dump(address: usize, len: usize) -> String {
    if address == 0 || len == 0 {
        return "none".to_string();
    }
    let mut bytes = vec![0u8; len];
    if !read_exact(address, &mut bytes) {
        return "read_failed".to_string();
    }
    hex_preview(&bytes)
}

fn hex_preview(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0f));
    }
    out
}

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'a' + (nibble - 10)) as char,
    }
}
