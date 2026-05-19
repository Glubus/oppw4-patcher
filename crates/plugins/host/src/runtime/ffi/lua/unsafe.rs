use std::ffi::c_void;

use plugin_abi::Oppw4LuaModule;

pub(crate) unsafe extern "system" fn host_register_lua_module(
    _host_context: *mut c_void,
    module: *const Oppw4LuaModule,
) -> i32 {
    super::register_lua_module(module)
}
