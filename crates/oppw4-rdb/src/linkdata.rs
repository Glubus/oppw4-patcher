use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

const LINKDATA_MAGIC: u32 = 0x0007_7df9;
const TABLE_OFFSET: usize = 0x10;
const RECORD_SIZE: usize = 0x10;
const OFFSET_GRANULARITY: usize = 0x100;
const REBUILD_CHUNK_SIZE: usize = 0x8000;

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
    InvalidEditIndex {
        index: usize,
        entry_count: usize,
    },
    Compress {
        index: usize,
        message: String,
    },
    RebuildValueTooLarge {
        index: usize,
        field: &'static str,
        value: usize,
    },
    EditedPayloadTooLarge {
        index: usize,
        payload_size: usize,
        capacity: usize,
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
        return original_compressed_payload(bytes, entry).map(|payload| payload.to_vec());
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

pub fn rebuild_linkdata_raw_with_edits(
    bytes: &[u8],
    edits: impl IntoIterator<Item = (usize, Vec<u8>)>,
) -> Result<Vec<u8>, LinkDataError> {
    let index = parse_linkdata(bytes)?;
    let entry_count = index.entries.len();
    let mut edits = edits.into_iter().collect::<BTreeMap<_, _>>();
    if let Some((&invalid, _)) = edits
        .iter()
        .find(|(entry_index, _)| **entry_index >= entry_count)
    {
        return Err(LinkDataError::InvalidEditIndex {
            index: invalid,
            entry_count,
        });
    }

    let table_end = TABLE_OFFSET
        .checked_add(entry_count * RECORD_SIZE)
        .ok_or(LinkDataError::TooSmall)?;
    let data_start = align_up(table_end, OFFSET_GRANULARITY).ok_or(LinkDataError::TooSmall)?;
    let mut output = vec![0u8; data_start];
    output[..TABLE_OFFSET].copy_from_slice(&bytes[..TABLE_OFFSET]);

    for entry in &index.entries {
        let payload = match edits.remove(&entry.index) {
            Some(inflated) => inflated,
            None => inflate_linkdata_entry(bytes, entry)?,
        };

        if payload.is_empty() {
            write_rebuilt_record(&mut output, entry.table_offset, 0, entry.field_04, 0, 0)?;
            continue;
        }

        while output.len() % OFFSET_GRANULARITY != 0 {
            output.push(0);
        }
        let offset_units = output.len() / OFFSET_GRANULARITY;
        write_rebuilt_record(
            &mut output,
            entry.table_offset,
            offset_units,
            entry.field_04,
            payload.len(),
            0,
        )?;
        output.extend_from_slice(&payload);
    }

    Ok(output)
}

pub fn rebuild_linkdata_with_edits(
    bytes: &[u8],
    edits: impl IntoIterator<Item = (usize, Vec<u8>)>,
) -> Result<Vec<u8>, LinkDataError> {
    let index = parse_linkdata(bytes)?;
    let entry_count = index.entries.len();
    let mut edits = edits.into_iter().collect::<BTreeMap<_, _>>();
    if let Some((&invalid, _)) = edits
        .iter()
        .find(|(entry_index, _)| **entry_index >= entry_count)
    {
        return Err(LinkDataError::InvalidEditIndex {
            index: invalid,
            entry_count,
        });
    }

    let table_end = TABLE_OFFSET
        .checked_add(entry_count * RECORD_SIZE)
        .ok_or(LinkDataError::TooSmall)?;
    let data_start = align_up(table_end, OFFSET_GRANULARITY).ok_or(LinkDataError::TooSmall)?;
    let mut output = vec![0u8; data_start];
    output[..TABLE_OFFSET].copy_from_slice(&bytes[..TABLE_OFFSET]);

    for entry in &index.entries {
        let (payload, uncompressed_size) = match edits.remove(&entry.index) {
            Some(inflated) => (
                encode_linkdata_payload(entry.index, &inflated)?,
                inflated.len(),
            ),
            None => (
                original_compressed_payload(bytes, entry)?.to_vec(),
                entry.uncompressed_size as usize,
            ),
        };

        if payload.is_empty() && uncompressed_size == 0 {
            write_rebuilt_record(&mut output, entry.table_offset, 0, entry.field_04, 0, 0)?;
            continue;
        }

        while output.len() % OFFSET_GRANULARITY != 0 {
            output.push(0);
        }
        let offset_units = output.len() / OFFSET_GRANULARITY;
        write_rebuilt_record(
            &mut output,
            entry.table_offset,
            offset_units,
            entry.field_04,
            payload.len(),
            uncompressed_size,
        )?;
        output.extend_from_slice(&payload);
    }

    Ok(output)
}

pub fn patch_linkdata_entries_in_place(
    bytes: &[u8],
    edits: impl IntoIterator<Item = (usize, Vec<u8>)>,
) -> Result<Vec<u8>, LinkDataError> {
    let index = parse_linkdata(bytes)?;
    let entry_count = index.entries.len();
    let edits = edits.into_iter().collect::<BTreeMap<_, _>>();
    if let Some((&invalid, _)) = edits
        .iter()
        .find(|(entry_index, _)| **entry_index >= entry_count)
    {
        return Err(LinkDataError::InvalidEditIndex {
            index: invalid,
            entry_count,
        });
    }

    let mut output = bytes.to_vec();
    for (entry_index, inflated) in edits {
        let entry = &index.entries[entry_index];
        let payload = encode_linkdata_payload(entry.index, &inflated)?;
        let capacity = entry_capacity(bytes.len(), &index.entries, entry);
        if payload.len() > capacity {
            return Err(LinkDataError::EditedPayloadTooLarge {
                index: entry.index,
                payload_size: payload.len(),
                capacity,
            });
        }

        write_rebuilt_record(
            &mut output,
            entry.table_offset,
            entry.data_offset / OFFSET_GRANULARITY,
            entry.field_04,
            payload.len(),
            inflated.len(),
        )?;
        let start = entry.data_offset;
        let end = start + payload.len();
        output[start..end].copy_from_slice(&payload);
        output[end..start + capacity].fill(0);
    }

    Ok(output)
}

fn entry_capacity(file_len: usize, entries: &[LinkDataEntry], entry: &LinkDataEntry) -> usize {
    let next = entries
        .iter()
        .filter(|candidate| candidate.data_offset > entry.data_offset)
        .map(|candidate| candidate.data_offset)
        .min()
        .unwrap_or(file_len);
    next.saturating_sub(entry.data_offset)
}

fn write_rebuilt_record(
    output: &mut [u8],
    table_offset: usize,
    offset_units: usize,
    field_04: u32,
    compressed_span: usize,
    uncompressed_size: usize,
) -> Result<(), LinkDataError> {
    write_u32_checked(output, table_offset, offset_units, "data_offset_units")?;
    write_u32(output, table_offset + 0x04, field_04);
    write_u32_checked(
        output,
        table_offset + 0x08,
        compressed_span,
        "compressed_span",
    )?;
    write_u32_checked(
        output,
        table_offset + 0x0c,
        uncompressed_size,
        "uncompressed_size",
    )
}

fn original_compressed_payload<'a>(
    bytes: &'a [u8],
    entry: &LinkDataEntry,
) -> Result<&'a [u8], LinkDataError> {
    let span = entry.compressed_span as usize;
    if span == 0 {
        return Ok(&[]);
    }
    let end = entry
        .data_offset
        .checked_add(span)
        .ok_or(LinkDataError::EntryOutOfBounds {
            index: entry.index,
            data_offset: entry.data_offset,
        })?;
    bytes
        .get(entry.data_offset..end)
        .ok_or(LinkDataError::TruncatedChunk {
            index: entry.index,
            offset: entry.data_offset,
            size: span,
            available: bytes.len().saturating_sub(entry.data_offset),
        })
}

