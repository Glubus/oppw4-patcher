// crates/oppw4-logging/src/timestamp.rs
use std::ffi::c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SystemTimeParts {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetLocalTime(system_time: *mut c_void);
}

pub(crate) fn current_log_file_name() -> String {
    format_log_file_name(local_time_parts())
}

fn local_time_parts() -> SystemTimeParts {
    let mut parts = SystemTimeParts {
        year: 1970,
        month: 1,
        day_of_week: 0,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe {
        GetLocalTime((&mut parts as *mut SystemTimeParts).cast());
    }
    parts
}

fn format_log_file_name(parts: SystemTimeParts) -> String {
    format!(
        "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.log",
        parts.year, parts.month, parts.day, parts.hour, parts.minute, parts.second
    )
}

#[cfg(test)]
mod tests {
    use crate::timestamp::{format_log_file_name, SystemTimeParts};

    #[test]
    fn log_file_name_uses_local_date_and_time() {
        let file_name = format_log_file_name(SystemTimeParts {
            year: 2026,
            month: 5,
            day_of_week: 0,
            day: 10,
            hour: 8,
            minute: 26,
            second: 43,
            milliseconds: 852,
        });

        assert_eq!(file_name, "2026-05-10_08-26-43.log");
    }
}
