#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Character {
    pub playable_id: Option<u16>,
    pub model_id: u16,
    pub canonical: &'static str,
    pub display_name: &'static str,
    pub model_stem: &'static str,
    pub aliases: &'static [&'static str],
}

macro_rules! character {
    ($playable_id:expr, $model_id:expr, $canonical:literal, $display_name:literal, $model_stem:literal) => {
        Character {
            playable_id: $playable_id,
            model_id: $model_id,
            canonical: $canonical,
            display_name: $display_name,
            model_stem: $model_stem,
            aliases: &[],
        }
    };
    ($playable_id:expr, $model_id:expr, $canonical:literal, $display_name:literal, $model_stem:literal, [$($alias:literal),* $(,)?]) => {
        Character {
            playable_id: $playable_id,
            model_id: $model_id,
            canonical: $canonical,
            display_name: $display_name,
            model_stem: $model_stem,
            aliases: &[$($alias),*],
        }
    };
}

const CHARACTERS: &[Character] = &[
    // LINKDATA_A entry 32 section 6 model/resource IDs, cross-referenced with
    // entry 32 section 4 playable IDs when present.
    character!(
        Some(0),
        0,
        "luffy",
        "Monkey D. Luffy",
        "MPLC000_Luffy",
        ["mugiwara", "straw_hat", "monkey_d_luffy"]
    ),
    character!(
        Some(1),
        1,
        "zoro",
        "Roronoa Zoro",
        "MPLC001_Zoro",
        ["roronoa_zoro", "zorro"]
    ),
    character!(Some(2), 2, "nami", "Nami", "MPLC002_Nami"),
    character!(Some(3), 3, "usopp", "Usopp", "MPLC003_Usopp"),
    character!(Some(4), 4, "sanji", "Sanji", "MPLC004_Sanji"),
    character!(Some(5), 5, "chopper", "Chopper", "MPLC005_Chopper"),
    character!(Some(6), 6, "robin", "Robin", "MPLC006_Robin"),
    character!(Some(7), 7, "franky", "Franky", "MPLC007_Franky"),
    character!(Some(8), 8, "brook", "Brook", "MPLC008_Brook"),
    character!(Some(9), 9, "ace", "Ace", "MPLC009_Ace", ["portgas_d_ace"]),
    character!(
        Some(10),
        10,
        "hancock",
        "Boa Hancock",
        "MPLC010_Hancock",
        ["boa_hancock"]
    ),
    character!(
        Some(11),
        11,
        "jimbei",
        "Jimbei",
        "MPLC011_Jimbei",
        ["jinbe", "jinbei"]
    ),
    character!(
        Some(12),
        12,
        "newgate",
        "Edward Newgate",
        "MPLC012_Newgate",
        ["whitebeard", "barbe_blanche", "edward_newgate"]
    ),
    character!(Some(13), 13, "buggy", "Buggy", "MPLC013_Buggy"),
    character!(
        Some(14),
        14,
        "mihawk",
        "Mihawk",
        "MPLC014_Mihawk",
        ["dracule_mihawk"]
    ),
    character!(Some(15), 15, "crocodile", "Crocodile", "MPLC015_Crocodile"),
    character!(
        Some(16),
        16,
        "teach",
        "Marshall D. Teach",
        "MPLC016_Teach",
        ["blackbeard", "marshall_d_teach"]
    ),
    character!(None, 17, "kuma", "Kuma", "MPLC017_Kuma"),
    character!(
        Some(17),
        18,
        "kizaru",
        "Kizaru",
        "MPLC018_Kizaru",
        ["borsalino"]
    ),
    character!(
        Some(18),
        19,
        "aokiji",
        "Aokiji",
        "MPLC019_Aokiji",
        ["kuzan"]
    ),
    character!(
        Some(19),
        20,
        "akainu",
        "Akainu",
        "MPLC020_Akainu",
        ["sakazuki"]
    ),
    character!(Some(20), 23, "smoker", "Smoker", "MPLC023_Smoker"),
    character!(Some(21), 24, "marco", "Marco", "MPLC024_Marco"),
    character!(
        Some(48),
        25,
        "garp",
        "Monkey D. Garp",
        "MPLC025_Garp",
        ["monkey_d_garp"]
    ),
    character!(
        Some(22),
        26,
        "law",
        "Law",
        "MPLC026_Law",
        ["trafalgar_law", "trafalgar_d_water_law"]
    ),
    character!(
        Some(23),
        27,
        "doflamingo",
        "Donquixote Doflamingo",
        "MPLC027_Doflamingo",
        ["doffy", "donquixote_doflamingo"]
    ),
    character!(Some(24), 28, "tashigi", "Tashigi", "MPLC028_Tashigi"),
    character!(Some(25), 30, "fujitora", "Fujitora", "MPLC030_Fujitora"),
    character!(Some(26), 31, "sabo", "Sabo", "MPLC031_Sabo"),
    character!(
        Some(27),
        32,
        "lucci",
        "Lucci",
        "MPLC032_Lucci",
        ["rob_lucci"]
    ),
    character!(Some(28), 35, "ivankov", "Ivankov", "MPLC035_Ivankov"),
    character!(Some(29), 36, "shanks", "Shanks", "MPLC036_Shanks"),
    character!(
        Some(30),
        37,
        "bartolomeo",
        "Bartolomeo",
        "MPLC037_Bartolomeo"
    ),
    character!(Some(31), 38, "cavendish", "Cavendish", "MPLC038_Cavendish"),
    character!(Some(32), 40, "carrot", "Carrot", "MPLC040_Carrot"),
    character!(Some(33), 41, "reiju", "Reiju", "MPLC041_Reiju"),
    character!(Some(34), 42, "ichiji", "Ichiji", "MPLC042_Ichiji"),
    character!(Some(35), 43, "niji", "Niji", "MPLC043_Niji"),
    character!(Some(36), 44, "yonji", "Yonji", "MPLC044_Yonji"),
    character!(
        Some(37),
        45,
        "bege",
        "Bege",
        "MPLC045_Bege",
        ["capone_bege"]
    ),
    character!(
        Some(38),
        46,
        "linlin",
        "Charlotte Linlin",
        "MPLC046_Linlin",
        ["big_mom", "bigmom", "charlotte_linlin"]
    ),
    character!(
        Some(39),
        47,
        "katakuri",
        "Katakuri",
        "MPLC047_Katakuri",
        ["charlotte_katakuri"]
    ),
    character!(Some(40), 48, "kaido", "Kaido", "MPLC048_Kaido"),
    character!(
        Some(41),
        49,
        "kid",
        "Eustass Kid",
        "MPLC049_Kid",
        ["eustass_kid", "kidd"]
    ),
    character!(Some(42), 50, "hawkins", "Hawkins", "MPLC050_Hawkins_NW"),
    character!(
        Some(49),
        56,
        "mr2",
        "Mr. 2",
        "MNPC005_Mr2",
        ["bon_clay", "bonclay"]
    ),
    character!(Some(50), 57, "mr3", "Mr. 3", "MNPC006_Mr3", ["galdino"]),
    character!(Some(51), 58, "kaku", "Kaku", "MNPC007_Kaku"),
    character!(Some(52), 59, "jabra", "Jabra", "MNPC008_Jabra"),
    character!(Some(53), 60, "blueno", "Blueno", "MNPC009_Blueno"),
    character!(Some(55), 62, "pacifista", "Pacifista", "MNPC011_Pacifista"),
    character!(Some(56), 70, "mr1", "Mr. 1", "MNPC019_Mr1", ["daz_bones"]),
    character!(Some(57), 71, "bellamy", "Bellamy", "MNPC020_Bellamy_NW"),
    character!(Some(58), 76, "burgess", "Burgess", "MNPC025_Burgess"),
    character!(Some(59), 77, "coby", "Coby", "MNPC026_Coby", ["koby"]),
    character!(
        Some(60),
        79,
        "sengoku_chg",
        "Sengoku Change",
        "MNPC028_Sengoku_Chg"
    ),
    character!(
        Some(61),
        80,
        "kinemon_npc",
        "Kinemon NPC",
        "MNPC029_Kinemon"
    ),
    character!(Some(62), 83, "pica_giant", "Pica Giant", "MNPC032_PicaG"),
    character!(Some(63), 86, "pica", "Pica", "MNPC035_PicaC"),
    character!(Some(66), 89, "diamante", "Diamante", "MNPC038_Diamante"),
    character!(Some(67), 91, "jack_chg", "Jack Change", "MNPC040_Jack_Chg"),
    character!(Some(68), 92, "perospero", "Perospero", "MNPC041_Perospero"),
    character!(
        Some(69),
        93,
        "drake_chg_npc",
        "Drake Change NPC",
        "MNPC042_Drake_Chg"
    ),
    character!(Some(70), 65, "jozu", "Jozu", "MNPC014_Jozu"),
    character!(Some(153), 280, "smoothie", "Smoothie", "MDLC041_Smoothie"),
    character!(Some(155), 281, "cracker", "Cracker", "MDLC042_Cracker"),
    character!(
        Some(156),
        282,
        "cracker_b",
        "Cracker B",
        "MDLC043_Cracker_B"
    ),
    character!(Some(157), 283, "judge", "Judge", "MDLC044_Judge_PC"),
    character!(Some(158), 284, "drake", "Drake", "MDLC045_Drake_PC"),
    character!(Some(160), 285, "killer", "Killer", "MDLC046_Killer"),
    character!(Some(161), 286, "urouge", "Urouge", "MDLC047_Urouge"),
    character!(Some(163), 287, "kiku", "Kiku", "MDLC048_Kiku"),
    character!(Some(164), 288, "kinemon", "Kinemon", "MDLC049_Kinemon_PC"),
    character!(Some(165), 289, "oden", "Oden", "MDLC050_Oden"),
    character!(
        Some(159),
        290,
        "drake_chg",
        "Drake Chg",
        "MDLC051_Drake_Chg_PC"
    ),
    character!(
        Some(162),
        291,
        "urouge_chg",
        "Urouge Chg",
        "MDLC052_Urouge_Chg"
    ),
    character!(
        Some(166),
        293,
        "luffy_oni",
        "Monkey D. Luffy (Onigashima)",
        "MDLC054_Luffy_Oni",
        ["oni_luffy"]
    ),
    character!(
        Some(167),
        294,
        "luffy_n_oni",
        "Monkey D. Luffy (New Onigashima)",
        "MDLC055_Luffy_N_Oni"
    ),
    character!(
        Some(168),
        295,
        "luffy_n_gear5",
        "Monkey D. Luffy Gear 5",
        "MDLC056_Luffy_N_Gear5",
        ["gear5", "gear_5"]
    ),
    character!(
        Some(169),
        296,
        "kaido_hb",
        "Kaido Hybrid",
        "MDLC057_Kaido_HB",
        ["hybrid_kaido"]
    ),
    character!(Some(171), 297, "yamato", "Yamato", "MDLC058_Yamato"),
    character!(
        Some(172),
        298,
        "yamato_hb",
        "Yamato Hybrid",
        "MDLC059_Yamato_HB",
        ["hybrid_yamato"]
    ),
    character!(Some(174), 299, "uta", "Uta", "MDLC060_Uta"),
    character!(
        Some(175),
        300,
        "uta_armor",
        "Uta Armor",
        "MDLC061_Uta_Armor"
    ),
    character!(Some(176), 301, "roger", "Roger", "MDLC062_Roger"),
    character!(
        Some(177),
        302,
        "shanks_red",
        "Shanks (Red)",
        "MDLC063_Shanks_RED",
        ["film_red_shanks"]
    ),
    character!(
        Some(178),
        303,
        "coby_red",
        "Coby Red",
        "MDLC064_Coby_RED",
        ["koby_red"]
    ),
    character!(
        Some(179),
        304,
        "garp_yng",
        "Monkey D. Garp (Young)",
        "MDLC065_Garp_YNG",
        ["young_garp"]
    ),
    character!(
        Some(180),
        305,
        "rayleigh_yng",
        "Rayleigh Young",
        "MDLC066_Rayleigh_YNG",
        ["young_rayleigh"]
    ),
    character!(
        Some(170),
        306,
        "kaido_d2",
        "Kaido Dragon",
        "MDLC067_Kaido_D2",
        ["dragon_kaido"]
    ),
    character!(Some(173), 307, "yamato_s", "Yamato S", "MDLC068_Yamato_S"),
    character!(Some(181), 310, "uta_note", "Uta Note", "MDLC071_Uta_Note"),
    character!(
        Some(182),
        311,
        "lucci_cp0",
        "Lucci CP0",
        "MDLC072_Lucci_CP0",
        ["cp0_lucci"]
    ),
    character!(
        Some(183),
        313,
        "lucci_c_bs",
        "Lucci Beast",
        "MDLC074_Lucci_C_BS"
    ),
    character!(
        Some(184),
        314,
        "bonney_egh",
        "Bonney",
        "MDLC075_Bonney_EGH",
        ["bonney"]
    ),
    character!(
        Some(185),
        316,
        "bonney_g",
        "Bonney Gear",
        "MDLC077_Bonney_G",
        ["bonney_gear"]
    ),
    character!(
        Some(186),
        318,
        "bonney_frf",
        "Bonney Future",
        "MDLC079_Bonney_FRF",
        ["bonney_future"]
    ),
    character!(
        Some(187),
        319,
        "ssnake",
        "S-Snake",
        "MDLC080_SSnake",
        ["s_snake", "s-snake"]
    ),
    character!(
        Some(188),
        320,
        "kuma_egh",
        "Kuma Egghead",
        "MDLC081_Kuma_EGH",
        ["egghead_kuma"]
    ),
    character!(
        Some(189),
        321,
        "luffy_n_bny",
        "Monkey D. Luffy Bonney",
        "MDLC082_Luffy_N_BNY",
        ["luffy_bonney"]
    ),
    character!(
        Some(190),
        322,
        "eneru_d",
        "Eneru Divine",
        "MDLC083_Eneru_D",
        ["eneru", "enel"]
    ),
    character!(
        Some(191),
        323,
        "eneru_a_d",
        "Eneru Amaru",
        "MDLC084_Eneru_A_D",
        ["enel_amaru"]
    ),
    character!(
        Some(192),
        324,
        "eneru_th_b",
        "Eneru Thunder Black",
        "MDLC085_Eneru_TH_B",
        ["enel_thunder_black"]
    ),
    character!(
        Some(193),
        325,
        "eneru_th_w",
        "Eneru Thunder White",
        "MDLC086_Eneru_TH_W",
        ["enel_thunder_white"]
    ),
    character!(
        Some(194),
        326,
        "eneru_th_d",
        "Eneru Thunder Divine",
        "MDLC087_Eneru_TH_D",
        ["enel_thunder_divine"]
    ),
    character!(Some(195), 327, "z_d", "Z", "MDLC088_Z_D", ["z", "zephyr"]),
    character!(Some(196), 328, "king", "King", "MDLC089_King"),
    character!(
        Some(197),
        329,
        "king_bs",
        "King Beast",
        "MDLC090_King_BS",
        ["king_beast"]
    ),
    character!(
        Some(198),
        330,
        "king_dr",
        "King Dragon",
        "MDLC091_King_DR",
        ["king_dragon"]
    ),
    character!(
        Some(199),
        331,
        "gatling_n",
        "Gatling N",
        "MDLC092_Gatling_N"
    ),
];

