#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archive {
    CharacterEditor,
    MaterialEditor,
    RRPreview,
    ScreenLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRoute {
    pub archive: Archive,
    pub name: &'static str,
    pub target_name: &'static str,
    pub hash: u32,
    pub size: u64,
    pub sha256: &'static str,
    pub shared_official_hash: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetPackage {
    pub owner: u16,
    pub source_variant: u16,
    pub target_variant: u16,
    pub source_model: u16,
    pub target_model: u16,
    pub preview_mapping: u16,
    pub preview_resource: u16,
    pub requires_private_route: bool,
    pub assets: Vec<AssetRoute>,
}

pub fn law_slot5_base_law_kids_package() -> AssetPackage {
    AssetPackage {
        owner: 26,
        source_variant: 57,
        target_variant: 699,
        source_model: 26,
        target_model: 730,
        preview_mapping: 294,
        preview_resource: 911,
        requires_private_route: true,
        assets: vec![
            AssetRoute {
                archive: Archive::CharacterEditor,
                name: "MPLC026_Law.g1m",
                target_name: "MDLC999_Law_Custom.g1m",
                hash: 0x8df2_d8cb,
                size: 1_943_768,
                sha256: "E6B05FA10CE2D99248B4F6DACC4A91F2F9CE2CCCAB10AD1B9C7B07FFC0E46246",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::MaterialEditor,
                name: "MPR_Bound_Character_MPLC026Law_cloth_kidsalb.g1t",
                target_name: "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsalb.g1t",
                hash: 0x966d_6276,
                size: 699_104,
                sha256: "9E66551B0E383794D790AA999D8DFEF4924DD45498AE87BCEE0BC7115CF2D062",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::MaterialEditor,
                name: "MPR_Bound_Character_MPLC026Law_cloth_kidsnmh.g1t",
                target_name: "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsnmh.g1t",
                hash: 0x81b6_a8a8,
                size: 349_584,
                sha256: "53964758BA557B7113CE0802FB5AE431329C7DCE5710D3187AB84D9AAAB2723A",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::MaterialEditor,
                name: "MPR_Bound_Character_MPLC026Law_cloth_kidsocc.g1t",
                target_name: "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsocc.g1t",
                hash: 0x38a6_472e,
                size: 349_584,
                sha256: "A256D23B17F0104FEE16100B50F203F4E8D60F5464114C9B99B1C2EA05362E11",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::MaterialEditor,
                name: "MPR_Bound_Character_MPLC026Law_cloth_kidsrfr.g1t",
                target_name: "MPR_Bound_Character_MDLC999LawCustom_cloth_kidsrfr.g1t",
                hash: 0xd341_d61d,
                size: 349_584,
                sha256: "D54BB8E53B21399BF8D330B793CB50754E65EB301B40131C1196D8542CD27B87",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::MaterialEditor,
                name: "MPR_Bound_Character_MPLC026Law_skin_kidsalb.g1t",
                target_name: "MPR_Bound_Character_MDLC999LawCustom_skin_kidsalb.g1t",
                hash: 0x8b5c_ca29,
                size: 16_777_272,
                sha256: "5DFD68B6E1BE7DE6817ED43129D5562D6CAB0BF257C6E785CE1FAD542CB49123",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::MaterialEditor,
                name: "MPR_Bound_Character_MPLC026Law_skin_kidsnmh.g1t",
                target_name: "MPR_Bound_Character_MDLC999LawCustom_skin_kidsnmh.g1t",
                hash: 0x2798_f5b7,
                size: 22_369_592,
                sha256: "A8FB9436C513B46A400D2FF9B4E44F70842918C235607715B3D0A61B6446337F",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::MaterialEditor,
                name: "MPR_Bound_Character_MPLC026Law_skin_kidsocc.g1t",
                target_name: "MPR_Bound_Character_MDLC999LawCustom_skin_kidsocc.g1t",
                hash: 0x9098_6e71,
                size: 22_369_592,
                sha256: "DEA9C81C7CDA2D92DD11334D2B61C77C27C527346C17573BCC162C2D7EC45B37",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::MaterialEditor,
                name: "MPR_Bound_Character_MPLC026Law_skin_kidss4m.g1t",
                target_name: "MPR_Bound_Character_MDLC999LawCustom_skin_kidss4m.g1t",
                hash: 0xbb2a_9834,
                size: 22_369_592,
                sha256: "3C75CFDC9BF03F00595392F0D11F32AC3B57EA5F153EE7298E5CCB427DC07C29",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::ScreenLayout,
                name: "800_294_face_law_dressrosa_External_00.g1t",
                target_name: "800_294_face_law_dressrosa_External_00.g1t",
                hash: 0x359b_9672,
                size: 2_097_208,
                sha256: "14EA14DCA40DEBCE5A9513B27AAC93538DABDEC12DFCEAA0CE41911A98A3B185",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::ScreenLayout,
                name: "801_294_chara_law_dressrosa_External_00.g1t",
                target_name: "801_294_chara_law_dressrosa_External_00.g1t",
                hash: 0x3bff_0f13,
                size: 2_097_208,
                sha256: "9EE1DFECD5B94841A9441EE30FD236EEECBC755E66C1369838D31BBA7F7E788B",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::RRPreview,
                name: "CE1_0080_STYLE_IMPACT_HAO.g1e",
                target_name: "CE1_0080_STYLE_IMPACT_HAO.g1e",
                hash: 0xf1d7_3d2e,
                size: 60_244,
                sha256: "961E5B10385B6A074890DB7CBF75483CA06748B28ECCC63A35577F4C79338DC7",
                shared_official_hash: true,
            },
            AssetRoute {
                archive: Archive::RRPreview,
                name: "CE1_0080_STYLE_IMPACT_HAO.ktid",
                target_name: "CE1_0080_STYLE_IMPACT_HAO.ktid",
                hash: 0x3dcc_d115,
                size: 352,
                sha256: "AB75152A2CFD8385D728BEF93C9FB6E50722FE1488FAA7457DB962AB87A46CF4",
                shared_official_hash: true,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn law_slot5_package_tracks_model_and_all_eight_textures() {
        let package = law_slot5_base_law_kids_package();

        assert_eq!(package.owner, 26);
        assert_eq!(package.target_variant, 699);
        assert_eq!(package.target_model, 730);
        assert_eq!(package.source_variant, 57);
        assert_eq!(package.source_model, 26);
        assert_eq!(package.preview_mapping, 294);
        assert_eq!(package.preview_resource, 911);
        assert!(package.requires_private_route);
        assert_eq!(package.assets.len(), 13);
        assert_eq!(
            package
                .assets
                .iter()
                .filter(|asset| asset.archive == Archive::CharacterEditor)
                .count(),
            1
        );
        assert_eq!(
            package
                .assets
                .iter()
                .filter(|asset| asset.archive == Archive::MaterialEditor)
                .count(),
            8
        );
        assert_eq!(
            package
                .assets
                .iter()
                .filter(|asset| asset.archive == Archive::ScreenLayout)
                .count(),
            2
        );
        assert_eq!(
            package
                .assets
                .iter()
                .filter(|asset| asset.archive == Archive::RRPreview)
                .count(),
            2
        );
        assert!(package
            .assets
            .iter()
            .all(|asset| asset.shared_official_hash));
        assert_eq!(package.assets[0].name, "MPLC026_Law.g1m");
        assert_eq!(package.assets[0].target_name, "MDLC999_Law_Custom.g1m");
        assert_eq!(package.assets[0].hash, 0x8df2_d8cb);
    }

    #[test]
    fn law_slot5_package_lists_known_texture_hashes() {
        let package = law_slot5_base_law_kids_package();
        let hashes = package
            .assets
            .iter()
            .filter(|asset| asset.archive == Archive::MaterialEditor)
            .map(|asset| asset.hash)
            .collect::<Vec<_>>();

        assert_eq!(
            hashes,
            vec![
                0x966d_6276,
                0x81b6_a8a8,
                0x38a6_472e,
                0xd341_d61d,
                0x8b5c_ca29,
                0x2798_f5b7,
                0x9098_6e71,
                0xbb2a_9834,
            ]
        );
    }
}
