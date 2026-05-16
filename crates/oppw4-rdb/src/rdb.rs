use crate::bytes::{align4, read_c_string, read_u32};

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
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdbIndex {
    pub header: RdbHeader,
    pub blocks: Vec<RdbBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdbError {
    TooSmall,
    InvalidRootMagic,
    InvalidBlockMagic { offset: usize },
    InvalidBlockLength { offset: usize, length: u32 },
    TruncatedBlock { offset: usize, length: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdbAppendSpec {
    pub template_hash: u32,
    pub private_hash: u32,
    pub virtual_bin_suffix: String,
    pub payload_size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdbAppendError {
    Parse(RdbError),
    MissingTemplateHash(u32),
    TemplateHasNoAddressTail(u32),
    SizeTooLarge(u64),
    BlockTooLarge(usize),
    CountOverflow(u32),
}

pub fn parse_rdb(bytes: &[u8]) -> Result<RdbIndex, RdbError> {
    validate_root(bytes)?;
    let header = parse_header(bytes)?;
    let blocks = parse_blocks(bytes, header.first_block_offset as usize)?;

    Ok(RdbIndex { header, blocks })
}

pub fn append_virtual_rdb_entry(
    bytes: &[u8],
    spec: RdbAppendSpec,
) -> Result<Vec<u8>, RdbAppendError> {
    let index = parse_rdb(bytes).map_err(RdbAppendError::Parse)?;
    let template = index
        .blocks
        .iter()
        .find(|block| block.primary_hash == spec.template_hash)
        .ok_or(RdbAppendError::MissingTemplateHash(spec.template_hash))?;
    let new_block = build_virtual_block(template, &spec)?;
    let new_count = index
        .header
        .declared_count
        .checked_add(1)
        .ok_or(RdbAppendError::CountOverflow(index.header.declared_count))?;

    let mut blocks = index
        .blocks
        .iter()
        .map(|block| block.raw.clone())
        .collect::<Vec<_>>();
    let insert_at = index
        .blocks
        .iter()
        .position(|block| block.primary_hash > spec.private_hash)
        .unwrap_or(blocks.len());
    blocks.insert(insert_at, new_block);

    let mut patched = bytes[..index.header.first_block_offset as usize].to_vec();
    patched[0x10..0x14].copy_from_slice(&new_count.to_le_bytes());
    for block in blocks {
        patched.extend_from_slice(&block);
        while patched.len() % 4 != 0 {
            patched.push(0);
        }
    }
    Ok(patched)
}

fn validate_root(bytes: &[u8]) -> Result<(), RdbError> {
    if bytes.len() < 0x20 {
        return Err(RdbError::TooSmall);
    }

    if &bytes[0..8] != b"_DRK0000" {
        return Err(RdbError::InvalidRootMagic);
    }

    Ok(())
}

fn parse_header(bytes: &[u8]) -> Result<RdbHeader, RdbError> {
    Ok(RdbHeader {
        first_block_offset: read_u32(bytes, 0x08)?,
        declared_count: read_u32(bytes, 0x10)?,
        data_prefix: read_c_string(&bytes[0x18..0x20]),
    })
}

fn parse_blocks(bytes: &[u8], first_offset: usize) -> Result<Vec<RdbBlock>, RdbError> {
    let mut blocks = Vec::new();
    let mut offset = first_offset;

    while offset < bytes.len() {
        let block = parse_block(bytes, offset)?;
        offset = align4(offset + block.length as usize);
        blocks.push(block);
    }

    Ok(blocks)
}

fn parse_block(bytes: &[u8], offset: usize) -> Result<RdbBlock, RdbError> {
    if bytes.len() - offset < 0x30 {
        return Err(RdbError::TruncatedBlock {
            offset,
            length: 0x30,
        });
    }

    if &bytes[offset..offset + 4] != b"IDRK" {
        return Err(RdbError::InvalidBlockMagic { offset });
    }

    let length = read_u32(bytes, offset + 0x08)?;
    if length < 0x30 {
        return Err(RdbError::InvalidBlockLength { offset, length });
    }

    let end = offset + length as usize;
    if end > bytes.len() {
        return Err(RdbError::TruncatedBlock { offset, length });
    }

    Ok(RdbBlock {
        offset,
        length,
        kind: bytes[offset..offset + 4].try_into().unwrap(),
        field_10: read_u32(bytes, offset + 0x10)?,
        data_offset: read_u32(bytes, offset + 0x18)?,
        field_20: read_u32(bytes, offset + 0x20)?,
        primary_hash: read_u32(bytes, offset + 0x24)?,
        field_28: read_u32(bytes, offset + 0x28)?,
        field_2c: read_u32(bytes, offset + 0x2c)?,
        payload: bytes[offset + 0x30..end].to_vec(),
        raw: bytes[offset..end].to_vec(),
    })
}

fn build_virtual_block(
    template: &RdbBlock,
    spec: &RdbAppendSpec,
) -> Result<Vec<u8>, RdbAppendError> {
    let address_len = template.field_10 as usize;
    if address_len == 0 || address_len > template.raw.len() {
        return Err(RdbAppendError::TemplateHasNoAddressTail(
            template.primary_hash,
        ));
    }
    if spec.payload_size > u32::MAX as u64 {
        return Err(RdbAppendError::SizeTooLarge(spec.payload_size));
    }

    let prefix_len = template.raw.len() - address_len;
    let tail = format!("0@{:x}#{}", spec.payload_size, spec.virtual_bin_suffix);
    let new_len = prefix_len + tail.len() + 1;
    let new_len_u32 = u32::try_from(new_len).map_err(|_| RdbAppendError::BlockTooLarge(new_len))?;
    let address_len_u32 =
        u32::try_from(tail.len() + 1).map_err(|_| RdbAppendError::BlockTooLarge(tail.len() + 1))?;

    let mut block = template.raw[..prefix_len].to_vec();
    block[0x08..0x0c].copy_from_slice(&new_len_u32.to_le_bytes());
    block[0x10..0x14].copy_from_slice(&address_len_u32.to_le_bytes());
    block[0x18..0x1c].copy_from_slice(&0u32.to_le_bytes());
    block[0x24..0x28].copy_from_slice(&spec.private_hash.to_le_bytes());
    block.extend_from_slice(tail.as_bytes());
    block.push(0);
    Ok(block)
}
