use std::ffi::c_void;

use plugin_abi::{optional_cstr, Oppw4FileProvider};

pub(crate) unsafe extern "system" fn host_register_file_provider(
    _host_context: *mut c_void,
    provider: *const Oppw4FileProvider,
) -> i32 {
    let Some(provider) = provider.as_ref() else {
        return -1;
    };
    super::register_file_provider(provider, optional_cstr(provider.plugin_id))
}
