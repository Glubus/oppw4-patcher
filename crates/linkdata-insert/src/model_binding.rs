use oppw4_data_struct::{entry32::Entry32, entry35::ModelEntry};

use crate::Result;

pub const PRIVATE_MODEL_NAME: &str = "MDLC999_Law_Custom_730";

pub fn patch_model_binding(
    mut entry32: Entry32,
    mut entry35: ModelEntry,
    source_model: u16,
    target_model: u16,
) -> Result<(Entry32, ModelEntry)> {
    entry32.set_name(6, usize::from(target_model), PRIVATE_MODEL_NAME)?;
    entry35.clone_row(source_model, target_model)?;
    Ok((entry32, entry35))
}

#[cfg(test)]
mod tests {
    use oppw4_data_struct::{
        entry32::Entry32,
        entry35::{ModelEntry, MODEL_ROW_BASE, MODEL_ROW_OWNER_I16_INDEX, MODEL_ROW_SIZE},
    };

    use super::*;

    #[test]
    fn patches_model_name_and_clones_model_row() {
        let entry32 = Entry32::parse(&entry32_with_section6(&[
            (308, "MDLC069_Law_Oni"),
            (730, ""),
        ]))
        .unwrap();
        let entry35 = ModelEntry::parse(model_entry_with_rows(&[(308, 26), (730, -1)]));

        let (entry32, entry35) = patch_model_binding(entry32, entry35, 308, 730).unwrap();

        assert_eq!(entry32.name(6, 730), Some("MDLC999_Law_Custom_730"));
        assert_eq!(entry35.row(730).unwrap().raw, entry35.row(308).unwrap().raw);
    }

    fn entry32_with_section6(entries: &[(usize, &str)]) -> Vec<u8> {
        let max_id = entries.iter().map(|(id, _)| *id).max().unwrap_or(0);
        let mut sections = vec![Vec::new(); 7];
        sections[6] = entry32_section(max_id + 1, entries);
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

    fn model_entry_with_rows(rows: &[(u16, i16)]) -> Vec<u8> {
        let max_row = rows.iter().map(|(row, _)| *row).max().unwrap_or(0);
        let mut bytes = vec![0xff; MODEL_ROW_BASE + (usize::from(max_row) + 1) * MODEL_ROW_SIZE];
        bytes[0..4].copy_from_slice(&(u32::from(max_row) + 1).to_le_bytes());
        for (row, owner) in rows {
            let offset = MODEL_ROW_BASE + usize::from(*row) * MODEL_ROW_SIZE;
            let owner_offset = offset + MODEL_ROW_OWNER_I16_INDEX * 2;
            bytes[owner_offset..owner_offset + 2].copy_from_slice(&owner.to_le_bytes());
        }
        bytes
    }
}
