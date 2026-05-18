use crate::{
    abi::{Oppw4LuaModule, Oppw4PluginApi},
    error::PluginError,
    host_api::r#unsafe,
    PluginResult,
};

#[derive(Clone, Copy)]
pub struct LuaService<'api> {
    abi: &'api Oppw4PluginApi,
}

impl<'api> LuaService<'api> {
    pub(super) const fn new(abi: &'api Oppw4PluginApi) -> Self {
        Self { abi }
    }

    pub fn register_module(self, module: &Oppw4LuaModule) -> PluginResult<()> {
        let register = self
            .abi
            .register_lua_module
            .ok_or(PluginError::MissingHostFunction("register_lua_module"))?;
        let code = r#unsafe::register_lua_module(self.abi.host_context, register, module);
        if code == 0 {
            Ok(())
        } else {
            Err(PluginError::HostCallFailed {
                operation: "register_lua_module",
                code,
            })
        }
    }
}
