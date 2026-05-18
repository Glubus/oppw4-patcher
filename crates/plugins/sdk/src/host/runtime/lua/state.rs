use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use super::{
    hot_reload::{mod_fingerprint, ModFingerprint},
    module::RegisteredModule,
    runner::{run_mod, ModRunReason},
};

#[derive(Default)]
pub(super) struct LuaHost {
    mods_root: PathBuf,
    modules: Vec<RegisteredModule>,
    executed: HashSet<String>,
    fingerprints: HashMap<String, ModFingerprint>,
    last_reload_attempts: HashMap<String, Instant>,
    hot_reload_started: bool,
}

impl LuaHost {
    pub(super) fn reset(&mut self, mods_root: &Path) {
        self.mods_root = mods_root.to_path_buf();
        self.modules.clear();
        self.executed.clear();
        self.fingerprints.clear();
        self.last_reload_attempts.clear();
    }

    pub(super) fn mods_root(&self) -> PathBuf {
        self.mods_root.clone()
    }

    pub(super) fn start_hot_reload(&mut self) -> bool {
        if self.hot_reload_started {
            return false;
        }
        self.hot_reload_started = true;
        true
    }

    pub(super) fn register_module(&mut self, entry: RegisteredModule) {
        self.modules.retain(|existing| {
            !(existing.plugin_id.eq_ignore_ascii_case(&entry.plugin_id)
                && existing
                    .module_name
                    .eq_ignore_ascii_case(&entry.module_name))
        });
        self.modules.push(entry);
    }

    pub(super) fn run_ready_mods(&mut self) {
        for mod_entry in lua_api::discover_mods(&self.mods_root) {
            if self.executed.contains(&mod_entry.manifest.id) {
                continue;
            }
            if !self.mod_dependencies_available(&mod_entry.manifest.uses_plugins) {
                continue;
            }
            if self.run_mod(&mod_entry, ModRunReason::Initial) {
                self.executed.insert(mod_entry.manifest.id.clone());
                if let Some(fingerprint) = mod_fingerprint(&mod_entry) {
                    self.fingerprints
                        .insert(mod_entry.manifest.id.clone(), fingerprint);
                }
            }
        }
    }

    pub(super) fn reload_changed_directory_mods(&mut self) {
        for mod_entry in lua_api::discover_mods(&self.mods_root) {
            if !matches!(mod_entry.source, lua_api::ModSource::Directory(_)) {
                continue;
            }
            if !self.executed.contains(&mod_entry.manifest.id) {
                continue;
            }
            if !self.mod_dependencies_available(&mod_entry.manifest.uses_plugins) {
                continue;
            }
            let Some(fingerprint) = mod_fingerprint(&mod_entry) else {
                continue;
            };
            if self
                .fingerprints
                .get(&mod_entry.manifest.id)
                .is_some_and(|known| *known == fingerprint)
            {
                continue;
            }
            if !self.reload_debounce_elapsed(&mod_entry.manifest.id) {
                continue;
            }
            if self.run_mod(&mod_entry, ModRunReason::HotReload) {
                self.fingerprints
                    .insert(mod_entry.manifest.id.clone(), fingerprint);
            }
        }
    }

    fn reload_debounce_elapsed(&mut self, mod_id: &str) -> bool {
        let now = Instant::now();
        if self
            .last_reload_attempts
            .get(mod_id)
            .is_some_and(|last| now.duration_since(*last) < Duration::from_millis(750))
        {
            return false;
        }
        self.last_reload_attempts.insert(mod_id.to_string(), now);
        true
    }

    fn run_mod(&self, mod_entry: &lua_api::LuaMod, reason: ModRunReason) -> bool {
        run_mod(
            mod_entry,
            self.modules_for_mod(&mod_entry.manifest.uses_plugins),
            reason,
        )
    }

    fn mod_dependencies_available(&self, uses_plugins: &[String]) -> bool {
        uses_plugins.iter().all(|plugin| {
            self.modules
                .iter()
                .any(|module| module.plugin_id.eq_ignore_ascii_case(plugin))
        })
    }

    fn modules_for_mod(&self, uses_plugins: &[String]) -> Vec<RegisteredModule> {
        self.modules
            .iter()
            .filter(|module| {
                uses_plugins
                    .iter()
                    .any(|plugin| module.plugin_id.eq_ignore_ascii_case(plugin))
            })
            .cloned()
            .collect()
    }
}
