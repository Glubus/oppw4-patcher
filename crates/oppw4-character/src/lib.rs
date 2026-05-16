#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Character {
    pub id: u16,
    pub canonical: &'static str,
    pub display_name: &'static str,
    pub model_stem: &'static str,
    pub aliases: &'static [&'static str],
}

const CHARACTERS: &[Character] = &[
    Character {
        id: 0,
        canonical: "luffy",
        display_name: "Monkey D. Luffy",
        model_stem: "MPLC000_Luffy",
        aliases: &["mugiwara", "straw_hat", "monkey_d_luffy"],
    },
    Character {
        id: 1,
        canonical: "zoro",
        display_name: "Roronoa Zoro",
        model_stem: "MPLC001_Zoro",
        aliases: &["roronoa_zoro", "zorro"],
    },
    Character {
        id: 2,
        canonical: "nami",
        display_name: "Nami",
        model_stem: "MPLC002_Nami",
        aliases: &[],
    },
    Character {
        id: 3,
        canonical: "usopp",
        display_name: "Usopp",
        model_stem: "MPLC003_Usopp",
        aliases: &["ussop", "sogeking"],
    },
    Character {
        id: 4,
        canonical: "sanji",
        display_name: "Sanji",
        model_stem: "MPLC004_Sanji",
        aliases: &[],
    },
    Character {
        id: 5,
        canonical: "chopper",
        display_name: "Tony Tony Chopper",
        model_stem: "MPLC005_Chopper",
        aliases: &["tony_tony_chopper"],
    },
    Character {
        id: 6,
        canonical: "robin",
        display_name: "Nico Robin",
        model_stem: "MPLC006_Robin",
        aliases: &["nico_robin"],
    },
    Character {
        id: 7,
        canonical: "franky",
        display_name: "Franky",
        model_stem: "MPLC007_Franky",
        aliases: &[],
    },
    Character {
        id: 8,
        canonical: "brook",
        display_name: "Brook",
        model_stem: "MPLC008_Brook",
        aliases: &[],
    },
    Character {
        id: 9,
        canonical: "ace",
        display_name: "Portgas D. Ace",
        model_stem: "MPLC009_Ace",
        aliases: &["portgas_d_ace"],
    },
    Character {
        id: 10,
        canonical: "hancock",
        display_name: "Boa Hancock",
        model_stem: "MPLC010_Hancock",
        aliases: &["boa_hancock"],
    },
    Character {
        id: 11,
        canonical: "jinbe",
        display_name: "Jinbe",
        model_stem: "MPLC011_Jinbe",
        aliases: &["jimbei", "jinbei"],
    },
    Character {
        id: 12,
        canonical: "newgate",
        display_name: "Edward Newgate",
        model_stem: "MPLC012_Newgate",
        aliases: &["whitebeard", "barbe_blanche", "edward_newgate"],
    },
    Character {
        id: 13,
        canonical: "buggy",
        display_name: "Buggy",
        model_stem: "MPLC013_Buggy",
        aliases: &[],
    },
    Character {
        id: 14,
        canonical: "mihawk",
        display_name: "Dracule Mihawk",
        model_stem: "MPLC014_Mihawk",
        aliases: &["dracule_mihawk"],
    },
    Character {
        id: 25,
        canonical: "garp",
        display_name: "Monkey D. Garp",
        model_stem: "MPLC025_Garp",
        aliases: &["monkey_d_garp"],
    },
    Character {
        id: 26,
        canonical: "law",
        display_name: "Trafalgar Law",
        model_stem: "MPLC026_Law",
        aliases: &["trafalgar_law", "trafalgar_d_water_law"],
    },
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
    fn finds_by_aliases_and_model_stems() {
        assert_eq!(find("zoro").map(|character| character.id), Some(1));
        assert_eq!(
            find("barbe blanche").map(|character| character.id),
            Some(12)
        );
        assert_eq!(find("MPLC026_Law").map(|character| character.id), Some(26));
    }

    #[test]
    fn rejects_unknown_names() {
        assert!(find("").is_none());
        assert!(find("not a real character").is_none());
    }
}
