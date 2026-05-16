// apps/dinput8-clean/src/changes/law_slot5_assets.rs
use std::path::Path;

use oppw4_rdb::{ModAsset, ReplacementMode, VirtualReplacement};

use crate::{log, mods::ModRepository};

const SHARED_LAW_ASSET_ROUTES_ENABLED: bool = false;
const DIRECT_PRIVATE_PATH_ROUTES_ENABLED: bool = true;
const TARGET_MODEL_RESOURCE_ID: u16 = 730;
const PRIVATE_MODEL_HASH: u32 = 0x730c_0575;
const PRIVATE_MODEL_BIN_SUFFIX: &str = "120";
const MODEL_TEMPLATE_HASH: u32 = 0x8df2_d8cb;

const ROUTES: &[AssetRoute] = &[
    AssetRoute::new(
        "CharacterEditor",
        "MPLC026_Law.g1m",
        "MDLC999_Law_Custom.g1m",
    ),
    AssetRoute::new(
        "MaterialEditor",
        "MPR_Bound_Character_MPLC026Law_cloth_kidsalb.g1t",
        "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsalb.g1t",
    ),
    AssetRoute::new(
        "MaterialEditor",
        "MPR_Bound_Character_MPLC026Law_cloth_kidsnmh.g1t",
        "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsnmh.g1t",
    ),
    AssetRoute::new(
        "MaterialEditor",
        "MPR_Bound_Character_MPLC026Law_cloth_kidsocc.g1t",
        "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsocc.g1t",
    ),
    AssetRoute::new(
        "MaterialEditor",
        "MPR_Bound_Character_MPLC026Law_cloth_kidsrfr.g1t",
        "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsrfr.g1t",
    ),
    AssetRoute::new(
        "MaterialEditor",
        "MPR_Bound_Character_MPLC026Law_skin_kidsalb.g1t",
        "MPR_Bound_Character_MDLC999LawCustom_skin_kidsalb.g1t",
    ),
    AssetRoute::new(
        "MaterialEditor",
        "MPR_Bound_Character_MPLC026Law_skin_kidsnmh.g1t",
        "MPR_Bound_Character_MDLC999LawCustom_skin_kidsnmh.g1t",
    ),
    AssetRoute::new(
        "MaterialEditor",
        "MPR_Bound_Character_MPLC026Law_skin_kidsocc.g1t",
        "MPR_Bound_Character_MDLC999LawCustom_skin_kidsocc.g1t",
    ),
    AssetRoute::new(
        "MaterialEditor",
        "MPR_Bound_Character_MPLC026Law_skin_kidss4m.g1t",
        "MPR_Bound_Character_MDLC999LawCustom_skin_kidss4m.g1t",
    ),
    AssetRoute::new(
        "RRPreview",
        "CE1_0080_STYLE_IMPACT_HAO.g1e",
        "CE1_0080_STYLE_IMPACT_HAO.g1e",
    ),
    AssetRoute::new(
        "RRPreview",
        "CE1_0080_STYLE_IMPACT_HAO.ktid",
        "CE1_0080_STYLE_IMPACT_HAO.ktid",
    ),
    AssetRoute::new(
        "ScreenLayout",
        "800_294_face_law_dressrosa_External_00.g1t",
        "800_294_face_law_dressrosa_External_00.g1t",
    ),
    AssetRoute::new(
        "ScreenLayout",
        "801_294_chara_law_dressrosa_External_00.g1t",
        "801_294_chara_law_dressrosa_External_00.g1t",
    ),
];