pub fn all() -> &'static [Character] {
    CHARACTERS
}

pub fn find(query: &str) -> Option<&'static Character> {
    let query = normalize(query);
    if query.is_empty() {
        return None;
    }

    CHARACTERS.iter().find(|character| {
        normalize(character.canonical) == query
            || normalize(character.display_name) == query
            || normalize(character.model_stem) == query
            || character
                .aliases
                .iter()
                .any(|alias| normalize(alias) == query)
    })
}

fn normalize(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut last_was_sep = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
            last_was_sep = false;
        } else if !last_was_sep {
            output.push('_');
            last_was_sep = true;
        }
    }
    output.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_linkdata_ids() {
        let law = find("law").unwrap();
        assert_eq!(law.playable_id, Some(22));
        assert_eq!(law.model_id, 26);

        let zoro = find("zoro").unwrap();
        assert_eq!(zoro.playable_id, Some(1));
        assert_eq!(zoro.model_id, 1);
    }

    #[test]
    fn finds_by_aliases_and_model_stems() {
        assert_eq!(
            find("barbe blanche").map(|character| character.model_id),
            Some(12)
        );
        assert_eq!(
            find("MPLC026_Law").map(|character| character.model_id),
            Some(26)
        );
        assert_eq!(
            find("gear 5").map(|character| character.model_id),
            Some(295)
        );
    }

    #[test]
    fn includes_late_dlc_and_missing_mplc_entries() {
        assert_eq!(find("kuma").map(|character| character.model_id), Some(17));
        assert_eq!(
            find("bartolomeo").map(|character| character.model_id),
            Some(37)
        );
        assert_eq!(
            find("bonney").map(|character| character.model_id),
            Some(314)
        );
        assert_eq!(find("z").map(|character| character.model_id), Some(327));
        assert_eq!(find("king").map(|character| character.model_id), Some(328));
        assert_eq!(find("eneru").map(|character| character.model_id), Some(322));
    }

    #[test]
    fn includes_linkdata_npc_rows_with_model_ids() {
        assert_eq!(find("kaku").map(|character| character.model_id), Some(58));
        assert_eq!(find("jabra").map(|character| character.model_id), Some(59));
        assert_eq!(find("blueno").map(|character| character.model_id), Some(60));
        assert_eq!(
            find("bon clay").map(|character| character.model_id),
            Some(56)
        );
    }

    #[test]
    fn rejects_unknown_names() {
        assert!(find("").is_none());
        assert!(find("not a real character").is_none());
    }
}
