use oppw4_plugin_api::{cstring_lossy, Oppw4PluginApi, OPPW4_PLUGIN_API_VERSION};

const LEGACY_NAME_HASH_CATALOG_ZIP: &[u8] =
    include_bytes!("../../../resources/name_hash_catalog.zip");

#[no_mangle]
pub unsafe extern "system" fn oppw4_plugin_init(api: *const Oppw4PluginApi) -> i32 {
    let Some(api) = api.as_ref() else {
        return -1;
    };
    if api.version != OPPW4_PLUGIN_API_VERSION {
        return -2;
    }

    let plugin_id = cstring_lossy("skin_patcher");
    let message = cstring_lossy(format!(
        "skin_patcher plugin initialized legacy_hash_catalog_zip_bytes={}",
        LEGACY_NAME_HASH_CATALOG_ZIP.len()
    ));
    api.log_line(&plugin_id, &message);
    0
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
