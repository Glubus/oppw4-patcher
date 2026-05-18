use std::{fs, path::Path};

use crate::host::log;

use super::{paths::mods_root, plugin::load_plugin};
use crate::host::runtime::manifest::PluginManifest;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PluginLoadReport {
    pub(super) scanned: usize,
    pub(super) manifests: usize,
    pub(super) loaded: usize,
}

pub(super) fn load_plugins(game_root: &Path, plugin_root: &Path) -> PluginLoadReport {
    let Some(entries) = plugin_dirs(plugin_root) else {
        return PluginLoadReport::default();
    };

    let mut report = PluginLoadReport {
        scanned: entries.len(),
        ..PluginLoadReport::default()
    };
    for plugin_dir in entries {
        let Some(manifest) = plugin_manifest(plugin_dir) else {
            continue;
        };
        report.manifests += 1;
        if unsafe { load_plugin(game_root, &mods_root(game_root), &manifest) } {
            report.loaded += 1;
        }
    }
    report
}

fn plugin_dirs(plugin_root: &Path) -> Option<Vec<std::path::PathBuf>> {
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

    let mut dirs = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    dirs.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
    Some(dirs)
}

fn plugin_manifest(plugin_dir: std::path::PathBuf) -> Option<PluginManifest> {
    PluginManifest::read_from_dir(&plugin_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn plugin_dirs_are_sorted_for_stable_load_order() {
        let root = temp_root("plugin-dir-order");
        fs::create_dir_all(root.join("z_plugin")).expect("z");
        fs::create_dir_all(root.join("a_plugin")).expect("a");
        fs::write(root.join("loose.dll"), []).expect("file");

        let dirs = plugin_dirs(&root).expect("dirs");

        assert_eq!(dirs, vec![root.join("a_plugin"), root.join("z_plugin")]);
        let _ = fs::remove_dir_all(root);
    }

    fn temp_root(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("oppw4-{label}-{nanos}"))
    }
}
