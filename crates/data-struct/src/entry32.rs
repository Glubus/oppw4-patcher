use crate::{checked_slice, read_u32, DataStructError, Result};

pub const LINKDATA_ENTRY_INDEX: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry32 {
    sections: Vec<Entry32Section>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry32Section {
    pub section: usize,
    pub offset: usize,
    pub strings: Vec<Entry32String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry32String {
    pub id: usize,
    pub relative_offset: usize,
    pub byte_len: usize,
    pub value: String,
}

impl Entry32 {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let section_count = read_u32(bytes, 0, "entry32 section count")? as usize;
        let mut sections = Vec::with_capacity(section_count);
        for section in 0..section_count {
            let offset = read_u32(bytes, 4 + section * 4, "entry32 section offset")? as usize;
            sections.push(parse_section(bytes, section, offset)?);
        }
        Ok(Self { sections })
    }

    pub fn from_linkdata_bytes(linkdata: &[u8]) -> Result<Self> {
        let index = oppw4_rdb::parse_linkdata(linkdata)
            .map_err(|error| DataStructError::LinkData(format!("{error:?}")))?;
        let entry = index
            .entries
            .get(LINKDATA_ENTRY_INDEX)
            .ok_or(DataStructError::EntryMissing(LINKDATA_ENTRY_INDEX))?;
        let bytes = oppw4_rdb::inflate_linkdata_entry(linkdata, entry)
            .map_err(|error| DataStructError::LinkData(format!("{error:?}")))?;
        Self::parse(&bytes)
    }

    pub fn section(&self, section: usize) -> Option<&Entry32Section> {
        self.sections.get(section)
    }

    pub fn name(&self, section: usize, id: usize) -> Option<&str> {
        self.section(section)?
            .strings
            .get(id)
            .map(|item| item.value.as_str())
    }

    pub fn sections(&self) -> &[Entry32Section] {
        &self.sections
    }

    pub fn set_name(&mut self, section: usize, id: usize, value: impl Into<String>) -> Result<()> {
        let Some(section) = self.sections.get_mut(section) else {
            return Err(DataStructError::OutOfBounds {
                what: "entry32 section",
                offset: section,
                size: 1,
                len: self.sections.len(),
            });
        };
        let len = section.strings.len();
        let Some(item) = section.strings.get_mut(id) else {
            return Err(DataStructError::OutOfBounds {
                what: "entry32 string id",
                offset: id,
                size: 1,
                len,
            });
        };
        item.value = value.into();
        item.byte_len = item.value.len() + 1;
        Ok(())
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let header_len = 4 + self.sections.len() * 4;
        let mut section_bytes = Vec::with_capacity(self.sections.len());
        for section in &self.sections {
            if section.strings.is_empty() {
                section_bytes.push(Vec::new());
            } else {
                section_bytes.push(rebuild_section(section));
            }
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.sections.len() as u32).to_le_bytes());
        let mut cursor = header_len;
        for section in &section_bytes {
            if section.is_empty() {
                bytes.extend_from_slice(&0u32.to_le_bytes());
            } else {
                bytes.extend_from_slice(&(cursor as u32).to_le_bytes());
                cursor += section.len();
            }
        }
        for section in section_bytes {
            bytes.extend_from_slice(&section);
        }
        bytes
    }
}

