use crate::{checked_slice, inflate_linkdata_entry, read_u16, read_u32, read_u8, Result};

pub const LINKDATA_ENTRY_INDEX: usize = 3;
pub const LAYOUT_ROW_SIZE: usize = 0x4b;
pub const LAYOUT_TARGET_OFFSET: usize = 0x14;
pub const LAYOUT_PREVIEW_OFFSET: usize = 0x16;
pub const LAYOUT_VARIANTS_OFFSET: usize = 0x28;
pub const LAYOUT_VARIANT_COUNT: usize = 16;
pub const LAYOUT_ACTIVE_COUNT_OFFSET: usize = 0x48;
pub const LAYOUT_FLAGS_4A_OFFSET: usize = 0x4a;
pub const VARIANT_METADATA_BASE_OFFSET: usize = 0xd92c;
pub const VARIANT_METADATA_STRIDE: usize = 0x1e;
pub const VARIANT_METADATA_SIZE: usize = 0x1e;
pub const VARIANT_METADATA_MODEL_OFFSET: usize = 0x00;
pub const VARIANT_METADATA_PREVIEW_OFFSET: usize = 0x06;
pub const VARIANT_METADATA_COLOR_OFFSET: usize = 0x0c;
pub const VARIANT_METADATA_COLOR_SIZE: usize = 0x08;
pub const VARIANT_METADATA_FLAGS_OFFSET: usize = 0x14;
pub const VARIANT_METADATA_LOAD_ARG2_OFFSET: usize = 0x1a;
pub const EMPTY_LAYOUT_VARIANT_ID: u16 = u16::MAX;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeStaticEntry {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeLayoutRow {
    pub offset: usize,
    pub target: u16,
    pub preview: u16,
    pub variants: [u16; LAYOUT_VARIANT_COUNT],
    pub active_count: u8,
    pub flags_4a: u8,
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostumeVariantMetadata {
    pub variant_id: u16,
    pub offset: usize,
    pub model_resource: u16,
    pub preview_mapping: u16,
    pub color_bytes: [u8; VARIANT_METADATA_COLOR_SIZE],
    pub flags: u8,
    pub raw: Vec<u8>,
}

impl CostumeStaticEntry {
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

    pub fn layout_row_at(&self, offset: usize) -> Result<CostumeLayoutRow> {
        parse_layout_row(&self.bytes, offset)
    }

    pub fn scan_layout_rows(&self) -> Result<Vec<CostumeLayoutRow>> {
        if self.bytes.len() < LAYOUT_ROW_SIZE {
            return Ok(Vec::new());
        }

        let mut rows = Vec::new();
        for offset in (0..=self.bytes.len() - LAYOUT_ROW_SIZE).step_by(2) {
            let row = parse_layout_row(&self.bytes, offset)?;
            if looks_like_layout_row(&row) {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    pub fn find_layout_rows(&self, target: u16, preview: u16) -> Result<Vec<CostumeLayoutRow>> {
        Ok(self
            .scan_layout_rows()?
            .into_iter()
            .filter(|row| row.target == target && row.preview == preview)
            .collect())
    }

    pub fn variant_metadata(&self, variant_id: u16) -> Result<CostumeVariantMetadata> {
        let offset =
            VARIANT_METADATA_BASE_OFFSET + usize::from(variant_id) * VARIANT_METADATA_STRIDE;
        parse_variant_metadata(&self.bytes, variant_id, offset)
    }

    pub fn set_layout_slot_variant(
        &mut self,
        row: &CostumeLayoutRow,
        slot_index: usize,
        variant: u16,
    ) -> Result<()> {
        if slot_index >= LAYOUT_VARIANT_COUNT {
            return Err(crate::DataStructError::OutOfBounds {
                what: "entry3 layout slot index",
                offset: slot_index,
                size: 1,
                len: LAYOUT_VARIANT_COUNT,
            });
        }
        let offset = row.offset + LAYOUT_VARIANTS_OFFSET + slot_index * 2;
        write_u16(&mut self.bytes, offset, variant, "layout variant write")
    }

    pub fn set_layout_active_count(
        &mut self,
        row: &CostumeLayoutRow,
        active_count: u8,
    ) -> Result<()> {
        let offset = row.offset + LAYOUT_ACTIVE_COUNT_OFFSET;
        write_u8(
            &mut self.bytes,
            offset,
            active_count,
            "layout active count write",
        )
    }

    pub fn clone_variant_metadata(
        &mut self,
        source_variant_id: u16,
        target_variant_id: u16,
    ) -> Result<()> {
        let source = self.variant_metadata(source_variant_id)?;
        let target = self.variant_metadata(target_variant_id)?;
        let target_slice = checked_slice_mut(
            &mut self.bytes,
            target.offset,
            VARIANT_METADATA_SIZE,
            "entry3 target variant metadata",
        )?;
        target_slice.copy_from_slice(&source.raw);
        Ok(())
    }

    pub fn set_variant_model_resource(
        &mut self,
        variant_id: u16,
        model_resource: u16,
    ) -> Result<()> {
        let metadata = self.variant_metadata(variant_id)?;
        write_u16(
            &mut self.bytes,
            metadata.offset + VARIANT_METADATA_MODEL_OFFSET,
            model_resource,
            "variant metadata model resource write",
        )
    }

    pub fn set_variant_preview_mapping(
        &mut self,
        variant_id: u16,
        preview_mapping: u16,
    ) -> Result<()> {
        let metadata = self.variant_metadata(variant_id)?;
        write_u16(
            &mut self.bytes,
            metadata.offset + VARIANT_METADATA_PREVIEW_OFFSET,
            preview_mapping,
            "variant metadata preview mapping write",
        )
    }

    pub fn set_variant_flags(&mut self, variant_id: u16, flags: u8) -> Result<()> {
        let metadata = self.variant_metadata(variant_id)?;
        write_u8(
            &mut self.bytes,
            metadata.offset + VARIANT_METADATA_FLAGS_OFFSET,
            flags,
            "variant metadata flags write",
        )
    }

    pub fn set_variant_load_arg1(&mut self, variant_id: u16, value: u16) -> Result<()> {
        let metadata = self.variant_metadata(variant_id)?;
        write_u16(
            &mut self.bytes,
            metadata.offset + VARIANT_METADATA_COLOR_OFFSET,
            value,
            "variant metadata load arg1 write",
        )
    }

    pub fn set_variant_load_arg2(&mut self, variant_id: u16, value: u8) -> Result<()> {
        let metadata = self.variant_metadata(variant_id)?;
        write_u8(
            &mut self.bytes,
            metadata.offset + VARIANT_METADATA_LOAD_ARG2_OFFSET,
            value,
            "variant metadata load arg2 write",
        )
    }
}

fn checked_slice_mut<'a>(
    bytes: &'a mut [u8],
    offset: usize,
    size: usize,
    what: &'static str,
) -> Result<&'a mut [u8]> {
    let len = bytes.len();
    let end = offset
        .checked_add(size)
        .ok_or(crate::DataStructError::OutOfBounds {
            what,
            offset,
            size,
            len,
        })?;
    bytes
        .get_mut(offset..end)
        .ok_or(crate::DataStructError::OutOfBounds {
            what,
            offset,
            size,
            len,
        })
}

fn write_u8(bytes: &mut [u8], offset: usize, value: u8, what: &'static str) -> Result<()> {
    let target = checked_slice_mut(bytes, offset, 1, what)?;
    target[0] = value;
    Ok(())
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16, what: &'static str) -> Result<()> {
    let target = checked_slice_mut(bytes, offset, 2, what)?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn parse_layout_row(bytes: &[u8], offset: usize) -> Result<CostumeLayoutRow> {
    let raw = checked_slice(bytes, offset, LAYOUT_ROW_SIZE, "entry3 layout row")?.to_vec();
    let target = read_u16(bytes, offset + LAYOUT_TARGET_OFFSET, "layout target")?;
    let preview = read_u16(bytes, offset + LAYOUT_PREVIEW_OFFSET, "layout preview")?;
    let mut variants = [0u16; LAYOUT_VARIANT_COUNT];
    for (index, variant) in variants.iter_mut().enumerate() {
        *variant = read_u16(
            bytes,
            offset + LAYOUT_VARIANTS_OFFSET + index * 2,
            "layout variant",
        )?;
    }
    let active_count = read_u8(
        bytes,
        offset + LAYOUT_ACTIVE_COUNT_OFFSET,
        "layout active count",
    )?;
    let flags_4a = read_u8(bytes, offset + LAYOUT_FLAGS_4A_OFFSET, "layout flags 4a")?;

    Ok(CostumeLayoutRow {
        offset,
        target,
        preview,
        variants,
        active_count,
        flags_4a,
        raw,
    })
}

fn parse_variant_metadata(
    bytes: &[u8],
    variant_id: u16,
    offset: usize,
) -> Result<CostumeVariantMetadata> {
    let raw = checked_slice(
        bytes,
        offset,
        VARIANT_METADATA_SIZE,
        "entry3 variant metadata",
    )?
    .to_vec();
    let model_resource = read_u16(
        bytes,
        offset + VARIANT_METADATA_MODEL_OFFSET,
        "variant metadata model resource",
    )?;
    let preview_mapping = read_u16(
        bytes,
        offset + VARIANT_METADATA_PREVIEW_OFFSET,
        "variant metadata preview mapping",
    )?;
    let mut color_bytes = [0u8; VARIANT_METADATA_COLOR_SIZE];
    color_bytes.copy_from_slice(checked_slice(
        bytes,
        offset + VARIANT_METADATA_COLOR_OFFSET,
        VARIANT_METADATA_COLOR_SIZE,
        "variant metadata color bytes",
    )?);
    let flags = read_u8(
        bytes,
        offset + VARIANT_METADATA_FLAGS_OFFSET,
        "variant metadata flags",
    )?;

    Ok(CostumeVariantMetadata {
        variant_id,
        offset,
        model_resource,
        preview_mapping,
        color_bytes,
        flags,
        raw,
    })
}

fn looks_like_layout_row(row: &CostumeLayoutRow) -> bool {
    let active_count = usize::from(row.active_count);
    row.target != u16::MAX
        && row.preview != u16::MAX
        && active_count <= LAYOUT_VARIANT_COUNT
        && row
            .variants
            .iter()
            .take(active_count)
            .all(|variant| *variant != u16::MAX)
        && row
            .variants
            .iter()
            .skip(active_count)
            .all(|variant| *variant == u16::MAX)
}

#[allow(dead_code)]
fn _read_reserved_u32_for_future_fields(bytes: &[u8], offset: usize) -> Result<u32> {
    read_u32(bytes, offset, "reserved entry3 u32")
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAW_ROW_OFFSET: usize = 0x6d8;

    fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_law_row(bytes: &mut [u8], slot4: u16, active_count: u8) {
        put_u16(bytes, LAW_ROW_OFFSET + LAYOUT_TARGET_OFFSET, 26);
        put_u16(bytes, LAW_ROW_OFFSET + LAYOUT_PREVIEW_OFFSET, 26);
        for (index, value) in [57u16, 58, 555, 586].into_iter().enumerate() {
            put_u16(
                bytes,
                LAW_ROW_OFFSET + LAYOUT_VARIANTS_OFFSET + index * 2,
                value,
            );
        }
        put_u16(
            bytes,
            LAW_ROW_OFFSET + LAYOUT_VARIANTS_OFFSET + 4 * 2,
            slot4,
        );
        for index in 5..LAYOUT_VARIANT_COUNT {
            put_u16(
                bytes,
                LAW_ROW_OFFSET + LAYOUT_VARIANTS_OFFSET + index * 2,
                u16::MAX,
            );
        }
        bytes[LAW_ROW_OFFSET + LAYOUT_ACTIVE_COUNT_OFFSET] = active_count;
        bytes[LAW_ROW_OFFSET + LAYOUT_FLAGS_4A_OFFSET] = 1;
    }

    fn put_variant_metadata(
        bytes: &mut [u8],
        variant_id: u16,
        model: u16,
        preview: u16,
        flags: u8,
    ) {
        let offset =
            VARIANT_METADATA_BASE_OFFSET + usize::from(variant_id) * VARIANT_METADATA_STRIDE;
        put_u16(bytes, offset + VARIANT_METADATA_MODEL_OFFSET, model);
        put_u16(bytes, offset + VARIANT_METADATA_PREVIEW_OFFSET, preview);
        bytes[offset + VARIANT_METADATA_COLOR_OFFSET..offset + VARIANT_METADATA_COLOR_OFFSET + 8]
            .copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        bytes[offset + VARIANT_METADATA_FLAGS_OFFSET] = flags;
    }

    #[test]
    fn parses_law_layout_row_fields() {
        let mut bytes = vec![0xff; 0x800];
        put_law_row(&mut bytes, 699, 5);
        let entry = CostumeStaticEntry::parse(bytes);

        let row = entry.layout_row_at(LAW_ROW_OFFSET).unwrap();

        assert_eq!(row.offset, LAW_ROW_OFFSET);
        assert_eq!(row.target, 26);
        assert_eq!(row.preview, 26);
        assert_eq!(&row.variants[..5], &[57, 58, 555, 586, 699]);
        assert_eq!(row.active_count, 5);
        assert_eq!(row.flags_4a, 1);
        assert_eq!(row.raw.len(), LAYOUT_ROW_SIZE);
    }

    #[test]
    fn scans_layout_rows_by_target_and_preview() {
        let mut bytes = vec![0xff; 0x800];
        put_law_row(&mut bytes, u16::MAX, 4);
        let entry = CostumeStaticEntry::parse(bytes);

        let rows = entry.find_layout_rows(26, 26).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].offset, LAW_ROW_OFFSET);
        assert_eq!(&rows[0].variants[..4], &[57, 58, 555, 586]);
    }

    #[test]
    fn scan_rejects_shifted_false_rows_inside_real_layout() {
        let mut bytes = vec![0xff; 0x800];
        put_law_row(&mut bytes, 699, 5);
        let entry = CostumeStaticEntry::parse(bytes);

        let rows = entry.find_layout_rows(26, 26).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].offset, LAW_ROW_OFFSET);
    }

    #[test]
    fn parses_variant_metadata_fields() {
        let mut bytes = vec![0; VARIANT_METADATA_BASE_OFFSET + 700 * VARIANT_METADATA_STRIDE];
        put_variant_metadata(&mut bytes, 699, 292, 294, 0x05);
        let entry = CostumeStaticEntry::parse(bytes);

        let metadata = entry.variant_metadata(699).unwrap();

        assert_eq!(metadata.variant_id, 699);
        assert_eq!(metadata.model_resource, 292);
        assert_eq!(metadata.preview_mapping, 294);
        assert_eq!(metadata.color_bytes, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(metadata.flags, 0x05);
        assert_eq!(metadata.raw.len(), VARIANT_METADATA_SIZE);
    }
}
