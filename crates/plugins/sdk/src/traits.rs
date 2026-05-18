use std::ffi::CStr;

use crate::{Oppw4FileProvider, Oppw4LuaModule};

pub trait FileProvider {
    fn raw_provider(&self, plugin_id: &CStr) -> Oppw4FileProvider;
}

pub trait LuaModule {
    fn raw_module(&self, plugin_id: &CStr) -> Oppw4LuaModule;
}
