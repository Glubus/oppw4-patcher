use oppw4_data_struct::entry3::{CostumeLayoutRow, EMPTY_LAYOUT_VARIANT_ID};

use crate::{
    error::{InsertError, Result},
    plan::LawSlotInsertPlan,
};

pub fn single_law_layout_row(rows: Vec<CostumeLayoutRow>) -> Result<CostumeLayoutRow> {
    match rows.len() {
        0 => Err(InsertError::NoLawLayoutRow),
        1 => Ok(rows.into_iter().next().expect("one row")),
        count => Err(InsertError::AmbiguousLawLayoutRows { count }),
    }
}

pub fn validate_law_slot_patch(row: &CostumeLayoutRow, plan: &LawSlotInsertPlan) -> Result<()> {
    let current_slot =
        row.variants
            .get(plan.slot_index)
            .copied()
            .ok_or(InsertError::SlotNotPatchable {
                slot_index: plan.slot_index,
                found: u16::MAX,
            })?;
    if current_slot != EMPTY_LAYOUT_VARIANT_ID && current_slot != plan.target_variant {
        return Err(InsertError::SlotNotPatchable {
            slot_index: plan.slot_index,
            found: current_slot,
        });
    }

    let official_count = plan.slot_index as u8;
    if row.active_count != official_count && row.active_count != plan.active_count {
        return Err(InsertError::ActiveCountNotPatchable {
            found: row.active_count,
        });
    }

    Ok(())
}
