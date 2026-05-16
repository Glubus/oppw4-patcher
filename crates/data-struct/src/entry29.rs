use crate::{inflate_linkdata_entry, Result};

pub const LINKDATA_ENTRY_INDEX: usize = 29;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DlcEntry {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DlcCostumeCode {
    pub code: String,
    pub pack: u16,
    pub variant: u16,
    pub owner: u16,
    pub costume_index: u16,
}

impl DlcEntry {
    pub fn parse(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    pub fn from_linkdata_bytes(linkdata: &[u8]) -> Result<Self> {
        let bytes = inflate_linkdata_entry(linkdata, LINKDATA_ENTRY_INDEX)?;
        Ok(Self::parse(bytes))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn append_costume_code(&mut self, code: &str) {
        if !self.bytes.ends_with(&[0]) {
            self.bytes.push(0);
        }
        self.bytes.extend_from_slice(code.as_bytes());
        self.bytes.push(0);
    }

    pub fn costume_codes(&self) -> Vec<DlcCostumeCode> {
        ascii_strings(&self.bytes, 8)
            .into_iter()
            .filter_map(|value| parse_dlc_costume_code(&value))
            .collect()
    }

    pub fn find_costume_codes(&self, owner: u16, variant: u16) -> Vec<DlcCostumeCode> {
        self.costume_codes()
            .into_iter()
            .filter(|code| code.owner == owner && code.variant == variant)
            .collect()
    }
}

fn parse_dlc_costume_code(value: &str) -> Option<DlcCostumeCode> {
    let rest = value.strip_prefix("DLC_COSTUME_")?;
    let mut parts = rest.split('_');
    let pack = parts.next()?.parse().ok()?;
    let variant = parts.next()?.parse().ok()?;
    let owner = parts.next()?.parse().ok()?;
    let costume_index = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(DlcCostumeCode {
        code: value.to_string(),
        pack,
        variant,
        owner,
        costume_index,
    })
}

fn ascii_strings(bytes: &[u8], min_len: usize) -> Vec<String> {
    let mut strings = Vec::new();
    let mut start = None;
    for (index, byte) in bytes.iter().copied().chain([0]).enumerate() {
        let printable = byte.is_ascii_graphic() || byte == b' ';
        match (start, printable) {
            (None, true) => start = Some(index),
            (Some(_), true) => {}
            (Some(start_index), false) => {
                if index.saturating_sub(start_index) >= min_len {
                    strings.push(String::from_utf8_lossy(&bytes[start_index..index]).to_string());
                }
                start = None;
            }
            (None, false) => {}
        }
    }
    strings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_dlc_costume_codes() {
        let entry = DlcEntry::parse(b"\0DLC_COSTUME_001_699_026_004\0noise\0".to_vec());

        let codes = entry.find_costume_codes(26, 699);

        assert_eq!(
            codes,
            vec![DlcCostumeCode {
                code: "DLC_COSTUME_001_699_026_004".to_string(),
                pack: 1,
                variant: 699,
                owner: 26,
                costume_index: 4,
            }]
        );
    }

    #[test]
    fn appends_dlc_costume_code() {
        let mut entry = DlcEntry::parse(b"\0DLC_COSTUME_006_586_026_003\0".to_vec());

        entry.append_costume_code("DLC_COSTUME_007_699_026_004");

        assert_eq!(entry.find_costume_codes(26, 699).len(), 1);
        assert!(entry
            .bytes()
            .windows(b"DLC_COSTUME_007_699_026_004".len())
            .any(|window| window == b"DLC_COSTUME_007_699_026_004"));
    }
}