fn encode_linkdata_payload(index: usize, inflated: &[u8]) -> Result<Vec<u8>, LinkDataError> {
    if inflated.is_empty() {
        return Ok(Vec::new());
    }

    let mut payload = Vec::new();
    payload.extend_from_slice(&(inflated.len() as u32).to_le_bytes());
    for chunk in inflated.chunks(REBUILD_CHUNK_SIZE) {
        let compressed = zlib_compress(index, chunk)?;
        payload.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        payload.extend_from_slice(&compressed);
    }
    Ok(payload)
}

fn zlib_compress(index: usize, bytes: &[u8]) -> Result<Vec<u8>, LinkDataError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(bytes)
        .map_err(|error| LinkDataError::Compress {
            index,
            message: error.to_string(),
        })?;
    encoder.finish().map_err(|error| LinkDataError::Compress {
        index,
        message: error.to_string(),
    })
}

fn align_up(value: usize, alignment: usize) -> Option<usize> {
    value
        .checked_add(alignment.checked_sub(1)?)
        .map(|value| value / alignment * alignment)
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

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u32_checked(
    bytes: &mut [u8],
    offset: usize,
    value: usize,
    field: &'static str,
) -> Result<(), LinkDataError> {
    let value = u32::try_from(value).map_err(|_| LinkDataError::RebuildValueTooLarge {
        index: (offset - TABLE_OFFSET) / RECORD_SIZE,
        field,
        value,
    })?;
    write_u32(bytes, offset, value);
    Ok(())
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
                field_04: 0x10,
                compressed_span: (bytes.len() - 0x100) as u32,
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

    #[test]
    fn rebuilds_linkdata_with_modified_entry_payload() {
        let bytes = synthetic_linkdata_entries(&[b"hello".as_slice(), b"world".as_slice()]);

        let rebuilt =
            rebuild_linkdata_with_edits(&bytes, [(1usize, b"patched world".to_vec())]).unwrap();
        let parsed = parse_linkdata(&rebuilt).unwrap();

        assert_eq!(parsed.entry_count, 2);
        assert_eq!(parsed.entries[0].field_04, 0x10);
        assert_eq!(parsed.entries[1].field_04, 0x11);
        assert_eq!(
            inflate_linkdata_entry(&rebuilt, &parsed.entries[0]).unwrap(),
            b"hello"
        );
        assert_eq!(
            inflate_linkdata_entry(&rebuilt, &parsed.entries[1]).unwrap(),
            b"patched world"
        );
        assert_eq!(parsed.entries[0].data_offset % OFFSET_GRANULARITY, 0);
        assert_eq!(parsed.entries[1].data_offset % OFFSET_GRANULARITY, 0);
        assert_eq!(parsed.entries[1].uncompressed_size, 13);
    }

    #[test]
    fn patches_linkdata_entry_in_place_when_payload_fits_capacity() {
        let mut bytes = synthetic_linkdata_entries(&[b"hello".as_slice(), b"world".as_slice()]);
        bytes.extend_from_slice(&[0xcc; 0x80]);
        let original_len = bytes.len();
        let original = parse_linkdata(&bytes).unwrap();
        let original_entry1_offset = original.entries[1].data_offset;

        let patched =
            patch_linkdata_entries_in_place(&bytes, [(1usize, b"patched world".to_vec())]).unwrap();
        let parsed = parse_linkdata(&patched).unwrap();

        assert_eq!(patched.len(), original_len);
        assert_eq!(parsed.entries[1].data_offset, original_entry1_offset);
        assert_eq!(
            inflate_linkdata_entry(&patched, &parsed.entries[0]).unwrap(),
            b"hello"
        );
        assert_eq!(
            inflate_linkdata_entry(&patched, &parsed.entries[1]).unwrap(),
            b"patched world"
        );
    }

    fn synthetic_linkdata(chunks: &[&[u8]]) -> Vec<u8> {
        synthetic_linkdata_entries(&[&chunks.concat()])
    }

    fn synthetic_linkdata_entries(entries: &[&[u8]]) -> Vec<u8> {
        let mut data = vec![0u8; 0x100];
        data[0..4].copy_from_slice(&LINKDATA_MAGIC.to_le_bytes());
        data[4..8].copy_from_slice(&(entries.len() as u32).to_le_bytes());

        for (index, entry) in entries.iter().enumerate() {
            while data.len() % OFFSET_GRANULARITY != 0 {
                data.push(0);
            }
            let table_offset = 0x10 + index * RECORD_SIZE;
            let offset_units = (data.len() / OFFSET_GRANULARITY) as u32;
            data[table_offset..table_offset + 4].copy_from_slice(&offset_units.to_le_bytes());
            data[table_offset + 4..table_offset + 8]
                .copy_from_slice(&(0x10u32 + index as u32).to_le_bytes());

            let payload_start = data.len();
            data.extend_from_slice(&(entry.len() as u32).to_le_bytes());
            let compressed = zlib(entry);
            data.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
            data.extend_from_slice(&compressed);
            let compressed_span = (data.len() - payload_start) as u32;
            data[table_offset + 8..table_offset + 12]
                .copy_from_slice(&compressed_span.to_le_bytes());
            data[table_offset + 12..table_offset + 16]
                .copy_from_slice(&(entry.len() as u32).to_le_bytes());
        }

        data
    }

    fn zlib(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }
}
