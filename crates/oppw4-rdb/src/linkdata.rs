use std::io::Read;

use flate2::read::ZlibDecoder;

const LINKDATA_MAGIC: u32 = 0x0007_7df9;
const TABLE_OFFSET: usize = 0x10;
const RECORD_SIZE: usize = 0x10;
const OFFSET_GRANULARITY: usize = 0x100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDataIndex {
    pub entry_count: u32,
    pub entries: Vec<LinkDataEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDataEntry {
    pub index: usize,
    pub table_offset: usize,
    pub data_offset: usize,
    pub field_04: u32,
    pub compressed_span: u32,
    pub uncompressed_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkDataError {
    TooSmall,
    InvalidMagic {
        found: u32,
    },
    TruncatedTable {
        needed: usize,
        available: usize,
    },
    EntryOutOfBounds {
        index: usize,
        data_offset: usize,
    },
    TruncatedChunkHeader {
        index: usize,
        offset: usize,
    },
    TruncatedChunk {
        index: usize,
        offset: usize,
        size: usize,
        available: usize,
    },
    InflatedPastDeclaredSize {
        index: usize,
        declared: usize,
        inflated: usize,
    },
    Inflate {
        index: usize,
        offset: usize,
        message: String,
    },
}

pub fn parse_linkdata(bytes: &[u8]) -> Result<LinkDataIndex, LinkDataError> {
    let magic = read_u32(bytes, 0)?;
    if magic != LINKDATA_MAGIC {
        return Err(LinkDataError::InvalidMagic { found: magic });
    }

    let entry_count = read_u32(bytes, 4)?;
    let table_len = entry_count as usize * RECORD_SIZE;
    let table_end = TABLE_OFFSET
        .checked_add(table_len)
        .ok_or(LinkDataError::TooSmall)?;
    if table_end > bytes.len() {
        return Err(LinkDataError::TruncatedTable {
            needed: table_end,
            available: bytes.len(),
        });
    }

    let mut entries = Vec::with_capacity(entry_count as usize);
    for index in 0..entry_count as usize {
        let table_offset = TABLE_OFFSET + index * RECORD_SIZE;
        let offset_units = read_u32(bytes, table_offset)? as usize;
        entries.push(LinkDataEntry {
            index,
            table_offset,
            data_offset: offset_units * OFFSET_GRANULARITY,
            field_04: read_u32(bytes, table_offset + 0x04)?,
            compressed_span: read_u32(bytes, table_offset + 0x08)?,
            uncompressed_size: read_u32(bytes, table_offset + 0x0c)?,
        });
    }

    Ok(LinkDataIndex {
        entry_count,
        entries,
    })
}

pub fn inflate_linkdata_entry(
    bytes: &[u8],
    entry: &LinkDataEntry,
) -> Result<Vec<u8>, LinkDataError> {
    let declared = entry.uncompressed_size as usize;
    if declared == 0 {
        return Ok(Vec::new());
    }
    if entry.data_offset + 4 > bytes.len() {
        return Err(LinkDataError::EntryOutOfBounds {
            index: entry.index,
            data_offset: entry.data_offset,
        });
    }

    let block_uncompressed = read_u32(bytes, entry.data_offset)? as usize;
    let expected = declared.min(block_uncompressed);
    let mut output = Vec::with_capacity(expected);
    let mut cursor = entry.data_offset + 4;

    while output.len() < expected {
        let compressed_size = read_chunk_size(bytes, entry, cursor)?;
        let chunk_start = cursor + 4;
        let chunk_end = chunk_start + compressed_size;
        let chunk = read_chunk(bytes, entry, chunk_start, chunk_end)?;
        output.extend_from_slice(&chunk);
        if output.len() > expected {
            return Err(LinkDataError::InflatedPastDeclaredSize {
                index: entry.index,
                declared: expected,
                inflated: output.len(),
            });
        }
        cursor = chunk_end;
    }

    Ok(output)
}

fn read_chunk_size(
    bytes: &[u8],
    entry: &LinkDataEntry,
    cursor: usize,
) -> Result<usize, LinkDataError> {
    if cursor + 4 > bytes.len() {
        return Err(LinkDataError::TruncatedChunkHeader {
            index: entry.index,
            offset: cursor,
        });
    }
    Ok(read_u32(bytes, cursor)? as usize)
}

fn read_chunk(
    bytes: &[u8],
    entry: &LinkDataEntry,
    chunk_start: usize,
    chunk_end: usize,
) -> Result<Vec<u8>, LinkDataError> {
    let Some(chunk_bytes) = bytes.get(chunk_start..chunk_end) else {
        return Err(LinkDataError::TruncatedChunk {
            index: entry.index,
            offset: chunk_start,
            size: chunk_end.saturating_sub(chunk_start),
            available: bytes.len().saturating_sub(chunk_start),
        });
    };

    let mut decoder = ZlibDecoder::new(chunk_bytes);
    let mut chunk = Vec::new();
    decoder
        .read_to_end(&mut chunk)
        .map_err(|error| LinkDataError::Inflate {
            index: entry.index,
            offset: chunk_start,
            message: error.to_string(),
        })?;
    Ok(chunk)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, LinkDataError> {
    let Some(slice) = bytes.get(offset..offset + 4) else {
        return Err(LinkDataError::TooSmall);
    };
    Ok(u32::from_le_bytes(slice.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{write::ZlibEncoder, Compression};

    use super::*;

    #[test]
    fn parses_linkdata_table_records() {
        let bytes = synthetic_linkdata(&[b"hello"]);

        let parsed = parse_linkdata(&bytes).unwrap();

        assert_eq!(parsed.entry_count, 1);
        assert_eq!(
            parsed.entries[0],
            LinkDataEntry {
                index: 0,
                table_offset: 0x10,
                data_offset: 0x100,
                field_04: 0,
                compressed_span: (bytes.len() - 0x100 + 4) as u32,
                uncompressed_size: 5,
            }
        );
    }

    #[test]
    fn inflates_chunked_linkdata_entry() {
        let bytes = synthetic_linkdata(&[b"hello ", b"world"]);
        let parsed = parse_linkdata(&bytes).unwrap();

        let inflated = inflate_linkdata_entry(&bytes, &parsed.entries[0]).unwrap();

        assert_eq!(inflated, b"hello world");
    }

    fn synthetic_linkdata(chunks: &[&[u8]]) -> Vec<u8> {
        let mut data = vec![0u8; 0x100];
        data[0..4].copy_from_slice(&LINKDATA_MAGIC.to_le_bytes());
        data[4..8].copy_from_slice(&1u32.to_le_bytes());
        data[0x10..0x14].copy_from_slice(&1u32.to_le_bytes());
        data[0x14..0x18].copy_from_slice(&0u32.to_le_bytes());

        let uncompressed_size: usize = chunks.iter().map(|chunk| chunk.len()).sum();
        data.extend_from_slice(&(uncompressed_size as u32).to_le_bytes());
        for chunk in chunks {
            let compressed = zlib(chunk);
            data.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
            data.extend_from_slice(&compressed);
        }

        let compressed_span = (data.len() - 0x100 + 4) as u32;
        data[0x18..0x1c].copy_from_slice(&compressed_span.to_le_bytes());
        data[0x1c..0x20].copy_from_slice(&(uncompressed_size as u32).to_le_bytes());
        data
    }

    fn zlib(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }
}
