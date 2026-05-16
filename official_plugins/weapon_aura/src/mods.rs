use std::{fs::File, io::Read, path::Path};

use oppw4_plugin_api::Oppw4PluginApi;

use crate::{config::AuraConfig, log};

const AURA_MOD_CONFIG: &str = "aura.toml";

pub(crate) fn load_config(api: &Oppw4PluginApi) -> AuraConfig {
    let mut config = AuraConfig::load(api).unwrap_or_default();
    let mut zip_paths = api.plugin_mod_zips();
    zip_paths.sort_by_key(|path| path.to_ascii_lowercase());

    if zip_paths.is_empty() {
        log::write_line("weapon_aura mods: no zip mods");
        return config;
    }

    log::write_line(format!("weapon_aura mods: {} zip(s)", zip_paths.len()));
    for zip_path in zip_paths {
        apply_zip_config(&mut config, Path::new(&zip_path));
    }
    config
}

fn apply_zip_config(config: &mut AuraConfig, zip_path: &Path) {
    let Ok(file) = File::open(zip_path) else {
        log::write_line(format!(
            "weapon_aura mods: cannot open {}",
            zip_path.display()
        ));
        return;
    };
    let Ok(mut archive) = zip::ZipArchive::new(file) else {
        log::write_line(format!(
            "weapon_aura mods: invalid zip {}",
            zip_path.display()
        ));
        return;
    };

    let Some(index) = find_aura_config_entry(&mut archive) else {
        log::write_line(format!(
            "weapon_aura mods: {} has no {AURA_MOD_CONFIG}",
            zip_path.display()
        ));
        return;
    };
    let Ok(mut entry) = archive.by_index(index) else {
        return;
    };
    let mut text = String::new();
    if entry.read_to_string(&mut text).is_err() {
        log::write_line(format!(
            "weapon_aura mods: cannot read {AURA_MOD_CONFIG} in {}",
            zip_path.display()
        ));
        return;
    }
    if config.apply_toml_text(&text).is_err() {
        log::write_line(format!(
            "weapon_aura mods: invalid {AURA_MOD_CONFIG} in {}",
            zip_path.display()
        ));
        return;
    }
    log::write_line(format!(
        "weapon_aura mods: applied {}",
        zip_path.display()
    ));
}

fn find_aura_config_entry<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<usize> {
    for index in 0..archive.len() {
        let Ok(entry) = archive.by_index(index) else {
            continue;
        };
        if !entry.is_file() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        if name
            .rsplit('/')
            .next()
            .is_some_and(|file_name| file_name.eq_ignore_ascii_case(AURA_MOD_CONFIG))
        {
            return Some(index);
        }
    }
    None
}
