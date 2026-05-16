use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString},
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

pub fn initialize(game_root: &Path, plugin_root: &Path) {
    let _ = fs::create_dir_all(plugin_root);
    let _ = PLUGIN_LOGS.set(Mutex::new(PluginLogRouter::new()));
    let _ = LOADED_PLUGINS.set(Mutex::new(Vec::new()));

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
    register_plugin_logs(manifest);

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
    let game_root_utf8 = cstring_lossy(&game_root.to_string_lossy());
    let api = Oppw4PluginApi {
        version: OPPW4_PLUGIN_API_VERSION,
        host_context: std::ptr::null_mut(),
        game_root_utf8: game_root_utf8.as_ptr(),
        log: Some(host_log),
    };
    let result = init(&api);
    if result != 0 {
        log::write_line(format!(
            "plugin host: init failed id={} path={} result={result}",
            manifest.id,
            manifest.entry_path.display()
        ));
        return false;
    }

    if let Some(plugins) = LOADED_PLUGINS.get() {
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

fn cstring_lossy(value: &str) -> CString {
    let bytes = value
        .as_bytes()
        .iter()
        .copied()
        .filter(|byte| *byte != 0)
        .collect::<Vec<_>>();
    CString::new(bytes).unwrap_or_else(|_| CString::new("").expect("empty cstring"))
}

fn register_plugin_logs(manifest: &PluginManifest) {
    if let Some(router) = PLUGIN_LOGS.get() {
        router
            .lock()
            .expect("plugin log router lock")
            .register(manifest.id.clone(), manifest.log_root.clone());
    }
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

#[derive(Debug, PartialEq)]
struct PluginManifest {
    id: String,
    version: String,
    entry_path: PathBuf,
    log_root: PathBuf,
}

impl PluginManifest {
    fn read_from_dir(plugin_dir: &Path) -> Option<Self> {
        let manifest_path = plugin_dir.join("mod.toml");
        let text = match fs::read_to_string(&manifest_path) {
            Ok(text) => text,
            Err(error) => {
                log::write_line(format!(
                    "plugin host: manifest missing path={} error={error}",
                    manifest_path.display()
                ));
                return None;
            }
        };

        match Self::parse(plugin_dir, &text) {
            Ok(manifest) => Some(manifest),
            Err(error) => {
                log::write_line(format!(
                    "plugin host: manifest invalid path={} error={error}",
                    manifest_path.display()
                ));
                None
            }
        }
    }

    fn parse(plugin_dir: &Path, text: &str) -> Result<Self, String> {
        let value = text
            .parse::<toml::Value>()
            .map_err(|error| error.to_string())?;
        let plugin = value
            .get("plugin")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| "missing [plugin] table".to_string())?;
        let id = plugin
            .get("id")
            .and_then(toml::Value::as_str)
            .map(sanitize_plugin_id)
            .filter(|id| id != "unknown_plugin")
            .ok_or_else(|| "missing plugin.id".to_string())?;
        let entry = plugin
            .get("entry")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| "missing plugin.entry".to_string())?;
        let version = plugin
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| "missing plugin.version".to_string())?
            .to_string();
        Ok(Self {
            id,
            version,
            entry_path: entry_file_path(plugin_dir, entry)?,
            log_root: plugin_dir.join("logs"),
        })
    }
}

struct PluginLogRouter {
    roots: HashMap<String, PathBuf>,
    files: HashMap<String, File>,
}

impl PluginLogRouter {
    fn new() -> Self {
        Self {
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
        let file = self.file_for(&plugin_id)?;
        writeln!(file, "{message}")?;
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
            let path = root.join(format!("{plugin_id}.log"));
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

fn entry_file_path(root: &Path, child: &str) -> Result<PathBuf, String> {
    let path = Path::new(child);
    if path.is_absolute()
        || path.components().count() != 1
        || path.file_name().and_then(|value| value.to_str()) != Some(child)
    {
        return Err(format!("entry must be a file name only: {child}"));
    }
    Ok(root.join(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oppw4_plugin_api::cstring_lossy;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn manifest_points_entry_and_logs_inside_plugin_folder() {
        let root = PathBuf::from(r"D:\Game\OPPW4\plugins\skin_patcher");
        let manifest = PluginManifest::parse(
            &root,
            r#"
                [plugin]
                id = "skin_patcher"
                version = "0.1.0"
                entry = "skin_patcher.dll"
            "#,
        )
        .expect("manifest");

        assert_eq!(manifest.id, "skin_patcher");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.entry_path, root.join("skin_patcher.dll"));
        assert_eq!(manifest.log_root, root.join("logs"));
    }

    #[test]
    fn manifest_rejects_entry_with_path_segments() {
        let error = PluginManifest::parse(
            Path::new(r"D:\Game\OPPW4\plugins\bad"),
            r#"
                [plugin]
                id = "bad"
                version = "0.1.0"
                entry = "bin/bad.dll"
            "#,
        )
        .expect_err("entry should be rejected");

        assert!(error.contains("file name only"));
    }

    #[test]
    fn manifest_requires_version() {
        let error = PluginManifest::parse(
            Path::new(r"D:\Game\OPPW4\plugins\bad"),
            r#"
                [plugin]
                id = "bad"
                entry = "bad.dll"
            "#,
        )
        .expect_err("version should be required");

        assert!(error.contains("plugin.version"));
    }

    #[test]
    fn plugin_logs_are_routed_to_registered_plugin_folder() {
        let root = temp_root("plugin-log-routing");
        let mut router = PluginLogRouter::new();
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
                    .join("skin_patcher.log")
            )
            .expect("skin log file"),
            "skin online\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("fx_tools").join("logs").join("fx_tools.log"))
                .expect("fx log file"),
            "fx online\n"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plugin_log_file_names_are_sanitized() {
        let root = temp_root("plugin-log-sanitize");
        let mut router = PluginLogRouter::new();
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
                    .join("skin_patcher_dll.log")
            )
            .expect("sanitized log file"),
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
