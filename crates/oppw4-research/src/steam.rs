pub const STEAM_APPS_B_IS_SUBSCRIBED_APP_VTABLE_INDEX: usize = 6;
pub const STEAM_APPS_B_IS_DLC_INSTALLED_VTABLE_INDEX: usize = 7;
pub const STEAM_APPS_MAX_LOGS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SteamAppsBoolMethod {
    IsSubscribedApp,
    IsDlcInstalled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SteamAppsSymbol {
    InterfaceVersion(&'static str),
    BoolMethod(SteamAppsBoolMethod),
}

impl SteamAppsBoolMethod {
    pub fn label(self) -> &'static str {
        match self {
            SteamAppsBoolMethod::IsSubscribedApp => "BIsSubscribedApp",
            SteamAppsBoolMethod::IsDlcInstalled => "BIsDlcInstalled",
        }
    }

    pub fn vtable_index(self) -> usize {
        match self {
            SteamAppsBoolMethod::IsSubscribedApp => STEAM_APPS_B_IS_SUBSCRIBED_APP_VTABLE_INDEX,
            SteamAppsBoolMethod::IsDlcInstalled => STEAM_APPS_B_IS_DLC_INSTALLED_VTABLE_INDEX,
        }
    }
}

pub fn steam_apps_import_version(name: &str) -> Option<&'static str> {
    match name {
        "SteamAPI_SteamApps_v006" => Some("v006"),
        "SteamAPI_SteamApps_v007" => Some("v007"),
        "SteamAPI_SteamApps_v008" => Some("v008"),
        _ => None,
    }
}

pub fn steam_apps_bool_method_import(name: &str) -> Option<SteamAppsBoolMethod> {
    match name {
        "SteamAPI_ISteamApps_BIsSubscribedApp" => Some(SteamAppsBoolMethod::IsSubscribedApp),
        "SteamAPI_ISteamApps_BIsDlcInstalled" => Some(SteamAppsBoolMethod::IsDlcInstalled),
        _ => None,
    }
}

pub fn steam_apps_probe_symbol(name: &str) -> Option<SteamAppsSymbol> {
    steam_apps_import_version(name)
        .map(SteamAppsSymbol::InterfaceVersion)
        .or_else(|| steam_apps_bool_method_import(name).map(SteamAppsSymbol::BoolMethod))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_steam_apps_import_versions() {
        assert_eq!(
            steam_apps_import_version("SteamAPI_SteamApps_v008"),
            Some("v008")
        );
        assert_eq!(
            steam_apps_import_version("SteamAPI_SteamApps_v007"),
            Some("v007")
        );
        assert_eq!(steam_apps_import_version("SteamAPI_Init"), None);
    }

    #[test]
    fn recognizes_steam_apps_dlc_check_imports() {
        assert_eq!(
            steam_apps_bool_method_import("SteamAPI_ISteamApps_BIsDlcInstalled"),
            Some(SteamAppsBoolMethod::IsDlcInstalled)
        );
        assert_eq!(
            steam_apps_bool_method_import("SteamAPI_ISteamApps_BIsSubscribedApp"),
            Some(SteamAppsBoolMethod::IsSubscribedApp)
        );
        assert_eq!(
            steam_apps_bool_method_import("SteamAPI_SteamApps_v008"),
            None
        );
    }

    #[test]
    fn documents_known_steam_apps_vtable_slots() {
        assert_eq!(
            SteamAppsBoolMethod::IsSubscribedApp.vtable_index(),
            STEAM_APPS_B_IS_SUBSCRIBED_APP_VTABLE_INDEX
        );
        assert_eq!(
            SteamAppsBoolMethod::IsDlcInstalled.vtable_index(),
            STEAM_APPS_B_IS_DLC_INSTALLED_VTABLE_INDEX
        );
    }

    #[test]
    fn classifies_dynamic_steam_apps_symbols() {
        assert_eq!(
            steam_apps_probe_symbol("SteamAPI_SteamApps_v008"),
            Some(SteamAppsSymbol::InterfaceVersion("v008"))
        );
        assert_eq!(
            steam_apps_probe_symbol("SteamAPI_ISteamApps_BIsDlcInstalled"),
            Some(SteamAppsSymbol::BoolMethod(
                SteamAppsBoolMethod::IsDlcInstalled
            ))
        );
        assert_eq!(steam_apps_probe_symbol("SteamAPI_Init"), None);
    }
}
