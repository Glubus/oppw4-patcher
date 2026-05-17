use std::{
    fs,
    path::{Path, PathBuf},
};

const DEFAULT_CONFIG: &str = r#"# OPPW4 dinput8 host config

[debug]
enabled = false

[active_character]
enabled = true
debug = false
"#;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostConfig {
    pub debug: DebugConfig,
    pub active_character: ActiveCharacterConfig,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DebugConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveCharacterConfig {
    pub enabled: bool,
    pub debug: bool,
}

impl Default for ActiveCharacterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debug: false,
        }
    }
}

pub fn load_or_create(game_root: &Path) -> HostConfig {
    let path = config_path(game_root);
    if !path.is_file() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, DEFAULT_CONFIG);
    }
    let Ok(text) = fs::read_to_string(&path) else {
        return HostConfig::default();
    };
    parse_config(&text).unwrap_or_default()
}

pub fn config_path(game_root: &Path) -> PathBuf {
    game_root.join("mods").join("_oppw4").join("config.toml")
}

fn parse_config(text: &str) -> Option<HostConfig> {
    let value = text.parse::<toml::Value>().ok()?;
    let debug_enabled = value
        .get("debug")
        .and_then(|debug| debug.get("enabled"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let active_character_enabled = value
        .get("active_character")
        .and_then(|active_character| active_character.get("enabled"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let active_character_debug = value
        .get("active_character")
        .and_then(|active_character| active_character.get("debug"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    Some(HostConfig {
        debug: DebugConfig {
            enabled: debug_enabled,
        },
        active_character: ActiveCharacterConfig {
            enabled: active_character_enabled,
            debug: active_character_debug,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_defaults_to_disabled() {
        assert_eq!(parse_config("").unwrap_or_default(), HostConfig::default());
        assert_eq!(
            parse_config("[debug]\n").unwrap().debug,
            DebugConfig { enabled: false }
        );
    }

    #[test]
    fn parses_debug_enabled() {
        assert_eq!(
            parse_config("[debug]\nenabled = true\n").unwrap().debug,
            DebugConfig { enabled: true }
        );
    }

    #[test]
    fn active_character_defaults_to_enabled() {
        assert_eq!(
            parse_config("").unwrap_or_default().active_character,
            ActiveCharacterConfig {
                enabled: true,
                debug: false
            }
        );
    }

    #[test]
    fn parses_active_character_config() {
        assert_eq!(
            parse_config("[active_character]\nenabled = false\ndebug = true\n")
                .unwrap()
                .active_character,
            ActiveCharacterConfig {
                enabled: false,
                debug: true
            }
        );
    }

    #[test]
    fn config_lives_under_oppw4_mod_config_root() {
        assert_eq!(
            config_path(Path::new(r"D:\Game\OPPW4")),
            PathBuf::from(r"D:\Game\OPPW4\mods\_oppw4\config.toml")
        );
    }
}
