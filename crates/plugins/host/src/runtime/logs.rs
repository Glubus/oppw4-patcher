use std::{
    collections::HashMap,
    ffi::CStr,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use super::time;
use plugin_sdk::manifest::sanitize_plugin_id;

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
mod tests;
