use oppw4_data_struct::{
    entry3::CostumeStaticEntry,
    entry32::Entry32,
    entry35::{ModelEntry, ModelRow},
};

use crate::{
    asset_package::law_slot5_base_law_kids_package,
    error::Result,
    model_binding::PRIVATE_MODEL_NAME,
    plan::LawSlotInsertPlan,
    validate::{single_law_layout_row, validate_law_slot_patch},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTargetStatus {
    Ready,
    Available,
    OccupiedByOther,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightReport {
    pub owner: u16,
    pub target_variant: u16,
    pub source_variant: u16,
    pub source_model: u16,
    pub target_model: u16,
    pub source_model_name: Option<String>,
    pub target_model_name: Option<String>,
    pub target_model_status: ModelTargetStatus,
    pub layout_slot_patchable: bool,
    pub source_model_row: Option<ModelRow>,
    pub target_model_row: Option<ModelRow>,
    pub recommended_model_target: Option<u16>,
    pub asset_count: usize,
    pub texture_count: usize,
    pub requires_private_route: bool,
}

pub fn preflight_law_slot5(linkdata: &[u8], plan: &LawSlotInsertPlan) -> Result<PreflightReport> {
    let package = law_slot5_base_law_kids_package();
    let entry3 = CostumeStaticEntry::from_linkdata_bytes(linkdata)?;
    let layout_row = single_law_layout_row(entry3.find_layout_rows(plan.owner, plan.preview)?)?;
    let layout_slot_patchable = validate_law_slot_patch(&layout_row, plan).is_ok();

    let entry32 = Entry32::from_linkdata_bytes(linkdata)?;
    let source_model_name = entry32
        .name(6, usize::from(plan.source_model))
        .map(str::to_string);
    let target_model_name = entry32
        .name(6, usize::from(plan.target_model))
        .map(str::to_string);
    let target_model_status =
        classify_model_target_name(target_model_name.as_deref(), PRIVATE_MODEL_NAME);

    let entry35 = ModelEntry::from_linkdata_bytes(linkdata)?;
    let source_model_row = entry35.row(plan.source_model).ok();
    let target_model_row = entry35.row(plan.target_model).ok();
    let recommended_model_target = match target_model_status {
        ModelTargetStatus::Ready | ModelTargetStatus::Available => Some(plan.target_model),
        ModelTargetStatus::OccupiedByOther => {
            find_available_model_target(&entry32, &entry35, 0, 730)
        }
    };
    let texture_count = package
        .assets
        .iter()
        .filter(|asset| asset.archive == crate::Archive::MaterialEditor)
        .count();

    Ok(PreflightReport {
        owner: plan.owner,
        target_variant: plan.target_variant,
        source_variant: plan.source_variant,
        source_model: plan.source_model,
        target_model: plan.target_model,
        source_model_name,
        target_model_name,
        target_model_status,
        layout_slot_patchable,
        source_model_row,
        target_model_row,
        recommended_model_target,
        asset_count: package.assets.len(),
        texture_count,
        requires_private_route: package.requires_private_route,
    })
}

pub fn find_available_model_target(
    entry32: &Entry32,
    entry35: &ModelEntry,
    start: u16,
    end: u16,
) -> Option<u16> {
    let section = entry32.section(6)?;
    (start..=end).find(|model_id| {
        let name_available = section
            .strings
            .get(usize::from(*model_id))
            .is_none_or(|item| item.value.is_empty());
        let row_available = entry35
            .row(*model_id)
            .map(|row| row.owner < 0)
            .unwrap_or(true);
        name_available && row_available
    })
}

pub fn classify_model_target_name(
    current_name: Option<&str>,
    desired_name: &str,
) -> ModelTargetStatus {
    match current_name {
        None | Some("") => ModelTargetStatus::Available,
        Some(current) if current.eq_ignore_ascii_case(desired_name) => ModelTargetStatus::Ready,
        Some(_) => ModelTargetStatus::OccupiedByOther,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_existing_foreign_model_name_as_occupied() {
        assert_eq!(
            classify_model_target_name(Some("MPLC000_Luffy"), "MDLC999_Law_Custom"),
            ModelTargetStatus::OccupiedByOther
        );
    }

    #[test]
    fn classifies_matching_model_name_as_ready() {
        assert_eq!(
            classify_model_target_name(Some("MDLC999_Law_Custom"), "MDLC999_Law_Custom"),
            ModelTargetStatus::Ready
        );
    }

    #[test]
    fn classifies_missing_or_empty_model_name_as_available() {
        assert_eq!(
            classify_model_target_name(None, "MDLC999_Law_Custom"),
            ModelTargetStatus::Available
        );
        assert_eq!(
            classify_model_target_name(Some(""), "MDLC999_Law_Custom"),
            ModelTargetStatus::Available
        );
    }

    #[test]
    fn finds_empty_model_name_with_free_model_row() {
        let entry32 = Entry32::parse(&entry32_with_section6(&[
            (700, "Occupied"),
            (701, ""),
            (702, ""),
        ]))
        .unwrap();
        let entry35 = ModelEntry::parse(model_entry_with_rows(&[(700, 26), (701, 26), (702, -1)]));

        assert_eq!(
            find_available_model_target(&entry32, &entry35, 700, 702),
            Some(702)
        );
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
        use oppw4_data_struct::entry35::{
            MODEL_ROW_BASE, MODEL_ROW_OWNER_I16_INDEX, MODEL_ROW_SIZE,
        };
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
