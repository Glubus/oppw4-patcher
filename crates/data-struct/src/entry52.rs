use crate::{checked_slice, inflate_linkdata_entry, read_u32, Result};

pub const LINKDATA_ENTRY_INDEX: usize = 52;
pub const COSTUME_RECORD_HEADER_SIZE: usize = 0x10;
pub const COSTUME_RECORD_SIZE: usize = 0x20;
pub const COSTUME_RECORD_FIELD_COUNT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeRecordEntry {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeRecord {
    pub row: u32,
    pub offset: usize,
    pub fields: [u32; COSTUME_RECORD_FIELD_COUNT],
    pub raw: Vec<u8>,
}

impl CostumeRecordEntry {
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

    pub fn declared_count(&self) -> Result<u32> {
        read_u32(&self.bytes, 0, "entry52 costume record count")
    }

    pub fn records(&self) -> Result<Vec<CostumeRecord>> {
        let count = self.declared_count()?;
        (0..count).map(|row| self.record(row)).collect()
    }

    pub fn record(&self, row: u32) -> Result<CostumeRecord> {
        let offset = COSTUME_RECORD_HEADER_SIZE + row as usize * COSTUME_RECORD_SIZE;
        let raw = checked_slice(
            &self.bytes,
            offset,
            COSTUME_RECORD_SIZE,
            "entry52 costume record",
        )?
        .to_vec();
        let mut fields = [0u32; COSTUME_RECORD_FIELD_COUNT];
        for (index, field) in fields.iter_mut().enumerate() {
            *field = read_u32(
                &self.bytes,
                offset + index * 4,
                "entry52 costume record field",
            )?;
        }
        Ok(CostumeRecord {
            row,
            offset,
            fields,
            raw,
        })
    }

    pub fn records_for_owner_layout(&self, owner: u32, layout: u32) -> Result<Vec<CostumeRecord>> {
        Ok(self
            .records()?
            .into_iter()
            .filter(|record| record.fields[0] == layout && record.fields[1] == owner)
            .collect())
    }

    pub fn append_record(&mut self, fields: [u32; COSTUME_RECORD_FIELD_COUNT]) -> Result<()> {
        let count = self.declared_count()?;
        self.bytes[0..4].copy_from_slice(&(count + 1).to_le_bytes());
        for value in fields {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_costume_records_after_header() {
        let mut bytes = vec![0; COSTUME_RECORD_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&2u32.to_le_bytes());
        for value in [57u32, 26, 1, 2, 3, 4, 5, 6] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [699u32, 26, 0, 0, 0, 0, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let entry = CostumeRecordEntry::parse(bytes);

        let matches = entry.records_for_owner_layout(26, 699).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].row, 1);
        assert_eq!(matches[0].fields, [699, 26, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn appends_costume_record() {
        let mut bytes = vec![0; COSTUME_RECORD_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&1u32.to_le_bytes());
        for value in [131u32, 26, 0, 0, 0, 0, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let mut entry = CostumeRecordEntry::parse(bytes);

        entry.append_record([699, 26, 0, 0, 0, 0, 0, 0]).unwrap();

        assert_eq!(entry.declared_count().unwrap(), 2);
        assert_eq!(entry.record(1).unwrap().fields, [699, 26, 0, 0, 0, 0, 0, 0]);
    }
}
