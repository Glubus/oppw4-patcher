use crate::{checked_slice, inflate_linkdata_entry, read_i16, read_u32, DataStructError, Result};

pub const LINKDATA_ENTRY_INDEX: usize = 35;
pub const MODEL_ROW_BASE: usize = 0x10;
pub const MODEL_ROW_SIZE: usize = 0x60;
pub const MODEL_ROW_OWNER_I16_INDEX: usize = 28;
pub const MODEL_ROW_RELATION_I16_INDEX: usize = 34;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEntry {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRow {
    pub row: u16,
    pub offset: usize,
    pub owner: i16,
    pub relation: i16,
    pub raw: Vec<u8>,
}

impl ModelEntry {
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
        read_u32(&self.bytes, 0, "entry35 model count")
    }

    pub fn row(&self, row: u16) -> Result<ModelRow> {
        let offset = MODEL_ROW_BASE + usize::from(row) * MODEL_ROW_SIZE;
        let raw = checked_slice(&self.bytes, offset, MODEL_ROW_SIZE, "entry35 model row")?.to_vec();
        let owner = read_i16(
            &self.bytes,
            offset + MODEL_ROW_OWNER_I16_INDEX * 2,
            "entry35 model owner",
        )?;
        let relation = read_i16(
            &self.bytes,
            offset + MODEL_ROW_RELATION_I16_INDEX * 2,
            "entry35 model relation",
        )?;
        Ok(ModelRow {
            row,
            offset,
            owner,
            relation,
            raw,
        })
    }

    pub fn rows(&self) -> Result<Vec<ModelRow>> {
        let count = self.declared_count()? as u16;
        (0..count).map(|row| self.row(row)).collect()
    }

    pub fn rows_for_owner(&self, owner: i16) -> Result<Vec<ModelRow>> {
        Ok(self
            .rows()?
            .into_iter()
            .filter(|row| row.owner == owner)
            .collect())
    }

    pub fn clone_row(&mut self, source_row: u16, target_row: u16) -> Result<()> {
        let source_offset = MODEL_ROW_BASE + usize::from(source_row) * MODEL_ROW_SIZE;
        let target_offset = MODEL_ROW_BASE + usize::from(target_row) * MODEL_ROW_SIZE;
        let source = checked_slice(
            &self.bytes,
            source_offset,
            MODEL_ROW_SIZE,
            "entry35 source model row",
        )?
        .to_vec();
        let target_end =
            target_offset
                .checked_add(MODEL_ROW_SIZE)
                .ok_or(DataStructError::OutOfBounds {
                    what: "entry35 target model row",
                    offset: target_offset,
                    size: MODEL_ROW_SIZE,
                    len: self.bytes.len(),
                })?;
        if target_end > self.bytes.len() {
            return Err(DataStructError::OutOfBounds {
                what: "entry35 target model row",
                offset: target_offset,
                size: MODEL_ROW_SIZE,
                len: self.bytes.len(),
            });
        }
        self.bytes[target_offset..target_end].copy_from_slice(&source);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_i16(bytes: &mut [u8], offset: usize, value: i16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn parses_model_row_owner_and_relation() {
        let mut bytes = vec![0xff; MODEL_ROW_BASE + MODEL_ROW_SIZE * 2];
        bytes[0..4].copy_from_slice(&2u32.to_le_bytes());
        let row1 = MODEL_ROW_BASE + MODEL_ROW_SIZE;
        put_i16(&mut bytes, row1 + MODEL_ROW_OWNER_I16_INDEX * 2, 26);
        put_i16(&mut bytes, row1 + MODEL_ROW_RELATION_I16_INDEX * 2, 26);
        let entry = ModelEntry::parse(bytes);

        let row = entry.row(1).unwrap();

        assert_eq!(row.offset, row1);
        assert_eq!(row.owner, 26);
        assert_eq!(row.relation, 26);
        assert_eq!(entry.rows_for_owner(26).unwrap(), vec![row]);
    }

    #[test]
    fn clones_model_row_to_existing_target_row() {
        let mut bytes = vec![0xff; MODEL_ROW_BASE + MODEL_ROW_SIZE * 4];
        bytes[0..4].copy_from_slice(&4u32.to_le_bytes());
        let source = MODEL_ROW_BASE + MODEL_ROW_SIZE;
        let target = MODEL_ROW_BASE + MODEL_ROW_SIZE * 3;
        bytes[source..source + MODEL_ROW_SIZE].fill(0x42);
        put_i16(&mut bytes, source + MODEL_ROW_OWNER_I16_INDEX * 2, 26);
        put_i16(&mut bytes, source + MODEL_ROW_RELATION_I16_INDEX * 2, 26);
        put_i16(&mut bytes, target + MODEL_ROW_OWNER_I16_INDEX * 2, -1);
        let mut entry = ModelEntry::parse(bytes);

        entry.clone_row(1, 3).unwrap();

        let cloned = entry.row(3).unwrap();
        assert_eq!(cloned.owner, 26);
        assert_eq!(cloned.relation, 26);
        assert_eq!(cloned.raw, entry.row(1).unwrap().raw);
    }
}
