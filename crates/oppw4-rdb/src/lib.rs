mod address;
mod bytes;
mod catalog;
mod hash;
mod mod_source;
mod rdb;
mod scan;
mod r#virtual;

pub use address::{parse_block_tail, parse_payload_tail, RdbAddressSuffix, RdbPayloadTail};
pub use catalog::{parse_name_hash_catalog, NameHashEntry};
pub use hash::parse_prefixed_hex_hash;
pub use mod_source::{ModAsset, ReadSeek, ReplacementSource};
pub use r#virtual::{
    assign_virtual_bin_offsets, attach_mod_file_sizes, build_virtualization_table,
    build_virtualization_table_from_assets, open_virtual_replacement, ReplacementMode, VirtualFile,
    VirtualHandle, VirtualHandleTable, VirtualManager, VirtualReplacement, VirtualReplacementFile,
};
pub use rdb::{parse_rdb, RdbBlock, RdbError, RdbHeader, RdbIndex};
pub use scan::{
    scan_archive_names_with_catalog, scan_virtualized_names, scan_virtualized_names_with_catalog,
    ArchiveScan, ArchiveScanCounts, VirtualizedFile,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn put_u32(buf: &mut [u8], offset: usize, value: u32) {
        buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn fixture() -> Vec<u8> {
        let mut bytes = vec![0u8; 0x20];
        bytes[0..8].copy_from_slice(b"_DRK0000");
        put_u32(&mut bytes, 0x08, 0x20);
        put_u32(&mut bytes, 0x10, 2);
        bytes[0x18..0x1d].copy_from_slice(b"data/");

        let block0_offset = bytes.len();
        let mut block0 = vec![0u8; 0x44];
        block0[0..8].copy_from_slice(b"IDRK0000");
        put_u32(&mut block0, 0x08, 0x44);
        put_u32(&mut block0, 0x10, 0x0c);
        put_u32(&mut block0, 0x18, 0x100);
        put_u32(&mut block0, 0x20, 0);
        put_u32(&mut block0, 0x24, 0x0011c397);
        put_u32(&mut block0, 0x28, 0x56efe45c);
        put_u32(&mut block0, 0x2c, 0x00020000);
        block0[0x38..0x44].copy_from_slice(b"11c3971@138\0");
        bytes.extend_from_slice(&block0);

        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
        assert_eq!(block0_offset, 0x20);

        let mut block1 = vec![0u8; 0x43];
        block1[0..8].copy_from_slice(b"IDRK0000");
        put_u32(&mut block1, 0x08, 0x43);
        put_u32(&mut block1, 0x10, 0x0b);
        put_u32(&mut block1, 0x18, 0xa0);
        put_u32(&mut block1, 0x20, 0);
        put_u32(&mut block1, 0x24, 0x009f7b2b);
        put_u32(&mut block1, 0x28, 0x56efe45c);
        put_u32(&mut block1, 0x2c, 0x00020000);
        block1[0x38..0x43].copy_from_slice(b"9f7b2b3@d8\0");
        bytes.extend_from_slice(&block1);
        bytes
    }

    #[test]
    fn parses_root_header() {
        let parsed = parse_rdb(&fixture()).unwrap();

        assert_eq!(parsed.header.first_block_offset, 0x20);
        assert_eq!(parsed.header.declared_count, 2);
        assert_eq!(parsed.header.data_prefix, "data/");
    }

    #[test]
    fn parses_aligned_idrk_blocks() {
        let parsed = parse_rdb(&fixture()).unwrap();

        assert_eq!(parsed.blocks.len(), 2);
        assert_eq!(parsed.blocks[0].offset, 0x20);
        assert_eq!(parsed.blocks[0].length, 0x44);
        assert_eq!(parsed.blocks[0].kind, *b"IDRK");
        assert_eq!(parsed.blocks[0].primary_hash, 0x0011c397);
        assert_eq!(&parsed.blocks[0].payload[..8], &[0; 8]);
        assert_eq!(&parsed.blocks[0].payload[8..], b"11c3971@138\0");

        assert_eq!(parsed.blocks[1].offset, 0x64);
        assert_eq!(parsed.blocks[1].length, 0x43);
        assert_eq!(parsed.blocks[1].primary_hash, 0x009f7b2b);
    }

    #[test]
    fn rejects_bad_root_magic() {
        let mut bytes = fixture();
        bytes[0] = b'X';

        assert_eq!(parse_rdb(&bytes), Err(RdbError::InvalidRootMagic));
    }

    #[test]
    fn rejects_truncated_block() {
        let mut bytes = fixture();
        bytes.truncate(0x50);

        assert_eq!(
            parse_rdb(&bytes),
            Err(RdbError::TruncatedBlock {
                offset: 0x20,
                length: 0x44
            })
        );
    }

    #[test]
    fn parses_prefixed_hex_hash_names() {
        assert_eq!(parse_prefixed_hex_hash("0x3b359352.g1m"), Some(0x3b359352));
        assert_eq!(parse_prefixed_hex_hash("0X4CE275FB.g1m"), Some(0x4ce275fb));
        assert_eq!(parse_prefixed_hex_hash("0x93dfb06c"), Some(0x93dfb06c));
        assert_eq!(parse_prefixed_hex_hash("MDLC038_Zoro_Wa.g1m"), None);
        assert_eq!(parse_prefixed_hex_hash("0x.g1m"), None);
    }

    #[test]
    fn parses_payload_tail() {
        let payload = [
            b"\0\0\0\0\xff\xff\xff\xff".as_slice(),
            b"1268d0@11c28b#8\0".as_slice(),
        ]
        .concat();

        assert_eq!(
            parse_payload_tail(&payload),
            Some(RdbPayloadTail {
                raw: "1268d0@11c28b#8".to_string(),
                part_a: 0x1268d0,
                part_b: 0x11c28b,
                suffix: Some(RdbAddressSuffix {
                    marker: '#',
                    value: 0x8,
                }),
            })
        );
    }

    #[test]
    fn parses_real_sequence_editor_index() {
        let parsed = parse_rdb(include_bytes!("../fixtures/rdb/SequenceEditor.rdb")).unwrap();

        assert_eq!(parsed.header.first_block_offset, 0x20);
        assert_eq!(parsed.header.data_prefix, "data/");
        assert!(!parsed.blocks.is_empty());
        assert_eq!(parsed.blocks.len(), parsed.header.declared_count as usize);
        assert!(parsed.blocks.iter().all(|block| block.kind == *b"IDRK"));
    }

    #[test]
    fn parses_known_real_payload_tail() {
        let parsed = parse_rdb(include_bytes!("../fixtures/rdb/CharacterEditor.rdb")).unwrap();
        let block = parsed
            .blocks
            .iter()
            .find(|block| block.primary_hash == 0x3b359352)
            .unwrap();

        assert_eq!(
            parse_payload_tail(&block.payload),
            Some(RdbPayloadTail {
                raw: "0@268005#9".to_string(),
                part_a: 0,
                part_b: 0x268005,
                suffix: Some(RdbAddressSuffix {
                    marker: '#',
                    value: 0x9,
                }),
            })
        );
        assert_eq!(parse_block_tail(block), parse_payload_tail(&block.payload));
    }

    #[test]
    fn parses_named_asset_address_forms() {
        assert_eq!(
            parse_payload_tail(b"\0\0\0\0c423de@300&0\0"),
            Some(RdbPayloadTail {
                raw: "c423de@300&0".to_string(),
                part_a: 0xc423de,
                part_b: 0x300,
                suffix: Some(RdbAddressSuffix {
                    marker: '&',
                    value: 0,
                }),
            })
        );
        assert_eq!(
            parse_payload_tail(b"\0\0\0\0b61b29a@ec\0"),
            Some(RdbPayloadTail {
                raw: "b61b29a@ec".to_string(),
                part_a: 0xb61b29a,
                part_b: 0xec,
                suffix: None,
            })
        );
    }

    #[test]
    fn scans_virtualized_hash_file_names() {
        let parsed = parse_rdb(include_bytes!("../fixtures/rdb/CharacterEditor.rdb")).unwrap();
        let scanned = scan_virtualized_names(
            &parsed,
            ["0x3b359352.g1m", "0xffffffff.g1m", "MDLC038_Zoro_Wa.g1m"],
        );

        assert_eq!(scanned.len(), 3);
        assert_eq!(scanned[0].file_name, "0x3b359352.g1m");
        assert_eq!(scanned[0].hash, Some(0x3b359352));
        assert_eq!(scanned[0].block.unwrap().primary_hash, 0x3b359352);
        assert_eq!(scanned[1].hash, Some(0xffffffff));
        assert!(scanned[1].block.is_none());
        assert_eq!(scanned[2].hash, None);
        assert!(scanned[2].block.is_none());
    }

    #[test]
    fn parses_exported_name_hash_catalog_lines() {
        let bytes = [
            b"noise,MPLC009_Ace.g1m".as_slice(),
            b"\r\n".as_slice(),
            b"0xa8bd2e49,MDLC038_Zoro_Wa.g1m".as_slice(),
            b"\r\n".as_slice(),
            b"0x1ad9ce2c,CE1_0080_STYLE_IMPACT_HAO.g1e".as_slice(),
            b"\r\n".as_slice(),
            b"0x1ad9ce2c,broken.g1m".as_slice(),
            &[0, 0],
            b"0xnothex".as_slice(),
        ]
        .concat();
        let catalog = parse_name_hash_catalog(&bytes);

        assert_eq!(
            catalog,
            vec![
                NameHashEntry {
                    name: "MDLC038_Zoro_Wa.g1m".to_string(),
                    hash: 0xa8bd2e49,
                },
                NameHashEntry {
                    name: "CE1_0080_STYLE_IMPACT_HAO.g1e".to_string(),
                    hash: 0x1ad9ce2c,
                },
            ]
        );
    }

    #[test]
    fn parses_embedded_name_hash_catalog_fallback() {
        let bytes = [
            b"MPLC009_Ace.g1m".as_slice(),
            b"\r\n".as_slice(),
            b"0xa8bd2e49".as_slice(),
            &[0, 0],
            b"MDLC038_Zoro_Wa.g1m".as_slice(),
            b"\r\n".as_slice(),
            b"0x1ad9ce2c".as_slice(),
        ]
        .concat();
        let catalog = parse_name_hash_catalog(&bytes);

        assert_eq!(
            catalog,
            vec![
                NameHashEntry {
                    name: "MPLC009_Ace.g1m".to_string(),
                    hash: 0xa8bd2e49,
                },
                NameHashEntry {
                    name: "MDLC038_Zoro_Wa.g1m".to_string(),
                    hash: 0x1ad9ce2c,
                },
            ]
        );
    }

    #[test]
    fn parses_embedded_hash_before_name_catalog_fallback() {
        let bytes = [
            &[0xff, 0],
            b"0x359b9672,800_294_face_law_dressrosa_External_00.g1t".as_slice(),
            &[0, 0],
            b"0x3bff0f13,801_294_chara_law_dressrosa_External_00.g1t".as_slice(),
            &[0, 0],
        ]
        .concat();
        let catalog = parse_name_hash_catalog(&bytes);

        assert_eq!(
            catalog,
            vec![
                NameHashEntry {
                    name: "800_294_face_law_dressrosa_External_00.g1t".to_string(),
                    hash: 0x359b9672,
                },
                NameHashEntry {
                    name: "801_294_chara_law_dressrosa_External_00.g1t".to_string(),
                    hash: 0x3bff0f13,
                },
            ]
        );
    }

    #[test]
    fn scans_virtualized_catalog_file_names() {
        let parsed = parse_rdb(include_bytes!("../fixtures/rdb/CharacterEditor.rdb")).unwrap();
        let catalog = vec![NameHashEntry {
            name: "MDLC038_Zoro_Wa.g1m".to_string(),
            hash: 0x1ad9ce2c,
        }];
        let scanned = scan_virtualized_names_with_catalog(
            &parsed,
            ["MDLC038_Zoro_Wa.g1m", "Unknown_Name.g1m"],
            &catalog,
        );

        assert_eq!(scanned[0].hash, Some(0x1ad9ce2c));
        assert_eq!(scanned[0].block.unwrap().primary_hash, 0x1ad9ce2c);
        assert_eq!(scanned[1].hash, None);
        assert!(scanned[1].block.is_none());
    }

    #[test]
    fn counts_archive_scan_results() {
        let parsed = parse_rdb(include_bytes!("../fixtures/rdb/CharacterEditor.rdb")).unwrap();
        let scan = scan_archive_names_with_catalog(
            "CharacterEditor",
            &parsed,
            ["0x3b359352.g1m", "0xffffffff.g1m", "Unknown_Name.g1m"],
            &[],
        );

        assert_eq!(scan.archive_name, "CharacterEditor");
        assert_eq!(
            scan.counts(),
            ArchiveScanCounts {
                total: 3,
                matched: 1,
                hash_missing: 1,
                unresolved_names: 1,
            }
        );
    }

    #[test]
    fn builds_virtualization_table_from_matched_files() {
        let parsed = parse_rdb(include_bytes!("../fixtures/rdb/CharacterEditor.rdb")).unwrap();
        let scan = scan_archive_names_with_catalog(
            "CharacterEditor",
            &parsed,
            ["0x3b359352.g1m", "0xffffffff.g1m", "Unknown_Name.g1m"],
            &[],
        );

        let table = build_virtualization_table(&scan, "mods/CharacterEditor");
        let matched_block = parsed
            .blocks
            .iter()
            .find(|block| block.primary_hash == 0x3b359352)
            .unwrap();
        let prefix_len = matched_block.raw.len() - matched_block.field_10 as usize;

        assert_eq!(
            table,
            vec![VirtualReplacement {
                archive_name: "CharacterEditor".to_string(),
                file_name: "0x3b359352.g1m".to_string(),
                source: ReplacementSource::File(
                    std::path::PathBuf::from("mods/CharacterEditor").join("0x3b359352.g1m"),
                ),
                mode: ReplacementMode::Virtual,
                mod_size: None,
                hash: 0x3b359352,
                rdb_block_offset: 0x8f28,
                original_data_offset: 0x420a0c,
                original_bin_offset: Some(0),
                original_bin_size: Some(0x268005),
                virtual_bin_offset: None,
                rdb_tail_offset: Some(0x8f90),
                original_tail: Some("0@268005#9".to_string()),
                virtual_prefix: Some(matched_block.raw[..prefix_len].to_vec()),
            }]
        );
    }

    #[test]
    fn builds_virtualization_table_from_zip_asset_source() {
        let parsed = parse_rdb(include_bytes!("../fixtures/rdb/CharacterEditor.rdb")).unwrap();
        let scan =
            scan_archive_names_with_catalog("CharacterEditor", &parsed, ["0x3b359352.g1m"], &[]);
        let source = ReplacementSource::ZipEntry {
            zip_path: "mods/law.zip".into(),
            entry_name: "CharacterEditor/0x3b359352.g1m".to_string(),
        };

        let table = build_virtualization_table_from_assets(
            &scan,
            &[ModAsset {
                file_name: "0x3b359352.g1m".to_string(),
                source: source.clone(),
            }],
        );

        assert_eq!(table.len(), 1);
        assert_eq!(table[0].source, source);
        assert_eq!(table[0].mode, ReplacementMode::Virtual);
        assert_eq!(table[0].hash, 0x3b359352);
    }

    #[test]
    fn attaches_mod_file_sizes_to_replacements() {
        let temp_dir =
            std::env::temp_dir().join(format!("oppw4-rdb-tools-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mod_path = temp_dir.join("0x3b359352.g1m");
        std::fs::write(&mod_path, [1u8, 2, 3, 4, 5]).unwrap();

        let replacements = vec![VirtualReplacement {
            archive_name: "CharacterEditor".to_string(),
            file_name: "0x3b359352.g1m".to_string(),
            source: ReplacementSource::File(mod_path.clone()),
            mode: ReplacementMode::Virtual,
            mod_size: None,
            hash: 0x3b359352,
            rdb_block_offset: 0x8f28,
            original_data_offset: 0x420a0c,
            original_bin_offset: Some(0),
            original_bin_size: Some(0x268005),
            virtual_bin_offset: None,
            rdb_tail_offset: Some(0x8f90),
            original_tail: Some("0@268005#9".to_string()),
            virtual_prefix: None,
        }];

        let enriched = attach_mod_file_sizes(replacements).unwrap();

        assert_eq!(enriched[0].source, ReplacementSource::File(mod_path));
        assert_eq!(enriched[0].mod_size, Some(5));

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn virtual_file_reads_and_tracks_position() {
        let cursor = std::io::Cursor::new(b"abcdef".to_vec());
        let mut file = VirtualFile::new(Box::new(cursor), 6);
        let mut buffer = [0u8; 4];

        assert_eq!(file.size(), 6);
        assert_eq!(file.position(), 0);
        assert_eq!(file.read(&mut buffer).unwrap(), 4);
        assert_eq!(&buffer, b"abcd");
        assert_eq!(file.position(), 4);
    }

    #[test]
    fn virtual_file_stops_at_virtual_size() {
        let cursor = std::io::Cursor::new(b"abcdef".to_vec());
        let mut file = VirtualFile::new(Box::new(cursor), 3);
        let mut buffer = [0u8; 8];

        assert_eq!(file.read(&mut buffer).unwrap(), 3);
        assert_eq!(&buffer[..3], b"abc");
        assert_eq!(file.position(), 3);
        assert_eq!(file.read(&mut buffer).unwrap(), 0);
        assert_eq!(file.position(), 3);
    }

    #[test]
    fn virtual_file_seeks_like_windows_file_pointer() {
        let cursor = std::io::Cursor::new(b"abcdef".to_vec());
        let mut file = VirtualFile::new(Box::new(cursor), 6);
        let mut buffer = [0u8; 2];

        assert_eq!(file.seek(std::io::SeekFrom::Start(2)).unwrap(), 2);
        assert_eq!(file.read(&mut buffer).unwrap(), 2);
        assert_eq!(&buffer, b"cd");
        assert_eq!(file.seek(std::io::SeekFrom::Current(-1)).unwrap(), 3);
        assert_eq!(file.read(&mut buffer).unwrap(), 2);
        assert_eq!(&buffer, b"de");
        assert_eq!(file.seek(std::io::SeekFrom::End(-1)).unwrap(), 5);
        assert_eq!(file.read(&mut buffer).unwrap(), 1);
        assert_eq!(&buffer[..1], b"f");
    }

    #[test]
    fn opens_virtual_replacement_from_disk() {
        let temp_dir =
            std::env::temp_dir().join(format!("oppw4-rdb-tools-open-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mod_path = temp_dir.join("replacement.bin");
        std::fs::write(&mod_path, b"hello").unwrap();

        let replacement = VirtualReplacement {
            archive_name: "Archive".to_string(),
            file_name: "replacement.bin".to_string(),
            source: ReplacementSource::File(mod_path.clone()),
            mode: ReplacementMode::Virtual,
            mod_size: Some(5),
            hash: 0x12345678,
            rdb_block_offset: 0x20,
            original_data_offset: 0x40,
            original_bin_offset: Some(0x100),
            original_bin_size: Some(5),
            virtual_bin_offset: None,
            rdb_tail_offset: None,
            original_tail: None,
            virtual_prefix: None,
        };
        let mut file = open_virtual_replacement(&replacement).unwrap();
        let mut buffer = [0u8; 5];

        assert_eq!(file.size(), 5);
        assert_eq!(file.read(&mut buffer).unwrap(), 5);
        assert_eq!(&buffer, b"hello");

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn virtual_handle_table_opens_reads_seeks_and_closes() {
        let temp_dir = std::env::temp_dir().join(format!(
            "oppw4-rdb-tools-handle-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mod_path = temp_dir.join("replacement.bin");
        std::fs::write(&mod_path, b"abcdef").unwrap();

        let replacement = VirtualReplacement {
            archive_name: "Archive".to_string(),
            file_name: "replacement.bin".to_string(),
            source: ReplacementSource::File(mod_path),
            mode: ReplacementMode::Virtual,
            mod_size: Some(6),
            hash: 0x12345678,
            rdb_block_offset: 0x20,
            original_data_offset: 0x40,
            original_bin_offset: Some(0x100),
            original_bin_size: Some(6),
            virtual_bin_offset: None,
            rdb_tail_offset: None,
            original_tail: None,
            virtual_prefix: None,
        };
        let mut table = VirtualHandleTable::new();
        let handle = table.open(&replacement).unwrap();
        let mut buffer = [0u8; 3];

        assert!(table.contains(handle));
        assert_eq!(table.read(handle, &mut buffer).unwrap(), 3);
        assert_eq!(&buffer, b"abc");
        assert_eq!(table.seek(handle, std::io::SeekFrom::Start(4)).unwrap(), 4);
        assert_eq!(table.read(handle, &mut buffer).unwrap(), 2);
        assert_eq!(&buffer[..2], b"ef");
        assert!(table.close(handle));
        assert!(!table.contains(handle));
        assert!(!table.close(handle));

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn virtual_manager_opens_by_archive_and_hash() {
        let temp_dir = std::env::temp_dir().join(format!(
            "oppw4-rdb-tools-manager-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let mod_path = temp_dir.join("replacement.bin");
        std::fs::write(&mod_path, b"abcdef").unwrap();

        let replacement = VirtualReplacement {
            archive_name: "CharacterEditor".to_string(),
            file_name: "replacement.bin".to_string(),
            source: ReplacementSource::File(mod_path),
            mode: ReplacementMode::Virtual,
            mod_size: Some(6),
            hash: 0x3b359352,
            rdb_block_offset: 0x8f28,
            original_data_offset: 0x420a0c,
            original_bin_offset: Some(0),
            original_bin_size: Some(6),
            virtual_bin_offset: None,
            rdb_tail_offset: Some(0x8f90),
            original_tail: Some("0@268005#9".to_string()),
            virtual_prefix: None,
        };
        let mut manager = VirtualManager::new(vec![replacement]);
        let mut buffer = [0u8; 4];

        assert!(manager
            .open_by_hash("MaterialEditor", 0x3b359352)
            .unwrap()
            .is_none());
        let handle = manager
            .open_by_hash("CharacterEditor", 0x3b359352)
            .unwrap()
            .unwrap();
        assert_eq!(manager.read(handle, &mut buffer).unwrap(), 4);
        assert_eq!(&buffer, b"abcd");
        assert!(manager.close(handle));

        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
