use std::sync::OnceLock;

use serde::Deserialize;

const EMBEDDED_CHARACTERS_JSON: &str = include_str!("../data/characters.json");

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Character {
    pub playable_id: Option<u16>,
    #[serde(default)]
    pub runtime_id: Option<u16>,
    #[serde(default)]
    pub boss_runtime_id: Option<u16>,
    pub model_id: u16,
    pub canonical: String,
    pub display_name: String,
    pub model_stem: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CharacterDataError {
    InvalidJson(String),
    Empty,
    MissingRequiredField { index: usize, field: &'static str },
}

static CHARACTERS: OnceLock<Vec<Character>> = OnceLock::new();

pub fn all() -> &'static [Character] {
    CHARACTERS
        .get_or_init(|| {
            parse_characters_json(EMBEDDED_CHARACTERS_JSON).expect("embedded characters json")
        })
        .as_slice()
}

pub fn find(query: &str) -> Option<&'static Character> {
    let query = normalize(query);
    if query.is_empty() {
        return None;
    }

    all().iter().find(|character| {
        normalize(&character.canonical) == query
            || normalize(&character.display_name) == query
            || normalize(&character.model_stem) == query
            || character
                .aliases
                .iter()
                .any(|alias| normalize(alias) == query)
    })
}

pub fn find_by_id(id: u16) -> Option<&'static Character> {
    all().iter().find(|character| {
        character.model_id == id
            || character.playable_id == Some(id)
            || character.runtime_id == Some(id)
            || character.boss_runtime_id == Some(id)
    })
}

pub fn parse_characters_json(text: &str) -> Result<Vec<Character>, CharacterDataError> {
    let characters: Vec<Character> = serde_json::from_str(text)
        .map_err(|error| CharacterDataError::InvalidJson(error.to_string()))?;
    validate_characters(&characters)?;
    Ok(characters)
}

fn validate_characters(characters: &[Character]) -> Result<(), CharacterDataError> {
    if characters.is_empty() {
        return Err(CharacterDataError::Empty);
    }

    for (index, character) in characters.iter().enumerate() {
        if character.canonical.trim().is_empty() {
            return Err(CharacterDataError::MissingRequiredField {
                index,
                field: "canonical",
            });
        }
        if character.display_name.trim().is_empty() {
            return Err(CharacterDataError::MissingRequiredField {
                index,
                field: "display_name",
            });
        }
        if character.model_stem.trim().is_empty() {
            return Err(CharacterDataError::MissingRequiredField {
                index,
                field: "model_stem",
            });
        }
    }

    Ok(())
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
    fn embedded_json_loads() {
        assert!(all().len() >= 100);
    }

    #[test]
    fn exposes_linkdata_ids() {
        let law = find("law").unwrap();
        assert_eq!(law.playable_id, Some(22));
        assert_eq!(law.runtime_id, Some(26));
        assert_eq!(law.boss_runtime_id, Some(26));
        assert_eq!(law.model_id, 26);

        let zoro = find("zoro").unwrap();
        assert_eq!(zoro.playable_id, Some(1));
        assert_eq!(zoro.runtime_id, Some(1));
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
    fn finds_by_any_known_id() {
        assert_eq!(
            find_by_id(26).map(|character| character.canonical.as_str()),
            Some("law")
        );
        assert_eq!(
            find_by_id(22).map(|character| character.canonical.as_str()),
            Some("law")
        );
        assert_eq!(
            find_by_id(107).map(|character| character.canonical.as_str()),
            Some("kaku")
        );
        assert!(find_by_id(u16::MAX).is_none());
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
        let kaku = find("kaku").unwrap();
        assert_eq!(kaku.model_id, 58);
        assert_eq!(kaku.runtime_id, Some(107));
        assert_eq!(kaku.boss_runtime_id, Some(80));
        assert_eq!(find("jabra").map(|character| character.model_id), Some(59));
        assert_eq!(find("blueno").map(|character| character.model_id), Some(60));
        assert_eq!(
            find("bon clay").map(|character| character.model_id),
            Some(56)
        );
    }

    #[test]
    fn parser_rejects_empty_data() {
        assert_eq!(parse_characters_json("[]"), Err(CharacterDataError::Empty));
    }

    #[test]
    fn rejects_unknown_names() {
        assert!(find("").is_none());
        assert!(find("not a real character").is_none());
    }
}