const GLOBAL_EXCLUDED_NAMES: &[&str] = &[
    "MPLC026_Law.g1m",
    "MPR_Bound_Character_MPLC026Law_cloth_kidsalb.g1t",
    "MPR_Bound_Character_MPLC026Law_cloth_kidsnmh.g1t",
    "MPR_Bound_Character_MPLC026Law_cloth_kidsocc.g1t",
    "MPR_Bound_Character_MPLC026Law_cloth_kidsrfr.g1t",
    "MPR_Bound_Character_MPLC026Law_skin_kidsalb.g1t",
    "MPR_Bound_Character_MPLC026Law_skin_kidsnmh.g1t",
    "MPR_Bound_Character_MPLC026Law_skin_kidsocc.g1t",
    "MPR_Bound_Character_MPLC026Law_skin_kidss4m.g1t",
    "CE1_0080_STYLE_IMPACT_HAO.g1e",
    "CE1_0080_STYLE_IMPACT_HAO.ktid",
    "800_294_face_law_dressrosa_External_00.g1t",
    "801_294_chara_law_dressrosa_External_00.g1t",
];

#[derive(Debug, Clone, Copy)]
struct AssetRoute {
    archive: &'static str,
    original_name: &'static str,
    custom_name: &'static str,
}

impl AssetRoute {
    const fn new(
        archive: &'static str,
        original_name: &'static str,
        custom_name: &'static str,
    ) -> Self {
        Self {
            archive,
            original_name,
            custom_name,
        }
    }
}

pub fn load_routes(
    mods_root: &Path,
    rdb_root: &Path,
    catalog: &[oppw4_rdb::NameHashEntry],
) -> Vec<VirtualReplacement> {
    if !SHARED_LAW_ASSET_ROUTES_ENABLED {
        log::write_line(format!(
            "law-slot5-asset-routes disabled reason=shared_law_hashes_contaminate_resource26 direct_private_path_routes={DIRECT_PRIVATE_PATH_ROUTES_ENABLED}"
        ));
        if DIRECT_PRIVATE_PATH_ROUTES_ENABLED {
            return load_direct_private_path_routes(mods_root);
        }
        return Vec::new();
    }

    let mods = ModRepository::new(mods_root.to_path_buf());
    let mut replacements = Vec::new();
    for archive in [
        "CharacterEditor",
        "MaterialEditor",
        "RRPreview",
        "ScreenLayout",
    ] {
        replacements.extend(load_archive_routes(&mods, rdb_root, archive, catalog));
    }
    let count = replacements.len();
    let replacements = oppw4_rdb::attach_mod_file_sizes(replacements).unwrap_or_else(|error| {
        log::write_line(format!(
            "law-slot5-asset-routes size attach failed: {error}"
        ));
        Vec::new()
    });
    let replacements = assign_archive_offsets(replacements, rdb_root);
    log::write_line(format!(
        "law-slot5-asset-routes ready count={count} published={}",
        replacements.len()
    ));
    replacements
}

