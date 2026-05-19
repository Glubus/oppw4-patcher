use std::collections::BTreeMap;

use flate2::read::ZlibDecoder;
use std::io::Read;

use crate::constants::{LINKDATA_MAGIC, OFFSET_GRANULARITY, RECORD_SIZE, TABLE_OFFSET};

#[derive(Clone, Debug)]
struct LinkDataEntry {
    index: usize,
    table_offset: usize,
    data_offset: usize,
    field_04: u32,
    compressed_span: usize,
    uncompressed_size: usize,
}

pub(crate) fn rebuild_raw_with_edits(
    bytes: &[u8],
    edits: &BTreeMap<usize, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let entries = parse(bytes)?;
    let table_end = TABLE_OFFSET + entries.len() * RECORD_SIZE;
    let data_start = align_up(table_end, OFFSET_GRANULARITY);
    let mut output = vec![0u8; data_start];
    output[..TABLE_OFFSET].copy_from_slice(&bytes[..TABLE_OFFSET]);

    for entry in &entries {
        let payload = match edits.get(&entry.index) {
            Some(payload) => payload.clone(),
            None => inflate_entry(bytes, entry)?,
        };
        while output.len() % OFFSET_GRANULARITY != 0 {
            output.push(0);
        }
        let offset_units = output.len() / OFFSET_GRANULARITY;
        write_u32(&mut output, entry.table_offset, offset_units as u32);
        write_u32(&mut output, entry.table_offset + 0x04, entry.field_04);
        write_u32(&mut output, entry.table_offset + 0x08, payload.len() as u32);
        write_u32(&mut output, entry.table_offset + 0x0c, 0);
        output.extend_from_slice(&payload);
    }
    Ok(output)
}

fn parse(bytes: &[u8]) -> Result<Vec<LinkDataEntry>, String> {
    if bytes.len() < TABLE_OFFSET || read_u32(bytes, 0)? != LINKDATA_MAGIC {
        return Err("invalid LINKDATA_A header".to_string());
    }
    let count = read_u32(bytes, 4)? as usize;
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let table_offset = TABLE_OFFSET + index * RECORD_SIZE;
        if table_offset + RECORD_SIZE > bytes.len() {
            return Err(format!("truncated LINKDATA table at entry {index}"));
        }
        entries.push(LinkDataEntry {
            index,
            table_offset,
            data_offset: read_u32(bytes, table_offset)? as usize * OFFSET_GRANULARITY,
            field_04: read_u32(bytes, table_offset + 0x04)?,
            compressed_span: read_u32(bytes, table_offset + 0x08)? as usize,
            uncompressed_size: read_u32(bytes, table_offset + 0x0c)? as usize,
        });
    }
    Ok(entries)
}

fn inflate_entry(bytes: &[u8], entry: &LinkDataEntry) -> Result<Vec<u8>, String> {
    if entry.uncompressed_size == 0 {
        let end = entry.data_offset + entry.compressed_span;
        return bytes
            .get(entry.data_offset..end)
            .map(|payload| payload.to_vec())
            .ok_or_else(|| format!("entry {} out of bounds", entry.index));
    }
    let declared = entry.uncompressed_size;
    let block_uncompressed = read_u32(bytes, entry.data_offset)? as usize;
    let expected = declared.min(block_uncompressed);
    let mut cursor = entry.data_offset + 4;
    let mut output = Vec::with_capacity(expected);
    while output.len() < expected {
        let compressed_size = read_u32(bytes, cursor)? as usize;
        cursor += 4;
        let end = cursor + compressed_size;
        let chunk = bytes
            .get(cursor..end)
            .ok_or_else(|| format!("entry {} truncated chunk", entry.index))?;
        let mut decoder = ZlibDecoder::new(chunk);
        decoder
            .read_to_end(&mut output)
            .map_err(|error| format!("entry {} inflate failed: {error}", entry.index))?;
        cursor = end;
    }
    Ok(output)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let Some(raw) = bytes.get(offset..offset + 4) else {
        return Err(format!("read_u32 out of bounds at 0x{offset:x}"));
    };
    Ok(u32::from_le_bytes(raw.try_into().unwrap()))
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
