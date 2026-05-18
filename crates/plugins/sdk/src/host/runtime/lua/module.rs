use std::ffi::c_void;

use crate::abi::Oppw4LuaRegisterFn;
use mlua::Lua;

#[derive(Clone)]
pub(super) struct RegisteredModule {
    pub(super) plugin_id: String,
    pub(super) module_name: String,
    pub(super) context: usize,
    pub(super) register: Oppw4LuaRegisterFn,
}

pub(super) fn register_plugin_module(lua: &Lua, module: &RegisteredModule) -> mlua::Result<()> {
    let result = unsafe {
        (module.register)(
            module.context as *mut c_void,
            (lua as *const Lua).cast_mut().cast(),
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(mlua::Error::external(format!(
            "lua module register failed plugin={} module={} result={result}",
            module.plugin_id, module.module_name
        )))
    }
}
