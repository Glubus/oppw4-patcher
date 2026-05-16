use std::fs;

pub(crate) struct DumpCommand {
    path: String,
    max_chunks: usize,
    max_words: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct KidsContainer {
    pub(crate) declared_size: u32,
    pub(crate) chunks: Vec<KidsChunk>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct KidsChunk {
    pub(crate) offset: usize,
    pub(crate) size: u32,
    pub(crate) payload_offset: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AlignedWord {
    pub(crate) offset: usize,
    pub(crate) value: u32,
}

pub(crate) fn parse_command(mut args: impl Iterator<Item = String>) -> Result<DumpCommand, String> {
    let Some(path) = args.next() else {
        return Err(usage());
    };

    let mut max_chunks = 64;
    let mut max_words = 12;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--max-chunks" => {
                let Some(raw) = args.next() else {
                    return Err(usage());
                };
                max_chunks = parse_usize(&raw, "--max-chunks")?;
            }
            "--max-words" => {
                let Some(raw) = args.next() else {
                    return Err(usage());
                };
                max_words = parse_usize(&raw, "--max-words")?;
            }
            _ => return Err(format!("unknown kidsdb-dump option: {arg}\n{}", usage())),
        }
    }

    Ok(DumpCommand {
        path,
        max_chunks,
        max_words,
    })
}

pub(crate) fn run(command: DumpCommand) -> Result<(), String> {
    let bytes = fs::read(&command.path)
        .map_err(|error| format!("failed to read KIDS DB {}: {error}", command.path))?;
    let container = parse_container(&bytes)?;

    println!(
        "kidsdb path={} declared_size=0x{:x} file_size=0x{:x} chunks={}",
        command.path,
        container.declared_size,
        bytes.len(),
        container.chunks.len()
    );

    for chunk in container.chunks.iter().take(command.max_chunks) {
        let payload_len = chunk.payload_len();
        let payload_end = chunk.payload_offset + payload_len;
        let words = aligned_words(
            &bytes[chunk.payload_offset..payload_end],
            chunk.payload_offset,
        );
        let word_preview = words
            .iter()
            .take(command.max_words)
            .map(|word| format!("0x{:x}:0x{:08x}", word.offset, word.value))
            .collect::<Vec<_>>()
            .join(",");

        println!(
            "chunk offset=0x{:x} total_size=0x{:x} payload=0x{:x} words={}",
            chunk.offset, chunk.size, payload_len, word_preview
        );
    }

    Ok(())
}

pub(crate) fn parse_container(bytes: &[u8]) -> Result<KidsContainer, String> {
    if bytes.len() < 12 {
        return Err("file is too small for an IDRK header".to_string());
    }
    if &bytes[0..8] != b"IDRK0000" {
        return Err("missing IDRK0000 header".to_string());
    }

    let declared_size = read_u32(bytes, 8)?;
    if declared_size as usize != bytes.len() {
        return Err(format!(
            "declared size 0x{declared_size:x} does not match file size 0x{:x}",
            bytes.len()
        ));
    }

    let mut chunks = Vec::new();
    let mut offset = 12;
    while offset + 12 <= bytes.len() {
        if &bytes[offset..offset + 8] != b"IDOK0000" {
            offset += 1;
            continue;
        }

        let size = read_u32(bytes, offset + 8)?;
        if size < 12 {
            return Err(format!(
                "IDOK chunk at 0x{offset:x} has invalid size 0x{size:x}"
            ));
        }

        let end = offset
            .checked_add(size as usize)
            .ok_or_else(|| format!("IDOK chunk at 0x{offset:x} overflows usize"))?;
        if end > bytes.len() {
            return Err(format!(
                "IDOK chunk at 0x{offset:x} ends at 0x{end:x}, past file size 0x{:x}",
                bytes.len()
            ));
        }

        chunks.push(KidsChunk {
            offset,
            size,
            payload_offset: offset + 12,
        });
        offset = end;
    }

    Ok(KidsContainer {
        declared_size,
        chunks,
    })
}

pub(crate) fn aligned_words(payload: &[u8], base_offset: usize) -> Vec<AlignedWord> {
    payload
        .chunks_exact(4)
        .enumerate()
        .map(|(index, bytes)| AlignedWord {
            offset: base_offset + index * 4,
            value: u32::from_le_bytes(bytes.try_into().expect("chunks_exact returns 4 bytes")),
        })
        .collect()
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("u32 read at 0x{offset:x} is out of range"))?;
    Ok(u32::from_le_bytes(
        raw.try_into().expect("slice length was checked"),
    ))
}

impl KidsChunk {
    pub(crate) fn payload_len(&self) -> usize {
        self.size as usize - 12
    }
}

fn parse_usize(raw: &str, label: &str) -> Result<usize, String> {
    raw.parse::<usize>()
        .map_err(|_| format!("invalid {label}: {raw}"))
}

fn usage() -> String {
    "usage: oppw4-rdb --kidsdb-dump <kids-db-file> [--max-chunks <n>] [--max-words <n>]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_idok_chunks_inside_idrk_container() {
        let mut bytes = vec![0; 0x44];
        bytes[0..8].copy_from_slice(b"IDRK0000");
        let declared_size = bytes.len() as u32;
        bytes[8..12].copy_from_slice(&declared_size.to_le_bytes());
        bytes[0x20..0x28].copy_from_slice(b"IDOK0000");
        bytes[0x28..0x2c].copy_from_slice(&0x10u32.to_le_bytes());
        bytes[0x2c..0x30].copy_from_slice(&0x11223344u32.to_le_bytes());
        bytes[0x30..0x38].copy_from_slice(b"IDOK0000");
        bytes[0x38..0x3c].copy_from_slice(&0x14u32.to_le_bytes());
        bytes[0x3c..0x44].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);

        let parsed = parse_container(&bytes).expect("container should parse");

        assert_eq!(parsed.declared_size, 0x44);
        assert_eq!(
            parsed.chunks,
            vec![
                KidsChunk {
                    offset: 0x20,
                    size: 0x10,
                    payload_offset: 0x2c,
                },
                KidsChunk {
                    offset: 0x30,
                    size: 0x14,
                    payload_offset: 0x3c,
                },
            ]
        );
    }

    #[test]
    fn reports_aligned_u32_words_with_file_offsets() {
        let payload = [
            0xa0, 0x71, 0x6d, 0x38, 0x01, 0x00, 0x00, 0x00, 0x79, 0x2a, 0xfc, 0xfb,
        ];

        assert_eq!(
            aligned_words(&payload, 0x2c),
            vec![
                AlignedWord {
                    offset: 0x2c,
                    value: 0x386d71a0,
                },
                AlignedWord {
                    offset: 0x30,
                    value: 1,
                },
                AlignedWord {
                    offset: 0x34,
                    value: 0xfbfc2a79,
                },
            ]
        );
    }
}
