use oppw4_data_struct::{
    entry17::{Entry17, LINKDATA_ENTRY_INDEX as ENTRY17},
    entry3::{CostumeStaticEntry, LINKDATA_ENTRY_INDEX as ENTRY3},
    entry32::{Entry32, LINKDATA_ENTRY_INDEX as ENTRY32},
    entry35::{ModelEntry, LINKDATA_ENTRY_INDEX as ENTRY35},
};

use crate::{
    costume_binding::patch_costume_binding,
    entry17_binding::patch_entry17_law_slot,
    entry3_insert::patch_entry3_law_slot,
    error::Result,
    model_binding::patch_model_binding,
    plan::{LawSlotInsertPlan, PatchMode},
};

pub fn insert_law_slot(linkdata: &[u8], plan: &LawSlotInsertPlan) -> Result<Vec<u8>> {
    let entry3 = CostumeStaticEntry::from_linkdata_bytes(linkdata)?;
    let patched_entry3 = patch_entry3_law_slot(entry3, plan)?.into_bytes();
    let entry32 = Entry32::from_linkdata_bytes(linkdata)?;
    let entry35 = ModelEntry::from_linkdata_bytes(linkdata)?;
    let (mut patched_entry32, patched_entry35) = if plan.source_model == plan.target_model {
        (entry32, entry35)
    } else {
        patch_model_binding(entry32, entry35, plan.source_model, plan.target_model)?
    };
    let mut edits = vec![(ENTRY3, patched_entry3)];
    if plan.linked_costume_binding {
        use oppw4_data_struct::{
            entry29::{DlcEntry, LINKDATA_ENTRY_INDEX as ENTRY29},
            entry39::{CostumeParamEntry, LINKDATA_ENTRY_INDEX as ENTRY39},
            entry52::{CostumeRecordEntry, LINKDATA_ENTRY_INDEX as ENTRY52},
            entry58::{CostumeSectionEntry, LINKDATA_ENTRY_INDEX as ENTRY58},
        };

        let entry29 = DlcEntry::from_linkdata_bytes(linkdata)?;
        let entry39 = CostumeParamEntry::from_linkdata_bytes(linkdata)?;
        let entry52 = CostumeRecordEntry::from_linkdata_bytes(linkdata)?;
        let entry58 = CostumeSectionEntry::from_linkdata_bytes(linkdata)?;
        let (
            patched_entry29,
            patched_entry32_with_costume_binding,
            patched_entry39,
            patched_entry52,
            patched_entry58,
        ) = patch_costume_binding(
            entry29,
            patched_entry32,
            entry39,
            entry52,
            entry58,
            plan.owner,
            plan.source_costume_suffix,
            plan.target_costume_suffix,
        )?;
        edits.push((ENTRY29, patched_entry29.into_bytes()));
        edits.push((ENTRY39, patched_entry39.into_bytes()));
        edits.push((ENTRY52, patched_entry52.into_bytes()));
        edits.push((ENTRY58, patched_entry58.into_bytes()));
        patched_entry32 = patched_entry32_with_costume_binding;
    }
    if plan.source_model != plan.target_model || plan.linked_costume_binding {
        edits.push((ENTRY32, patched_entry32.into_bytes()));
    }
    if plan.source_model != plan.target_model {
        edits.push((ENTRY35, patched_entry35.into_bytes()));
    }
    if plan.entry17_record_binding {
        let entry17 = Entry17::from_linkdata_bytes(linkdata)?;
        let patched_entry17 = patch_entry17_law_slot(entry17, plan)?;
        edits.push((ENTRY17, patched_entry17.into_bytes()));
    }

    let output = match plan.mode {
        PatchMode::InPlace => oppw4_rdb::patch_linkdata_entries_in_place(linkdata, edits)?,
        PatchMode::RebuildCompressed => oppw4_rdb::rebuild_linkdata_with_edits(linkdata, edits)?,
        PatchMode::RebuildRaw => oppw4_rdb::rebuild_linkdata_raw_with_edits(linkdata, edits)?,
    };

    Ok(output)
}
