mod hot_reload;
mod module;
mod runner;
mod state;

use std::{
    path::Path,
    sync::{Mutex, OnceLock},
};

use crate::abi::{optional_cstr, Oppw4LuaModule};

use crate::host::log;

use self::{hot_reload::start_hot_reload_worker, module::RegisteredModule, state::LuaHost};

static HOST: OnceLock<Mutex<LuaHost>> = OnceLock::new();

pub(crate) fn initialize(mods_root: &Path) {
    let host = HOST.get_or_init(|| Mutex::new(LuaHost::default()));
    let mut host = host.lock().expect("lua host lock");
    host.reset(mods_root);
    if host.start_hot_reload() {
        start_hot_reload_worker(mods_root.to_path_buf());
    }
}

pub(crate) unsafe fn register_module(module: *const Oppw4LuaModule) -> i32 {
    let Some(module) = module.as_ref() else {
        return -1;
    };
    let Some(plugin_id) = optional_cstr(module.plugin_id) else {
        return -2;
    };
    let Some(module_name) = optional_cstr(module.module_name) else {
        return -3;
    };
    let Some(register) = module.register else {
        return -4;
    };
    let Some(host) = HOST.get() else {
        return -5;
    };
    let mut host = host.lock().expect("lua host lock");
    let entry = RegisteredModule {
        plugin_id: plugin_id.to_string_lossy().into_owned(),
        module_name: module_name.to_string_lossy().into_owned(),
        context: module.module_context as usize,
        register,
    };
    log::write_line(format!(
        "lua host: registered module plugin={} module={}",
        entry.plugin_id, entry.module_name
    ));
    host.register_module(entry);
    host.run_ready_mods();
    0
}

fn with_host(action: impl FnOnce(&mut LuaHost)) {
    let Some(host) = HOST.get() else {
        return;
    };
    action(&mut host.lock().expect("lua host lock"));
}