impl Entry32Section {
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

fn parse_section(bytes: &[u8], section: usize, offset: usize) -> Result<Entry32Section> {
    if offset == 0 || offset >= bytes.len() {
        return Ok(Entry32Section {
            section,
            offset,
            strings: Vec::new(),
        });
    }

    let count = read_u32(bytes, offset, "entry32 string count")? as usize;
    let mut strings = Vec::with_capacity(count);
    for id in 0..count {
        let row = offset + 4 + id * 8;
        let relative_offset = read_u32(bytes, row, "entry32 string relative offset")? as usize;
        let byte_len = read_u32(bytes, row + 4, "entry32 string byte len")? as usize;
        let start = offset + relative_offset;
        let slice = checked_slice(bytes, start, byte_len, "entry32 string bytes")?;
        strings.push(Entry32String {
            id,
            relative_offset,
            byte_len,
            value: trim_nul(slice),
        });
    }

    Ok(Entry32Section {
        section,
        offset,
        strings,
    })
}

fn trim_nul(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn rebuild_section(section: &Entry32Section) -> Vec<u8> {
    let count = section.strings.len();
    let table_len = 4 + count * 8;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(count as u32).to_le_bytes());
    let mut payload = Vec::new();
    for item in &section.strings {
        let encoded = item
            .value
            .as_bytes()
            .iter()
            .copied()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let relative_offset = table_len + payload.len();
        bytes.extend_from_slice(&(relative_offset as u32).to_le_bytes());
        bytes.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        payload.extend_from_slice(&encoded);
    }
    bytes.extend_from_slice(&payload);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry32_section(strings: &[&str]) -> Vec<u8> {
        let count = strings.len() as u32;
        let table_len = 4 + strings.len() * 8;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&count.to_le_bytes());
        let mut cursor = table_len;
        let mut payload = Vec::new();
        for value in strings {
            let encoded = value
                .as_bytes()
                .iter()
                .copied()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>();
            bytes.extend_from_slice(&(cursor as u32).to_le_bytes());
            bytes.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
            cursor += encoded.len();
            payload.extend_from_slice(&encoded);
        }
        bytes.extend_from_slice(&payload);
        bytes
    }

    fn entry32_bytes(sections: &[Vec<u8>]) -> Vec<u8> {
        let header_len = 4 + sections.len() * 4;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(sections.len() as u32).to_le_bytes());
        let mut cursor = header_len;
        for section in sections {
            if section.is_empty() {
                bytes.extend_from_slice(&0u32.to_le_bytes());
            } else {
                bytes.extend_from_slice(&(cursor as u32).to_le_bytes());
                cursor += section.len();
            }
        }
        for section in sections {
            if !section.is_empty() {
                bytes.extend_from_slice(section);
            }
        }
        bytes
    }

    #[test]
    fn parses_entry32_sections_and_names() {
        let bytes = entry32_bytes(&[
            entry32_section(&["zero"]),
            Vec::new(),
            entry32_section(&["alpha", "beta"]),
        ]);

        let entry32 = Entry32::parse(&bytes).unwrap();

        assert_eq!(entry32.sections().len(), 3);
        assert_eq!(entry32.name(0, 0), Some("zero"));
        assert_eq!(entry32.name(2, 0), Some("alpha"));
        assert_eq!(entry32.name(2, 1), Some("beta"));
        assert_eq!(entry32.section(1).unwrap().len(), 0);
    }

    #[test]
    fn entry32_string_keeps_byte_length_and_relative_offset() {
        let bytes = entry32_bytes(&[Vec::new(), entry32_section(&["806_131_costume_law_oni"])]);

        let entry32 = Entry32::parse(&bytes).unwrap();
        let item = &entry32.section(1).unwrap().strings[0];

        assert_eq!(item.id, 0);
        assert!(item.relative_offset >= 12);
        assert_eq!(item.byte_len, "806_131_costume_law_oni".len() + 1);
    }

    #[test]
    fn sets_name_and_rebuilds_entry32() {
        let bytes = entry32_bytes(&[
            entry32_section(&["zero"]),
            Vec::new(),
            entry32_section(&["alpha", ""]),
        ]);
        let mut entry32 = Entry32::parse(&bytes).unwrap();

        entry32.set_name(2, 1, "MDLC999_Law_Custom").unwrap();
        let rebuilt = entry32.into_bytes();
        let parsed = Entry32::parse(&rebuilt).unwrap();

        assert_eq!(parsed.name(2, 0), Some("alpha"));
        assert_eq!(parsed.name(2, 1), Some("MDLC999_Law_Custom"));
    }
}
