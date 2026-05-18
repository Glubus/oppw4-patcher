use crate::abi::Oppw4LuaModule;

use crate::host::runtime::lua;

mod r#unsafe;

pub(crate) use r#unsafe::host_register_lua_module;

fn register_lua_module(module: *const Oppw4LuaModule) -> i32 {
    unsafe { lua::register_module(module) }
}
