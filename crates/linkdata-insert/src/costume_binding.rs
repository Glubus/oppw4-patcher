use oppw4_data_struct::{
    entry29::DlcEntry, entry32::Entry32, entry39::CostumeParamEntry, entry52::CostumeRecordEntry,
    entry58::CostumeSectionEntry,
};

use crate::error::Result;

pub const PRIVATE_COSTUME_NAME: &str = "806_699_costume_law_custom";

pub fn patch_costume_binding(
    mut entry29: DlcEntry,
    mut entry32: Entry32,
    mut entry39: CostumeParamEntry,
    mut entry52: CostumeRecordEntry,
    mut entry58: CostumeSectionEntry,
    owner: u16,
    source_suffix: u16,
    target_suffix: u16,
) -> Result<(
    DlcEntry,
    Entry32,
    CostumeParamEntry,
    CostumeRecordEntry,
    CostumeSectionEntry,
)> {
    if entry29.find_costume_codes(owner, target_suffix).is_empty() {
        entry29.append_costume_code(&format!(
            "DLC_COSTUME_007_{target_suffix:03}_{owner:03}_004"
        ));
    }

    let costume_name = find_costume_name_by_suffix(&entry32, source_suffix)
        .map(|name| retarget_costume_name(name, target_suffix))
        .unwrap_or_else(|| PRIVATE_COSTUME_NAME.to_string());
    entry32.set_name(7, usize::from(target_suffix), costume_name)?;

    if entry52
        .records_for_owner_layout(u32::from(owner), u32::from(target_suffix))?
        .is_empty()
    {
        entry52.append_record([u32::from(target_suffix), u32::from(owner), 0, 0, 0, 0, 0, 0])?;
    }

    entry58.clone_section(source_suffix, target_suffix)?;
    entry39.clone_row(source_suffix, target_suffix)?;

    Ok((entry29, entry32, entry39, entry52, entry58))
}

fn find_costume_name_by_suffix(entry32: &Entry32, suffix: u16) -> Option<&str> {
    let prefix = format!("806_{suffix:03}_costume");
    entry32
        .section(7)?
        .strings
        .iter()
        .find(|item| item.value.starts_with(&prefix))
        .map(|item| item.value.as_str())
}

fn retarget_costume_name(source_name: &str, target_suffix: u16) -> String {
    let Some(rest) = source_name.strip_prefix("806_") else {
        return PRIVATE_COSTUME_NAME.to_string();
    };
    let Some((_old_suffix, tail)) = rest.split_once("_costume") else {
        return PRIVATE_COSTUME_NAME.to_string();
    };

    format!("806_{target_suffix:03}_costume{tail}")
}

#[cfg(test)]
mod tests {
    use oppw4_data_struct::{
        entry29::DlcEntry,
        entry32::Entry32,
        entry39::CostumeParamEntry,
        entry52::{CostumeRecordEntry, COSTUME_RECORD_HEADER_SIZE},
        entry58::CostumeSectionEntry,
    };

    use super::*;

    #[test]
    fn patches_costume_name_dlc_record_and_section() {
        let entry29 = DlcEntry::parse(b"\0DLC_COSTUME_006_586_026_003\0".to_vec());
        let entry32 = Entry32::parse(&entry32_with_section7(&[
            (699, ""),
            (131, "806_131_costume_law_oni"),
            (2179, "806_131_costume_law_oni"),
        ]))
        .unwrap();
        let entry39 = CostumeParamEntry::parse(entry39_with_row(131));
        let entry52 =
            CostumeRecordEntry::parse(entry52_with_records(&[[131, 26, 0, 0, 0, 0, 0, 0]]));
        let entry58 = CostumeSectionEntry::parse(entry58_with_section(131));

        let (entry29, entry32, entry39, entry52, entry58) =
            patch_costume_binding(entry29, entry32, entry39, entry52, entry58, 26, 131, 699)
                .unwrap();

        assert_eq!(entry29.find_costume_codes(26, 699).len(), 1);
        assert_eq!(entry32.name(7, 699), Some("806_699_costume_law_oni"));
        assert!(entry39.row(699).unwrap().present);
        assert_eq!(
            entry52.records_for_owner_layout(26, 699).unwrap()[0].fields,
            [699, 26, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            entry58.section(699).unwrap().unwrap().records[0].fields,
            [8, 7, 0, 0]
        );
    }

    #[test]
    fn retargets_costume_name_suffix_while_preserving_tail() {
        assert_eq!(
            retarget_costume_name("806_131_costume_law_oni", 699),
            "806_699_costume_law_oni"
        );
    }

    fn entry32_with_section7(entries: &[(usize, &str)]) -> Vec<u8> {
        let max_id = entries.iter().map(|(id, _)| *id).max().unwrap_or(0);
        let mut sections = vec![Vec::new(); 8];
        sections[7] = entry32_section(max_id + 1, entries);
        entry32_bytes(&sections)
    }

    fn entry32_bytes(sections: &[Vec<u8>]) -> Vec<u8> {
        let header_len = 4 + sections.len() * 4;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(sections.len() as u32).to_le_bytes());
        let mut cursor = header_len;
        for section in sections {
            if section.is_empty() {
                bytes.extend_from_slice(&0u32.to_le_bytes());
            } else {
                bytes.extend_from_slice(&(cursor as u32).to_le_bytes());
                cursor += section.len();
            }
        }
        for section in sections {
            bytes.extend_from_slice(section);
        }
        bytes
    }

    fn entry32_section(count: usize, entries: &[(usize, &str)]) -> Vec<u8> {
        let table_len = 4 + count * 8;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(count as u32).to_le_bytes());
        let mut payload = Vec::new();
        for id in 0..count {
            let value = entries
                .iter()
                .find(|(entry_id, _)| *entry_id == id)
                .map(|(_, value)| *value)
                .unwrap_or("");
            let encoded = value
                .as_bytes()
                .iter()
                .copied()
                .chain(std::iter::once(0))
                .collect::<Vec<_>>();
            let relative_offset = table_len + payload.len();
            bytes.extend_from_slice(&(relative_offset as u32).to_le_bytes());
            bytes.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
            payload.extend_from_slice(&encoded);
        }
        bytes.extend_from_slice(&payload);
        bytes
    }

    fn entry52_with_records(records: &[[u32; 8]]) -> Vec<u8> {
        let mut bytes = vec![0; COSTUME_RECORD_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&(records.len() as u32).to_le_bytes());
        for record in records {
            for value in record {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes
    }

    fn entry39_with_row(row: u16) -> Vec<u8> {
        const BASE: usize = 0x10;
        const ROW_SIZE: usize = 0x30;
        let count = usize::from(699u16.max(row)) + 1;
        let mut bytes = vec![0xff; BASE + count * ROW_SIZE];
        bytes[0..4].copy_from_slice(&(count as u32).to_le_bytes());
        let offset = BASE + usize::from(row) * ROW_SIZE;
        bytes[offset + 4..offset + 6].copy_from_slice(&23378i16.to_le_bytes());
        bytes
    }

    fn entry58_with_section(suffix: u16) -> Vec<u8> {
        let section_count = usize::from(699u16.max(suffix)) + 2;
        let mut bytes = vec![0; 4 + section_count * 4];
        bytes[0..4].copy_from_slice(&(section_count as u32).to_le_bytes());
        let section_offset = bytes.len();
        let offset_slot = 4 + usize::from(suffix) * 4;
        bytes[offset_slot..offset_slot + 4].copy_from_slice(&(section_offset as u32).to_le_bytes());
        for value in [1u32, 0, 0, 0, 8, 7, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}
