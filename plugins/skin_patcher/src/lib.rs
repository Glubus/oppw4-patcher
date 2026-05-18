use plugin_sdk::{plugin_abi_from_raw, HostApi, Oppw4PluginApi};

mod ffi;
mod log;
mod lua;
mod mods;
mod patching;
mod rdb_tracker;
mod runtime;

pub(crate) const LEGACY_NAME_HASH_CATALOG_ZIP: &[u8] =
    include_bytes!("../../../resources/name_hash_catalog.zip");

#[no_mangle]
pub unsafe extern "system" fn oppw4_plugin_init(api: *const Oppw4PluginApi) -> i32 {
    let api = match plugin_abi_from_raw(api) {
        Ok(api) => api,
        Err(error) => return error.code(),
    };

    let host = HostApi::from(api);
    log::initialize(host);
    log::write_line(format!(
        "skin_patcher plugin initialized legacy_hash_catalog_zip_bytes={}",
        LEGACY_NAME_HASH_CATALOG_ZIP.len()
    ));
    lua::register(host);
    runtime::initialize(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_legacy_name_hash_catalog() {
        assert!(LEGACY_NAME_HASH_CATALOG_ZIP.len() > 0x1000);
        assert_eq!(&LEGACY_NAME_HASH_CATALOG_ZIP[..2], b"PK");
    }
}
