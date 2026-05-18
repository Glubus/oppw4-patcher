use std::sync::OnceLock;

use plugin_sdk::{HostApi, OwnedHostApi};

const PLUGIN_ID: &str = "fx_director";

#[derive(Clone, Copy)]
struct Logger {
    host: OwnedHostApi,
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn initialize(host: HostApi<'_>) {
    let _ = LOGGER.set(Logger {
        host: OwnedHostApi::from(*host.abi()),
    });
}

pub fn write_line(message: impl AsRef<str>) {
    let Some(logger) = LOGGER.get().copied() else {
        return;
    };
    let _ = logger.host.log().write(PLUGIN_ID, message);
}
