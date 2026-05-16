use std::{ffi::c_void, path::PathBuf, sync::Once};

use crate::{log, plugin_host, win};

static INIT: Once = Once::new();

pub fn initialize_once(module: *mut c_void) {
    INIT.call_once(|| initialize_loader(module));
}

fn initialize_loader(module: *mut c_void) {
    if let Some(base_dir) = win::module_directory(module) {
        log::initialize(base_dir.clone());
        initialize_loader_thread(base_dir);
    }
    log::write_line("loader init placeholder reached");
}

fn initialize_loader_thread(base_dir: PathBuf) {
    log::write_line("loader init started");
    oppw4_hooks::set_logger(write_hook_log);
    oppw4_hooks::install_main_module_hooks();
    let paths = LoaderPaths::from_base_dir(base_dir);
    log_loader_paths(&paths);
    plugin_host::initialize(&paths.game_root, &paths.plugin_root);
}

fn write_hook_log(message: String) {
    log::write_line(message);
}

struct LoaderPaths {
    game_root: PathBuf,
    plugin_root: PathBuf,
}

impl LoaderPaths {
    fn from_base_dir(base_dir: PathBuf) -> Self {
        Self {
            game_root: base_dir.clone(),
            plugin_root: base_dir.join("plugins"),
        }
    }
}

fn log_loader_paths(paths: &LoaderPaths) {
    log::write_line(format!("game root: {}", paths.game_root.display()));
    log::write_line(format!("plugin root: {}", paths.plugin_root.display()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_paths_use_root_plugins_folder() {
        let game_root = PathBuf::from(r"D:\Game\OPPW4");
        let paths = LoaderPaths::from_base_dir(game_root.clone());

        assert_eq!(paths.game_root, game_root);
        assert_eq!(paths.plugin_root, PathBuf::from(r"D:\Game\OPPW4\plugins"));
    }
}
