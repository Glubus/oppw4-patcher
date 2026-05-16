use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use oppw4_plugin_api::{PluginInitFn, OPPW4_PLUGIN_INIT_SYMBOL};

use crate::log;

use super::{ffi, logs, manifest::PluginManifest, win};

static LOADED: OnceLock<Mutex<Vec<LoadedPlugin>>> = OnceLock::new();

pub fn initialize(game_root: &Path, plugin_root: &Path) {
    let _ = fs::create_dir_all(plugin_root);
    logs::initialize();
    let _ = LOADED.set(Mutex::new(Vec::new()));

    load_plugins(game_root, plugin_root);
}

fn load_plugins(game_root: &Path, plugin_root: &Path) {
    let Ok(entries) = fs::read_dir(plugin_root) else {
        log::write_line(format!(
            "plugin host: plugin dir not readable path={}",
            plugin_root.display()
        ));
        return;
    };

    let mut loaded = 0usize;
    for entry in entries.flatten() {
        let plugin_dir = entry.path();
        if !plugin_dir.is_dir() {
            continue;
        }
        let Some(manifest) = PluginManifest::read_from_dir(&plugin_dir) else {
            continue;
        };
        if unsafe { load_plugin(game_root, &manifest) } {
            loaded += 1;
        }
    }
    log::write_line(format!("plugin host: loaded={loaded}"));
}

unsafe fn load_plugin(game_root: &Path, manifest: &PluginManifest) -> bool {
    logs::register(manifest.id.clone(), manifest.log_root.clone());
    let _ = fs::create_dir_all(&manifest.mods_root);

    let wide = path_to_wide(&manifest.entry_path);
    let module = win::load_library(&wide);
    if module.is_null() {
        log::write_line(format!(
            "plugin host: load failed id={} path={}",
            manifest.id,
            manifest.entry_path.display()
        ));
        return false;
    }

    let proc = win::get_proc_address(module, OPPW4_PLUGIN_INIT_SYMBOL.as_ptr().cast());
    if proc.is_null() {
        log::write_line(format!(
            "plugin host: init symbol missing id={} path={}",
            manifest.id,
            manifest.entry_path.display()
        ));
        return false;
    }

    let init: PluginInitFn = std::mem::transmute(proc);
    let game_root_utf8 = ffi::cstring_lossy(&game_root.to_string_lossy());
    let plugin_root_utf8 = ffi::cstring_lossy(&manifest.root.to_string_lossy());
    let plugin_mods_root_utf8 = ffi::cstring_lossy(&manifest.mods_root.to_string_lossy());
    let api = ffi::build_api(
        game_root,
        &game_root_utf8,
        &plugin_root_utf8,
        &plugin_mods_root_utf8,
    );
    let result = init(&api);
    if result != 0 {
        log::write_line(format!(
            "plugin host: init failed id={} path={} result={result}",
            manifest.id,
            manifest.entry_path.display()
        ));
        return false;
    }

    if let Some(plugins) = LOADED.get() {
        plugins
            .lock()
            .expect("plugin list lock")
            .push(LoadedPlugin {
                _id: manifest.id.clone(),
                _path: manifest.entry_path.clone(),
                _module: module as usize,
            });
    }
    log::write_line(format!(
        "plugin host: initialized id={} path={}",
        manifest.id,
        manifest.entry_path.display()
    ));
    true
}

fn path_to_wide(path: &Path) -> Vec<u16> {
    let mut wide = path.to_string_lossy().encode_utf16().collect::<Vec<_>>();
    wide.push(0);
    wide
}

struct LoadedPlugin {
    _id: String,
    _path: PathBuf,
    _module: usize,
}
