use crate::{checked_slice, read_u32, DataStructError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdbFile {
    pub header: RdbHeader,
    pub blocks: Vec<RdbBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdbHeader {
    pub first_block_offset: u32,
    pub declared_count: u32,
    pub data_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdbBlock {
    pub offset: usize,
    pub length: u32,
    pub kind: [u8; 4],
    pub field_10: u32,
    pub data_offset: u32,
    pub field_20: u32,
    pub primary_hash: u32,
    pub field_28: u32,
    pub field_2c: u32,
    pub payload: Vec<u8>,
    pub address_tail: Option<String>,
    pub address_suffix: Option<RdbAddressSuffix>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdbAddressSuffix {
    pub part_a: u64,
    pub part_b: u64,
    pub suffix: u64,
}

impl RdbFile {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        validate_magic(bytes, 0, b"_DRK0000", "rdb root magic")?;
        let header = parse_header(bytes)?;
        let blocks = parse_blocks(bytes, header.first_block_offset as usize)?;
        Ok(Self { header, blocks })
    }

    pub fn find_hash(&self, hash: u32) -> impl Iterator<Item = &RdbBlock> {
        self.blocks
            .iter()
            .filter(move |block| block.primary_hash == hash)
    }
}

impl RdbBlock {
    pub fn address_len(&self) -> usize {
        self.address_tail.as_ref().map_or(0, |tail| tail.len() + 1)
    }
}

fn parse_header(bytes: &[u8]) -> Result<RdbHeader> {
    Ok(RdbHeader {
        first_block_offset: read_u32(bytes, 0x08, "rdb first block offset")?,
        declared_count: read_u32(bytes, 0x10, "rdb declared count")?,
        data_prefix: read_c_string(checked_slice(bytes, 0x18, 8, "rdb data prefix")?),
    })
}

fn parse_blocks(bytes: &[u8], first_offset: usize) -> Result<Vec<RdbBlock>> {
    let mut blocks = Vec::new();
    let mut offset = first_offset;
    while offset < bytes.len() {
        let block = parse_block(bytes, offset)?;
        offset = align4(offset + block.length as usize);
        blocks.push(block);
    }
    Ok(blocks)
}

fn parse_block(bytes: &[u8], offset: usize) -> Result<RdbBlock> {
    validate_magic(bytes, offset, b"IDRK", "rdb block magic")?;
    let length = read_u32(bytes, offset + 0x08, "rdb block length")?;
    if length < 0x30 {
        return Err(DataStructError::OutOfBounds {
            what: "rdb block length",
            offset,
            size: length as usize,
            len: 0x30,
        });
    }

    let raw = checked_slice(bytes, offset, length as usize, "rdb block")?;
    let payload = raw[0x30..].to_vec();
    let address_tail = payload_tail_string(&payload);
    let address_suffix = address_tail.as_deref().and_then(parse_address_suffix);

    Ok(RdbBlock {
        offset,
        length,
        kind: raw[0..4].try_into().expect("checked magic bytes"),
        field_10: read_u32(bytes, offset + 0x10, "rdb block field_10")?,
        data_offset: read_u32(bytes, offset + 0x18, "rdb block data offset")?,
        field_20: read_u32(bytes, offset + 0x20, "rdb block field_20")?,
        primary_hash: read_u32(bytes, offset + 0x24, "rdb block primary hash")?,
        field_28: read_u32(bytes, offset + 0x28, "rdb block field_28")?,
        field_2c: read_u32(bytes, offset + 0x2c, "rdb block field_2c")?,
        payload,
        address_tail,
        address_suffix,
    })
}

fn validate_magic(bytes: &[u8], offset: usize, magic: &[u8], what: &'static str) -> Result<()> {
    let actual = checked_slice(bytes, offset, magic.len(), what)?;
    if actual != magic {
        return Err(DataStructError::LinkData(format!(
            "{what}: expected {:?}, got {:?}",
            String::from_utf8_lossy(magic),
            actual
        )));
    }
    Ok(())
}

fn payload_tail_string(payload: &[u8]) -> Option<String> {
    let nul = payload.iter().rposition(|byte| *byte == 0)?;
    let start = payload[..nul]
        .iter()
        .rposition(|byte| *byte == 0)
        .map_or(0, |index| index + 1);
    let tail = &payload[start..nul];
    if tail.is_empty() || !tail.iter().all(|byte| byte.is_ascii_graphic()) {
        return None;
    }
    std::str::from_utf8(tail).ok().map(ToOwned::to_owned)
}

fn parse_address_suffix(tail: &str) -> Option<RdbAddressSuffix> {
    let (part_a, rest) = tail.split_once('@')?;
    let (part_b, suffix) = rest.split_once('#')?;
    Some(RdbAddressSuffix {
        part_a: u64::from_str_radix(part_a, 16).ok()?,
        part_b: u64::from_str_radix(part_b, 16).ok()?,
        suffix: u64::from_str_radix(suffix, 16).ok()?,
    })
}

fn read_c_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn align4(value: usize) -> usize {
    (value + 3) & !3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_hash_returns_matching_blocks() {
        let mut bytes = vec![0; 0x20];
        bytes[0..8].copy_from_slice(b"_DRK0000");
        bytes[0x08..0x0c].copy_from_slice(&0x20u32.to_le_bytes());
        bytes[0x10..0x14].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x18..0x1d].copy_from_slice(b"data/");
        let mut block = vec![0; 0x30];
        block[0..4].copy_from_slice(b"IDRK");
        block[0x08..0x0c].copy_from_slice(&0x30u32.to_le_bytes());
        block[0x24..0x28].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        bytes.extend(block);

        let rdb = RdbFile::parse(&bytes).unwrap();

        assert_eq!(rdb.find_hash(0x1234_5678).count(), 1);
        assert_eq!(rdb.find_hash(0x8765_4321).count(), 0);
    }
}
