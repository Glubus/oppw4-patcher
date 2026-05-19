use std::sync::OnceLock;

use plugin_sdk::{HostApi, OwnedHostApi};

use crate::constants::PLUGIN_ID;

static HOST: OnceLock<OwnedHostApi> = OnceLock::new();

pub(crate) fn init(host: HostApi<'_>) {
    let _ = HOST.set(OwnedHostApi::new(*host.abi()));
}

pub(crate) fn write(host: HostApi<'_>, message: impl Into<String>) {
    let _ = host.log().write(PLUGIN_ID, message.into());
}

pub(crate) fn write_global(message: impl Into<String>) {
    if let Some(host) = HOST.get() {
        let _ = host.log().write(PLUGIN_ID, message.into());
    }
}
