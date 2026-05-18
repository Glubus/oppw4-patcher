use std::{
    collections::HashMap,
    ffi::CStr,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use crate::manifest::sanitize_plugin_id;
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
            .route(plugin_id, message);
    }
}

struct PluginLogRouter {
    session_stamp: String,
    roots: HashMap<String, PathBuf>,
    writers: HashMap<String, PluginLogWriter>,
}

struct PluginLogWriter {
    root: PathBuf,
    file: Option<File>,
}

impl PluginLogRouter {
    fn new(session_stamp: Option<String>) -> Self {
        Self {
            session_stamp: session_stamp.unwrap_or_else(time::file_timestamp),
            roots: HashMap::new(),
            writers: HashMap::new(),
        }
    }

    fn register(&mut self, plugin_id: String, log_root: PathBuf) {
        let plugin_id = sanitize_plugin_id(&plugin_id);
        let _ = fs::create_dir_all(&log_root);
        self.roots.insert(plugin_id, log_root);
    }

    fn route(&mut self, plugin_id: &CStr, message: &CStr) -> std::io::Result<()> {
        let plugin_id = sanitize_plugin_id(&plugin_id.to_string_lossy());
        let message = message.to_string_lossy();
        let session_stamp = self.session_stamp.clone();
        self.writer_for(&plugin_id)?.write(&message, &session_stamp)
    }

    fn writer_for(&mut self, plugin_id: &str) -> std::io::Result<&mut PluginLogWriter> {
        if !self.writers.contains_key(plugin_id) {
            let root = self.log_root_for(plugin_id);
            fs::create_dir_all(&root)?;
            self.writers
                .insert(plugin_id.to_string(), PluginLogWriter::new(root));
        }
        Ok(self
            .writers
            .get_mut(plugin_id)
            .expect("plugin log writer was inserted"))
    }

    fn log_root_for(&self, plugin_id: &str) -> PathBuf {
        self.roots
            .get(plugin_id)
            .cloned()
            .unwrap_or_else(|| PathBuf::from("plugins").join(plugin_id).join("logs"))
    }
}

impl PluginLogWriter {
    fn new(root: PathBuf) -> Self {
        Self { root, file: None }
    }

    fn write(&mut self, message: &str, session_stamp: &str) -> std::io::Result<()> {
        let timestamp = time::line_timestamp();
        let file = self.file(session_stamp)?;
        writeln!(file, "[{timestamp}] {message}")?;
        file.flush()
    }

    fn file(&mut self, session_stamp: &str) -> std::io::Result<&mut File> {
        if self.file.is_none() {
            let path = self.root.join(format!("{session_stamp}.log"));
            self.file = Some(OpenOptions::new().create(true).append(true).open(path)?);
        }
        Ok(self.file.as_mut().expect("plugin log file was initialized"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::cstring_lossy;
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
            .route(&skin, &cstring_lossy("skin online"))
            .expect("skin log");
        router
            .route(&fx, &cstring_lossy("fx online"))
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
            fs::read_to_string(
                root.join("fx_tools")
                    .join("logs")
                    .join("2026-05-16-201122.log")
            )
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
            .route(&plugin, &cstring_lossy("clean path"))
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
