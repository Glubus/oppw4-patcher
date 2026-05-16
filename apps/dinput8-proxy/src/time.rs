#[repr(C)]
struct SystemTimeParts {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

extern "system" {
    fn GetLocalTime(time: *mut SystemTimeParts);
}

pub fn file_timestamp() -> String {
    let time = local_time();
    format!(
        "{:04}-{:02}-{:02}-{:02}{:02}{:02}",
        time.year, time.month, time.day, time.hour, time.minute, time.second
    )
}

fn local_time() -> SystemTimeParts {
    let mut time = SystemTimeParts {
        year: 1970,
        month: 1,
        day_of_week: 0,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe { GetLocalTime(&mut time) };
    time
}
