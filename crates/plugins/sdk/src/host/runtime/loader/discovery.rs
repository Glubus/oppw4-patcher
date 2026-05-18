use std::{fs, path::Path};

use crate::host::log;

use super::{paths::mods_root, plugin::load_plugin};
use crate::host::runtime::manifest::PluginManifest;

pub(super) fn load_plugins(game_root: &Path, plugin_root: &Path) -> usize {
    let Some(entries) = plugin_dirs(plugin_root) else {
        return 0;
    };

    entries
        .filter_map(plugin_manifest)
        .filter(|manifest| unsafe { load_plugin(game_root, &mods_root(game_root), manifest) })
        .count()
}

fn plugin_dirs(plugin_root: &Path) -> Option<impl Iterator<Item = std::path::PathBuf>> {
    let entries = match fs::read_dir(plugin_root) {
        Ok(entries) => entries,
        Err(_) => {
            log::write_line(format!(
                "plugin host: plugin dir not readable path={}",
                plugin_root.display()
            ));
            return None;
        }
    };

    Some(
        entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir()),
    )
}

fn plugin_manifest(plugin_dir: std::path::PathBuf) -> Option<PluginManifest> {
    PluginManifest::read_from_dir(&plugin_dir)
}
