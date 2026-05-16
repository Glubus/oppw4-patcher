use std::{
    collections::HashMap,
    ffi::CStr,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use super::manifest::sanitize_plugin_id;
use super::time;

static ROUTER: OnceLock<Mutex<PluginLogRouter>> = OnceLock::new();

pub(crate) fn initialize(session_stamp: Option<String>) {
    let _ = ROUTER.set(Mutex::new(PluginLogRouter::new(session_stamp)));
}

pub(crate) fn register(plugin_id: String, log_root: PathBuf) {
    if let Some(router) = ROUTER.get() {
        router
            .lock()
            .expect("plugin log router lock")
            .register(plugin_id, log_root);
    }
}

pub(crate) fn write(plugin_id: &CStr, message: &CStr) {
    if let Some(router) = ROUTER.get() {
        let _ = router
            .lock()
            .expect("plugin log router lock")
            .write(plugin_id, message);
    }
}

struct PluginLogRouter {
    session_stamp: String,
    roots: HashMap<String, PathBuf>,
    files: HashMap<String, File>,
}

impl PluginLogRouter {
    fn new(session_stamp: Option<String>) -> Self {
        Self {
            session_stamp: session_stamp.unwrap_or_else(time::file_timestamp),
            roots: HashMap::new(),
            files: HashMap::new(),
        }
    }

    fn register(&mut self, plugin_id: String, log_root: PathBuf) {
        let plugin_id = sanitize_plugin_id(&plugin_id);
        let _ = fs::create_dir_all(&log_root);
        self.roots.insert(plugin_id, log_root);
    }

    fn write(&mut self, plugin_id: &CStr, message: &CStr) -> std::io::Result<()> {
        let plugin_id = sanitize_plugin_id(&plugin_id.to_string_lossy());
        let message = message.to_string_lossy();
        tracing::info!(plugin_id = %plugin_id, "plugin: {message}");
        let timestamp = time::line_timestamp();
        let file = self.file_for(&plugin_id)?;
        writeln!(file, "[{timestamp}] {message}")?;
        file.flush()
    }

    fn file_for(&mut self, plugin_id: &str) -> std::io::Result<&mut File> {
        if !self.files.contains_key(plugin_id) {
            let root = self
                .roots
                .get(plugin_id)
                .cloned()
                .unwrap_or_else(|| PathBuf::from("plugins").join(plugin_id).join("logs"));
            fs::create_dir_all(&root)?;
            let path = root.join(format!("{}.log", self.session_stamp));
            let file = OpenOptions::new().create(true).append(true).open(path)?;
            self.files.insert(plugin_id.to_string(), file);
        }
        Ok(self
            .files
            .get_mut(plugin_id)
            .expect("plugin log file was inserted"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oppw4_plugin_api::cstring_lossy;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn plugin_logs_are_routed_to_registered_plugin_folder() {
        let root = temp_root("plugin-log-routing");
        let mut router = PluginLogRouter::new(Some("2026-05-16-201122".to_string()));
        router.register(
            "skin_patcher".to_string(),
            root.join("skin_patcher").join("logs"),
        );
        router.register("fx_tools".to_string(), root.join("fx_tools").join("logs"));
        let skin = cstring_lossy("skin_patcher");
        let fx = cstring_lossy("fx_tools");

        router
            .write(&skin, &cstring_lossy("skin online"))
            .expect("skin log");
        router
            .write(&fx, &cstring_lossy("fx online"))
            .expect("fx log");

        assert_eq!(
            fs::read_to_string(
                root.join("skin_patcher")
                    .join("logs")
                    .join("2026-05-16-201122.log")
            )
            .expect("skin log file"),
            "[2026-05-16 20:11:22] skin online\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("fx_tools").join("logs").join("2026-05-16-201122.log"))
                .expect("fx log file"),
            "[2026-05-16 20:11:22] fx online\n"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plugin_log_file_names_are_sanitized() {
        let root = temp_root("plugin-log-sanitize");
        let mut router = PluginLogRouter::new(Some("2026-05-16-201122".to_string()));
        router.register(
            "../skin patcher.dll".to_string(),
            root.join("skin_patcher_dll").join("logs"),
        );
        let plugin = cstring_lossy("../skin patcher.dll");

        router
            .write(&plugin, &cstring_lossy("clean path"))
            .expect("sanitized log");

        assert_eq!(
            fs::read_to_string(
                root.join("skin_patcher_dll")
                    .join("logs")
                    .join("2026-05-16-201122.log")
            )
            .expect("sanitized log file"),
            "[2026-05-16 20:11:22] clean path\n"
        );
        assert!(!root.join("..").join("2026-05-16-201122.log").exists());
        let _ = fs::remove_dir_all(root);
    }

    fn temp_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("oppw4-{label}-{nanos}"))
    }
}