pub fn load_global_virtual_rdb_routes(
    mods_root: &Path,
    rdb_root: &Path,
) -> Vec<VirtualReplacement> {
    let mods = ModRepository::new(mods_root.to_path_buf());
    let custom_assets = mods.archive_assets("CharacterEditor");
    let Some(model_asset) = custom_assets.iter().find(|asset| {
        asset
            .file_name
            .eq_ignore_ascii_case("MDLC999_Law_Custom_730.g1m")
            || asset
                .file_name
                .eq_ignore_ascii_case("MDLC999_Law_Custom.g1m")
            || asset.file_name.eq_ignore_ascii_case("MPLC026_Law.g1m")
    }) else {
        log::write_line("law-slot5-virtual-rdb skipped: no custom CharacterEditor model asset");
        return Vec::new();
    };

    let Ok(model_size) = model_asset.source.payload_size() else {
        log::write_line(format!(
            "law-slot5-virtual-rdb skipped: model size unreadable source={}",
            model_asset.source.display_name()
        ));
        return Vec::new();
    };
    let rdb_path = rdb_root.join("CharacterEditor.rdb");
    let Ok(rdb_bytes) = std::fs::read(&rdb_path) else {
        log::write_line(format!(
            "law-slot5-virtual-rdb skipped: rdb unreadable path={}",
            rdb_path.display()
        ));
        return Vec::new();
    };
    let Ok(patched_rdb) = oppw4_rdb::append_virtual_rdb_entry(
        &rdb_bytes,
        oppw4_rdb::RdbAppendSpec {
            template_hash: MODEL_TEMPLATE_HASH,
            private_hash: PRIVATE_MODEL_HASH,
            virtual_bin_suffix: PRIVATE_MODEL_BIN_SUFFIX.to_string(),
            payload_size: model_size,
        },
    ) else {
        log::write_line(format!(
            "law-slot5-virtual-rdb skipped: append failed template_hash=0x{MODEL_TEMPLATE_HASH:08x} private_hash=0x{PRIVATE_MODEL_HASH:08x}"
        ));
        return Vec::new();
    };

    log::write_line(format!(
        "law-slot5-virtual-rdb ready archive=CharacterEditor hash=0x{PRIVATE_MODEL_HASH:08x} bin=CharacterEditor.rdb.bin{PRIVATE_MODEL_BIN_SUFFIX} model_size=0x{model_size:x} rdb_size=0x{:x}->0x{:x}",
        rdb_bytes.len(),
        patched_rdb.len()
    ));

    vec![
        virtual_memory_replacement(
            "CharacterEditor",
            "CharacterEditor.rdb",
            oppw4_rdb::ReplacementSource::Memory {
                name: "CharacterEditor.rdb".to_string(),
                bytes: patched_rdb,
            },
        ),
        virtual_memory_replacement(
            "CharacterEditor",
            &format!("CharacterEditor.rdb.bin{PRIVATE_MODEL_BIN_SUFFIX}"),
            model_asset.source.clone(),
        ),
    ]
}

fn load_archive_routes(
    mods: &ModRepository,
    rdb_root: &Path,
    archive: &str,
    catalog: &[oppw4_rdb::NameHashEntry],
) -> Vec<VirtualReplacement> {
    let custom_assets = mods.archive_assets(archive);
    let route_assets = ROUTES
        .iter()
        .filter(|route| route.archive.eq_ignore_ascii_case(archive))
        .filter_map(|route| route_asset(route, &custom_assets))
        .collect::<Vec<_>>();
    if route_assets.is_empty() {
        log::write_line(format!(
            "law-slot5-asset-routes {archive}: no custom assets"
        ));
        return Vec::new();
    }

    let names = route_assets
        .iter()
        .map(|asset| asset.file_name.as_str())
        .collect::<Vec<_>>();
    let rdb_path = rdb_root.join(format!("{archive}.rdb"));
    let Ok(bytes) = std::fs::read(&rdb_path) else {
        log::write_line(format!(
            "law-slot5-asset-routes {archive}: rdb not readable path={}",
            rdb_path.display()
        ));
        return Vec::new();
    };
    let Ok(index) = oppw4_rdb::parse_rdb(&bytes) else {
        log::write_line(format!(
            "law-slot5-asset-routes {archive}: rdb parse failed"
        ));
        return Vec::new();
    };
    let scan = oppw4_rdb::scan_archive_names_with_catalog(archive, &index, &names, catalog);
    let counts = scan.counts();
    log::write_line(format!(
        "law-slot5-asset-routes {archive}: routes={} matched={} hash_missing={} unresolved={}",
        route_assets.len(),
        counts.matched,
        counts.hash_missing,
        counts.unresolved_names
    ));
    oppw4_rdb::build_virtualization_table_from_assets(&scan, &route_assets)
}

fn route_asset(route: &AssetRoute, custom_assets: &[ModAsset]) -> Option<ModAsset> {
    let matched = custom_assets.iter().find(|asset| {
        asset.file_name.eq_ignore_ascii_case(route.custom_name)
            || asset.file_name.eq_ignore_ascii_case(route.original_name)
    })?;
    let source = matched.source.clone();
    log::write_line(format!(
        "law-slot5-asset-route source={} target_original={} private_name={}",
        matched.file_name, route.original_name, route.custom_name
    ));
    Some(ModAsset {
        file_name: route.original_name.to_string(),
        source,
    })
}

