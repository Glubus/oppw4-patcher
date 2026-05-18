use std::path::PathBuf;

pub(crate) struct ApiContext {
    pub(super) plugin_id: String,
    pub(super) plugin_mods_root: PathBuf,
}

impl ApiContext {
    pub(crate) fn new(plugin_id: String, plugin_mods_root: PathBuf) -> Self {
        Self {
            plugin_id,
            plugin_mods_root,
        }
    }
}
