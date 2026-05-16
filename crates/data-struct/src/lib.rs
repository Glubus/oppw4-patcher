pub mod entry17;
pub mod entry29;
pub mod entry3;
pub mod entry32;
pub mod entry35;
pub mod entry39;
pub mod entry52;
pub mod entry58;
pub mod rdb;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataStructError {
    EntryMissing(usize),
    OutOfBounds {
        what: &'static str,
        offset: usize,
        size: usize,
        len: usize,
    },
    LinkData(String),
}

impl std::fmt::Display for DataStructError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntryMissing(index) => write!(f, "LinkData entry {index} is missing"),
            Self::OutOfBounds {
                what,
                offset,
                size,
                len,
            } => write!(
                f,
                "{what} out of bounds: offset=0x{offset:x} size=0x{size:x} len=0x{len:x}"
            ),
            Self::LinkData(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DataStructError {}

pub type Result<T> = std::result::Result<T, DataStructError>;

pub(crate) fn checked_slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    size: usize,
    what: &'static str,
) -> Result<&'a [u8]> {
    let end = offset
        .checked_add(size)
        .ok_or(DataStructError::OutOfBounds {
            what,
            offset,
            size,
            len: bytes.len(),
        })?;
    bytes.get(offset..end).ok_or(DataStructError::OutOfBounds {
        what,
        offset,
        size,
        len: bytes.len(),
    })
}

pub(crate) fn read_u8(bytes: &[u8], offset: usize, what: &'static str) -> Result<u8> {
    Ok(*checked_slice(bytes, offset, 1, what)?
        .first()
        .expect("checked one byte"))
}

pub(crate) fn read_u16(bytes: &[u8], offset: usize, what: &'static str) -> Result<u16> {
    let slice = checked_slice(bytes, offset, 2, what)?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

pub(crate) fn read_i16(bytes: &[u8], offset: usize, what: &'static str) -> Result<i16> {
    let slice = checked_slice(bytes, offset, 2, what)?;
    Ok(i16::from_le_bytes([slice[0], slice[1]]))
}

pub(crate) fn read_u32(bytes: &[u8], offset: usize, what: &'static str) -> Result<u32> {
    let slice = checked_slice(bytes, offset, 4, what)?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

pub(crate) fn inflate_linkdata_entry(linkdata: &[u8], entry_index: usize) -> Result<Vec<u8>> {
    let index = oppw4_rdb::parse_linkdata(linkdata)
        .map_err(|error| DataStructError::LinkData(format!("{error:?}")))?;
    let entry = index
        .entries
        .get(entry_index)
        .ok_or(DataStructError::EntryMissing(entry_index))?;
    oppw4_rdb::inflate_linkdata_entry(linkdata, entry)
        .map_err(|error| DataStructError::LinkData(format!("{error:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_slice_reports_bounds_with_context() {
        let err = checked_slice(&[1, 2], 1, 4, "probe").unwrap_err();

        assert_eq!(
            err,
            DataStructError::OutOfBounds {
                what: "probe",
                offset: 1,
                size: 4,
                len: 2,
            }
        );
    }

    #[test]
    fn rdb_file_parses_header_blocks_and_address_tail() {
        let bytes = sample_rdb_bytes();
        let rdb = crate::rdb::RdbFile::parse(&bytes).unwrap();

        assert_eq!(rdb.header.first_block_offset, 0x20);
        assert_eq!(rdb.header.declared_count, 1);
        assert_eq!(rdb.header.data_prefix, "data/");
        assert_eq!(rdb.blocks.len(), 1);

        let block = &rdb.blocks[0];
        assert_eq!(block.offset, 0x20);
        assert_eq!(block.length, 0x3f);
        assert_eq!(block.data_offset, 0x0015_b938);
        assert_eq!(block.primary_hash, 0x8df2_d8cb);
        assert_eq!(block.address_tail.as_deref(), Some("386280@c851b#8"));
        assert_eq!(
            block.address_suffix,
            Some(crate::rdb::RdbAddressSuffix {
                part_a: 0x386280,
                part_b: 0xc851b,
                suffix: 8,
            })
        );
    }

    fn sample_rdb_bytes() -> Vec<u8> {
        let mut bytes = vec![0; 0x20];
        bytes[0..8].copy_from_slice(b"_DRK0000");
        bytes[0x08..0x0c].copy_from_slice(&0x20u32.to_le_bytes());
        bytes[0x10..0x14].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x18..0x1d].copy_from_slice(b"data/");

        let payload = b"386280@c851b#8\0";
        let length = 0x30 + payload.len() as u32;
        let mut block = vec![0; length as usize];
        block[0..4].copy_from_slice(b"IDRK");
        block[0x08..0x0c].copy_from_slice(&length.to_le_bytes());
        block[0x10..0x14].copy_from_slice(&0x03u32.to_le_bytes());
        block[0x18..0x1c].copy_from_slice(&0x0015_b938u32.to_le_bytes());
        block[0x20..0x24].copy_from_slice(&0x01u32.to_le_bytes());
        block[0x24..0x28].copy_from_slice(&0x8df2_d8cbu32.to_le_bytes());
        block[0x30..].copy_from_slice(payload);
        bytes.extend_from_slice(&block);
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
        bytes
    }
}