fn load_direct_private_path_routes(mods_root: &Path) -> Vec<VirtualReplacement> {
    let mods = ModRepository::new(mods_root.to_path_buf());
    let mut replacements = Vec::new();
    for archive in ["CharacterEditor", "MaterialEditor"] {
        let custom_assets = mods.archive_assets(archive);
        for route in ROUTES
            .iter()
            .filter(|route| route.archive.eq_ignore_ascii_case(archive))
        {
            let Some(asset) = route_asset(route, &custom_assets) else {
                continue;
            };
            for private_name in direct_private_names(route) {
                replacements.push(direct_private_replacement(
                    archive,
                    private_name,
                    asset.source.clone(),
                ));
            }
        }
    }

    let replacements = oppw4_rdb::attach_mod_file_sizes(replacements).unwrap_or_else(|error| {
        log::write_line(format!(
            "law-slot5-direct-private-routes size attach failed: {error}"
        ));
        Vec::new()
    });
    log::write_line(format!(
        "law-slot5-direct-private-routes ready published={} target_model={TARGET_MODEL_RESOURCE_ID}",
        replacements.len()
    ));
    replacements
}

fn direct_private_names(route: &AssetRoute) -> Vec<&'static str> {
    match route.original_name {
        "MPLC026_Law.g1m" => vec!["MDLC999_Law_Custom_730.g1m", "MDLC999_Law_Custom.g1m"],
        "MPR_Bound_Character_MPLC026Law_cloth_kidsalb.g1t" => vec![
            "MPR_Bound_Character_MDLC999LawCustom730_cloth_kidsalb.g1t",
            "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsalb.g1t",
        ],
        "MPR_Bound_Character_MPLC026Law_cloth_kidsnmh.g1t" => vec![
            "MPR_Bound_Character_MDLC999LawCustom730_cloth_kidsnmh.g1t",
            "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsnmh.g1t",
        ],
        "MPR_Bound_Character_MPLC026Law_cloth_kidsocc.g1t" => vec![
            "MPR_Bound_Character_MDLC999LawCustom730_cloth_kidsocc.g1t",
            "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsocc.g1t",
        ],
        "MPR_Bound_Character_MPLC026Law_cloth_kidsrfr.g1t" => vec![
            "MPR_Bound_Character_MDLC999LawCustom730_cloth_kidsrfr.g1t",
            "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsrfr.g1t",
        ],
        "MPR_Bound_Character_MPLC026Law_skin_kidsalb.g1t" => vec![
            "MPR_Bound_Character_MDLC999LawCustom730_skin_kidsalb.g1t",
            "MPR_Bound_Character_MDLC999LawCustom_skin_kidsalb.g1t",
        ],
        "MPR_Bound_Character_MPLC026Law_skin_kidsnmh.g1t" => vec![
            "MPR_Bound_Character_MDLC999LawCustom730_skin_kidsnmh.g1t",
            "MPR_Bound_Character_MDLC999LawCustom_skin_kidsnmh.g1t",
        ],
        "MPR_Bound_Character_MPLC026Law_skin_kidsocc.g1t" => vec![
            "MPR_Bound_Character_MDLC999LawCustom730_skin_kidsocc.g1t",
            "MPR_Bound_Character_MDLC999LawCustom_skin_kidsocc.g1t",
        ],
        "MPR_Bound_Character_MPLC026Law_skin_kidss4m.g1t" => vec![
            "MPR_Bound_Character_MDLC999LawCustom730_skin_kidss4m.g1t",
            "MPR_Bound_Character_MDLC999LawCustom_skin_kidss4m.g1t",
        ],
        _ => Vec::new(),
    }
}

