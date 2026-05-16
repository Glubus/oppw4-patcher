use crate::{checked_slice, inflate_linkdata_entry, read_u32, Result};

pub const LINKDATA_ENTRY_INDEX: usize = 58;
pub const SECTION_HEADER_SIZE: usize = 0x10;
pub const SECTION_RECORD_SIZE: usize = 0x10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeSectionEntry {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeSection {
    pub suffix: u16,
    pub offset: usize,
    pub record_count: u32,
    pub header: [u32; 4],
    pub records: Vec<CostumeSectionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeSectionRecord {
    pub index: u32,
    pub offset: usize,
    pub fields: [u32; 4],
    pub raw: Vec<u8>,
}

impl CostumeSectionEntry {
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

    pub fn section_count(&self) -> Result<u32> {
        read_u32(&self.bytes, 0, "entry58 section count")
    }

    pub fn section_offset(&self, suffix: u16) -> Result<Option<usize>> {
        if usize::from(suffix) >= self.section_count()? as usize {
            return Ok(None);
        }
        let offset = read_u32(
            &self.bytes,
            4 + usize::from(suffix) * 4,
            "entry58 section offset",
        )? as usize;
        Ok((offset != 0).then_some(offset))
    }

    pub fn section(&self, suffix: u16) -> Result<Option<CostumeSection>> {
        let Some(offset) = self.section_offset(suffix)? else {
            return Ok(None);
        };
        checked_slice(
            &self.bytes,
            offset,
            SECTION_HEADER_SIZE,
            "entry58 section header",
        )?;
        let mut header = [0u32; 4];
        for (index, field) in header.iter_mut().enumerate() {
            *field = read_u32(
                &self.bytes,
                offset + index * 4,
                "entry58 section header field",
            )?;
        }
        let record_count = header[0];
        let mut records = Vec::new();
        let records_start = offset + SECTION_HEADER_SIZE;
        for index in 0..record_count {
            let record_offset = records_start + index as usize * SECTION_RECORD_SIZE;
            let raw = checked_slice(
                &self.bytes,
                record_offset,
                SECTION_RECORD_SIZE,
                "entry58 section record",
            )?
            .to_vec();
            let mut fields = [0u32; 4];
            for (field_index, field) in fields.iter_mut().enumerate() {
                *field = read_u32(
                    &self.bytes,
                    record_offset + field_index * 4,
                    "entry58 section record field",
                )?;
            }
            records.push(CostumeSectionRecord {
                index,
                offset: record_offset,
                fields,
                raw,
            });
        }
        Ok(Some(CostumeSection {
            suffix,
            offset,
            record_count,
            header,
            records,
        }))
    }

    pub fn clone_section(&mut self, source_suffix: u16, target_suffix: u16) -> Result<()> {
        let Some(source_offset) = self.section_offset(source_suffix)? else {
            return Ok(());
        };
        if self.section_offset(target_suffix)?.is_some() {
            return Ok(());
        }

        let source = self.section(source_suffix)?.expect("source offset exists");
        let source_size = SECTION_HEADER_SIZE + source.record_count as usize * SECTION_RECORD_SIZE;
        let section_bytes = checked_slice(
            &self.bytes,
            source_offset,
            source_size,
            "entry58 source section",
        )?
        .to_vec();

        self.ensure_section_slot(target_suffix)?;

        let target_offset = self.bytes.len();
        self.bytes.extend_from_slice(&section_bytes);
        let offset_table = 4 + usize::from(target_suffix) * 4;
        self.bytes[offset_table..offset_table + 4]
            .copy_from_slice(&(target_offset as u32).to_le_bytes());
        Ok(())
    }

    fn ensure_section_slot(&mut self, suffix: u16) -> Result<()> {
        let old_count = self.section_count()? as usize;
        let needed_count = usize::from(suffix) + 1;
        if needed_count <= old_count {
            let offset_table = 4 + usize::from(suffix) * 4;
            checked_slice(
                &self.bytes,
                offset_table,
                4,
                "entry58 target section offset slot",
            )?;
            return Ok(());
        }

        let old_table_end = 4 + old_count * 4;
        checked_slice(&self.bytes, 0, old_table_end, "entry58 offset table")?;
        let growth = (needed_count - old_count) * 4;
        let mut bytes = Vec::with_capacity(self.bytes.len() + growth);
        bytes.extend_from_slice(&(needed_count as u32).to_le_bytes());

        for index in 0..old_count {
            let raw_offset = read_u32(&self.bytes, 4 + index * 4, "entry58 section offset")?;
            let new_offset = if raw_offset == 0 {
                0
            } else {
                raw_offset + growth as u32
            };
            bytes.extend_from_slice(&new_offset.to_le_bytes());
        }
        bytes.resize(4 + needed_count * 4, 0);
        bytes.extend_from_slice(&self.bytes[old_table_end..]);
        self.bytes = bytes;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_offset_section_records() {
        let mut bytes = vec![0; 4 + 3 * 4];
        bytes[0..4].copy_from_slice(&3u32.to_le_bytes());
        let section_offset = bytes.len();
        bytes[4 + 2 * 4..4 + 3 * 4].copy_from_slice(&(section_offset as u32).to_le_bytes());
        for value in [1u32, 2, 3, 4] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [26u32, 699, 0, 1] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let entry = CostumeSectionEntry::parse(bytes);

        let section = entry.section(2).unwrap().unwrap();

        assert_eq!(section.suffix, 2);
        assert_eq!(section.record_count, 1);
        assert_eq!(section.header, [1, 2, 3, 4]);
        assert_eq!(section.records[0].fields, [26, 699, 0, 1]);
    }

    #[test]
    fn clones_section_to_empty_target_suffix() {
        let mut bytes = vec![0; 4 + 5 * 4];
        bytes[0..4].copy_from_slice(&5u32.to_le_bytes());
        let source_offset = bytes.len();
        bytes[4 + 2 * 4..4 + 3 * 4].copy_from_slice(&(source_offset as u32).to_le_bytes());
        for value in [1u32, 0, 0, 0, 8, 7, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let mut entry = CostumeSectionEntry::parse(bytes);

        entry.clone_section(2, 4).unwrap();

        let target = entry.section(4).unwrap().unwrap();
        assert_eq!(target.header, [1, 0, 0, 0]);
        assert_eq!(target.records[0].fields, [8, 7, 0, 0]);
    }

    #[test]
    fn clones_section_to_suffix_beyond_current_table() {
        let mut bytes = vec![0; 4 + 3 * 4];
        bytes[0..4].copy_from_slice(&3u32.to_le_bytes());
        let source_offset = bytes.len();
        bytes[4 + 2 * 4..4 + 3 * 4].copy_from_slice(&(source_offset as u32).to_le_bytes());
        for value in [1u32, 0, 0, 0, 8, 7, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let mut entry = CostumeSectionEntry::parse(bytes);

        entry.clone_section(2, 699).unwrap();

        assert_eq!(entry.section_count().unwrap(), 700);
        let source = entry.section(2).unwrap().unwrap();
        let target = entry.section(699).unwrap().unwrap();
        assert_eq!(source.records[0].fields, [8, 7, 0, 0]);
        assert_eq!(target.records[0].fields, [8, 7, 0, 0]);
    }
}
