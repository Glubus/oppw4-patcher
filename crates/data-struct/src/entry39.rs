use crate::{checked_slice, inflate_linkdata_entry, read_i16, read_u32, Result};

pub const LINKDATA_ENTRY_INDEX: usize = 39;
pub const COSTUME_PARAM_ROW_BASE: usize = 0x10;
pub const COSTUME_PARAM_ROW_SIZE: usize = 0x30;
pub const COSTUME_PARAM_FIELD_COUNT: usize = COSTUME_PARAM_ROW_SIZE / 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeParamEntry {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeParamRow {
    pub row: u16,
    pub offset: usize,
    pub fields: [i16; COSTUME_PARAM_FIELD_COUNT],
    pub present: bool,
    pub raw: Vec<u8>,
}

impl CostumeParamEntry {
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
        read_u32(&self.bytes, 0, "entry39 costume param count")
    }

    pub fn row(&self, row: u16) -> Result<CostumeParamRow> {
        let offset = COSTUME_PARAM_ROW_BASE + usize::from(row) * COSTUME_PARAM_ROW_SIZE;
        let raw = checked_slice(
            &self.bytes,
            offset,
            COSTUME_PARAM_ROW_SIZE,
            "entry39 costume param row",
        )?
        .to_vec();
        let mut fields = [0i16; COSTUME_PARAM_FIELD_COUNT];
        for (index, field) in fields.iter_mut().enumerate() {
            *field = read_i16(&self.bytes, offset + index * 2, "entry39 costume param")?;
        }
        let present = fields.iter().any(|value| *value != 0 && *value != -1);
        Ok(CostumeParamRow {
            row,
            offset,
            fields,
            present,
            raw,
        })
    }

    pub fn clone_row(&mut self, source_row: u16, target_row: u16) -> Result<()> {
        let source = self.row(source_row)?;
        if self.row(target_row).map(|row| row.present).unwrap_or(false) {
            return Ok(());
        }

        let required_len =
            COSTUME_PARAM_ROW_BASE + (usize::from(target_row) + 1) * COSTUME_PARAM_ROW_SIZE;
        if self.bytes.len() < required_len {
            self.bytes.resize(required_len, 0xff);
        }

        let target_count = u32::from(target_row) + 1;
        if self.declared_count()? < target_count {
            self.bytes[0..4].copy_from_slice(&target_count.to_le_bytes());
        }

        let target_offset =
            COSTUME_PARAM_ROW_BASE + usize::from(target_row) * COSTUME_PARAM_ROW_SIZE;
        self.bytes[target_offset..target_offset + COSTUME_PARAM_ROW_SIZE]
            .copy_from_slice(&source.raw);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_costume_param_row_fields() {
        let mut bytes = vec![0xff; COSTUME_PARAM_ROW_BASE + COSTUME_PARAM_ROW_SIZE * 112];
        bytes[0..4].copy_from_slice(&112u32.to_le_bytes());
        let offset = COSTUME_PARAM_ROW_BASE + 111 * COSTUME_PARAM_ROW_SIZE;
        bytes[offset..offset + 2].copy_from_slice(&26i16.to_le_bytes());
        bytes[offset + 2..offset + 4].copy_from_slice(&111i16.to_le_bytes());
        let entry = CostumeParamEntry::parse(bytes);

        let row = entry.row(111).unwrap();

        assert!(row.present);
        assert_eq!(row.fields[0], 26);
        assert_eq!(row.fields[1], 111);
        assert_eq!(row.offset, offset);
    }

    #[test]
    fn clones_costume_param_row_to_empty_target() {
        let mut bytes = vec![0xff; COSTUME_PARAM_ROW_BASE + COSTUME_PARAM_ROW_SIZE * 132];
        bytes[0..4].copy_from_slice(&132u32.to_le_bytes());
        let offset = COSTUME_PARAM_ROW_BASE + 131 * COSTUME_PARAM_ROW_SIZE;
        bytes[offset..offset + 2].copy_from_slice(&(-1i16).to_le_bytes());
        bytes[offset + 2..offset + 4].copy_from_slice(&(-1i16).to_le_bytes());
        bytes[offset + 4..offset + 6].copy_from_slice(&23378i16.to_le_bytes());
        bytes[offset + 6..offset + 8].copy_from_slice(&0i16.to_le_bytes());
        let mut entry = CostumeParamEntry::parse(bytes);

        entry.clone_row(131, 699).unwrap();

        let target = entry.row(699).unwrap();
        assert!(target.present);
        assert_eq!(target.fields[2], 23378);
        assert_eq!(entry.declared_count().unwrap(), 700);
    }
}