fn direct_private_replacement(
    archive_name: &str,
    file_name: &str,
    source: oppw4_rdb::ReplacementSource,
) -> VirtualReplacement {
    VirtualReplacement {
        archive_name: archive_name.to_string(),
        file_name: file_name.to_string(),
        source,
        mode: ReplacementMode::Virtual,
        mod_size: None,
        hash: 0,
        rdb_block_offset: usize::MAX,
        original_data_offset: 0,
        original_bin_offset: None,
        original_bin_size: None,
        virtual_bin_offset: None,
        rdb_tail_offset: None,
        original_tail: None,
        virtual_prefix: None,
    }
}

fn virtual_memory_replacement(
    archive_name: &str,
    file_name: &str,
    source: oppw4_rdb::ReplacementSource,
) -> VirtualReplacement {
    VirtualReplacement {
        archive_name: archive_name.to_string(),
        file_name: file_name.to_string(),
        source,
        mode: ReplacementMode::Virtual,
        mod_size: None,
        hash: PRIVATE_MODEL_HASH,
        rdb_block_offset: usize::MAX,
        original_data_offset: 0,
        original_bin_offset: None,
        original_bin_size: None,
        virtual_bin_offset: None,
        rdb_tail_offset: None,
        original_tail: None,
        virtual_prefix: None,
    }
}

fn assign_archive_offsets(
    replacements: Vec<VirtualReplacement>,
    rdb_root: &Path,
) -> Vec<VirtualReplacement> {
    let mut assigned = replacements;
    for archive in [
        "CharacterEditor",
        "MaterialEditor",
        "RRPreview",
        "ScreenLayout",
    ] {
        let bin_path = rdb_root.join(format!("{archive}.rdb.bin"));
        let Ok(metadata) = std::fs::metadata(&bin_path) else {
            continue;
        };
        assigned = oppw4_rdb::assign_virtual_bin_offsets(assigned, archive, metadata.len());
    }
    assigned
}

pub fn is_reserved_private_asset_name(file_name: &str) -> bool {
    GLOBAL_EXCLUDED_NAMES
        .iter()
        .any(|reserved| file_name.eq_ignore_ascii_case(reserved))
}

#[cfg(test)]
mod tests {
    #[test]
    fn shared_law_asset_routes_are_disabled_until_private_model_is_verified() {
        assert!(!super::SHARED_LAW_ASSET_ROUTES_ENABLED);
    }

    #[test]
    fn direct_private_model_route_uses_virtual_id_suffix() {
        let names = super::direct_private_names(&super::AssetRoute::new(
            "CharacterEditor",
            "MPLC026_Law.g1m",
            "MDLC999_Law_Custom.g1m",
        ));

        assert_eq!(names[0], "MDLC999_Law_Custom_730.g1m");
    }

    #[test]
    fn direct_private_replacement_does_not_patch_rdb_index() {
        let replacement = super::direct_private_replacement(
            "CharacterEditor",
            "MDLC999_Law_Custom_730.g1m",
            oppw4_rdb::ReplacementSource::File("model.g1m".into()),
        );

        assert_eq!(replacement.rdb_block_offset, usize::MAX);
        assert_eq!(replacement.virtual_bin_offset, None);
    }

    #[test]
    fn virtual_rdb_bin_route_uses_no_underscore_suffix() {
        let replacement = super::virtual_memory_replacement(
            "CharacterEditor",
            &format!("CharacterEditor.rdb.bin{}", super::PRIVATE_MODEL_BIN_SUFFIX),
            oppw4_rdb::ReplacementSource::Memory {
                name: "model".to_string(),
                bytes: b"g1m".to_vec(),
            },
        );

        assert_eq!(replacement.file_name, "CharacterEditor.rdb.bin120");
        assert_eq!(replacement.hash, super::PRIVATE_MODEL_HASH);
    }

    #[test]
    fn reserves_law_slot5_assets_from_global_runtime() {
        assert!(super::is_reserved_private_asset_name("MPLC026_Law.g1m"));
        assert!(super::is_reserved_private_asset_name(
            "800_294_face_law_dressrosa_External_00.g1t"
        ));
        assert!(!super::is_reserved_private_asset_name(
            "MDLC038_Zoro_Wa.g1m"
        ));
    }
}
