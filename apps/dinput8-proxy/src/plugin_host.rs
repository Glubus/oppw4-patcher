use std::{
    collections::HashMap,
    ffi::{c_void, CStr},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use oppw4_plugin_api::{
    optional_cstr, Oppw4LogEntry, Oppw4PluginApi, PluginInitFn, OPPW4_PLUGIN_API_VERSION,
    OPPW4_PLUGIN_INIT_SYMBOL,
};

use crate::{log, win};

static PLUGIN_LOGS: OnceLock<Mutex<PluginLogRouter>> = OnceLock::new();
static LOADED_PLUGINS: OnceLock<Mutex<Vec<LoadedPlugin>>> = OnceLock::new();

pub fn initialize(plugin_root: &Path, plugin_log_root: &Path) {
    let _ = fs::create_dir_all(plugin_root);
    let _ = fs::create_dir_all(plugin_log_root);
    let _ = PLUGIN_LOGS.set(Mutex::new(PluginLogRouter::new(
        plugin_log_root.to_path_buf(),
    )));
    let _ = LOADED_PLUGINS.set(Mutex::new(Vec::new()));

    load_plugins(plugin_root);
}

fn load_plugins(plugin_root: &Path) {
    let Ok(entries) = fs::read_dir(plugin_root) else {
        log::write_line(format!(
            "plugin host: plugin dir not readable path={}",
            plugin_root.display()
        ));
        return;
    };

    let mut loaded = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_plugin_dll(&path) {
            continue;
        }
        if unsafe { load_plugin(&path) } {
            loaded += 1;
        }
    }
    log::write_line(format!("plugin host: loaded={loaded}"));
}

unsafe fn load_plugin(path: &Path) -> bool {
    let wide = path_to_wide(path);
    let module = win::load_library(&wide);
    if module.is_null() {
        log::write_line(format!("plugin host: load failed path={}", path.display()));
        return false;
    }

    let proc = win::get_proc_address(module, OPPW4_PLUGIN_INIT_SYMBOL.as_ptr().cast());
    if proc.is_null() {
        log::write_line(format!(
            "plugin host: init symbol missing path={}",
            path.display()
        ));
        return false;
    }

    let init: PluginInitFn = std::mem::transmute(proc);
    let api = Oppw4PluginApi {
        version: OPPW4_PLUGIN_API_VERSION,
        host_context: std::ptr::null_mut(),
        log: Some(host_log),
    };
    let result = init(&api);
    if result != 0 {
        log::write_line(format!(
            "plugin host: init failed path={} result={result}",
            path.display()
        ));
        return false;
    }

    if let Some(plugins) = LOADED_PLUGINS.get() {
        let mut plugins = plugins.lock().expect("plugin list lock");
        plugins.push(LoadedPlugin {
            _path: path.to_path_buf(),
            _module: module as usize,
        });
    }
    log::write_line(format!("plugin host: initialized path={}", path.display()));
    true
}

unsafe extern "system" fn host_log(_host_context: *mut c_void, entry: *const Oppw4LogEntry) {
    let Some(entry) = entry.as_ref() else {
        return;
    };
    let Some(plugin_id) = optional_cstr(entry.plugin_id) else {
        return;
    };
    let Some(message) = optional_cstr(entry.message) else {
        return;
    };
    if let Some(router) = PLUGIN_LOGS.get() {
        let _ = router
            .lock()
            .expect("plugin log router lock")
            .write(plugin_id, message);
    }
}

fn is_plugin_dll(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
}

fn path_to_wide(path: &Path) -> Vec<u16> {
    let mut wide = path.to_string_lossy().encode_utf16().collect::<Vec<_>>();
    wide.push(0);
    wide
}

struct LoadedPlugin {
    _path: PathBuf,
    _module: usize,
}

struct PluginLogRouter {
    root: PathBuf,
    files: HashMap<String, File>,
}

impl PluginLogRouter {
    fn new(root: PathBuf) -> Self {
        let _ = fs::create_dir_all(&root);
        Self {
            root,
            files: HashMap::new(),
        }
    }

    fn write(&mut self, plugin_id: &CStr, message: &CStr) -> std::io::Result<()> {
        let plugin_id = sanitize_plugin_id(&plugin_id.to_string_lossy());
        let message = message.to_string_lossy();
        let file = self.file_for(&plugin_id)?;
        writeln!(file, "{message}")?;
        file.flush()
    }

    fn file_for(&mut self, plugin_id: &str) -> std::io::Result<&mut File> {
        if !self.files.contains_key(plugin_id) {
            let path = self.root.join(format!("{plugin_id}.log"));
            let file = OpenOptions::new().create(true).append(true).open(path)?;
            self.files.insert(plugin_id.to_string(), file);
        }
        Ok(self
            .files
            .get_mut(plugin_id)
            .expect("plugin log file was inserted"))
    }
}

fn sanitize_plugin_id(raw: &str) -> String {
    let sanitized = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if sanitized.is_empty() {
        "unknown_plugin".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oppw4_plugin_api::cstring_lossy;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn plugin_logs_are_routed_to_per_plugin_files() {
        let root = temp_root("plugin-log-routing");
        let mut router = PluginLogRouter::new(root.clone());
        let skin = cstring_lossy("skin_patcher");
        let fx = cstring_lossy("fx_tools");

        router
            .write(&skin, &cstring_lossy("skin online"))
            .expect("skin log");
        router
            .write(&fx, &cstring_lossy("fx online"))
            .expect("fx log");

        assert_eq!(
            fs::read_to_string(root.join("skin_patcher.log")).expect("skin log file"),
            "skin online\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("fx_tools.log")).expect("fx log file"),
            "fx online\n"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plugin_log_file_names_are_sanitized() {
        let root = temp_root("plugin-log-sanitize");
        let mut router = PluginLogRouter::new(root.clone());
        let plugin = cstring_lossy("../skin patcher.dll");

        router
            .write(&plugin, &cstring_lossy("clean path"))
            .expect("sanitized log");

        assert_eq!(
            fs::read_to_string(root.join("skin_patcher_dll.log")).expect("sanitized log file"),
            "clean path\n"
        );
        assert!(!root.join("..").join("skin patcher.dll.log").exists());
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
