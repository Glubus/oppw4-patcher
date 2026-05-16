use oppw4_data_struct::entry3::CostumeStaticEntry;

use crate::{
    error::Result,
    plan::{Entry3PatchScope, LawSlotInsertPlan},
    validate::{single_law_layout_row, validate_law_slot_patch},
};

pub fn patch_entry3_law_slot(
    mut entry: CostumeStaticEntry,
    plan: &LawSlotInsertPlan,
) -> Result<CostumeStaticEntry> {
    let row = single_law_layout_row(entry.find_layout_rows(plan.owner, plan.preview)?)?;
    validate_law_slot_patch(&row, plan)?;

    entry.set_layout_slot_variant(&row, plan.slot_index, plan.target_variant)?;
    entry.set_layout_active_count(&row, plan.active_count)?;
    if plan.entry3_scope == Entry3PatchScope::LayoutOnly {
        return Ok(entry);
    }

    entry.clone_variant_metadata(plan.source_variant, plan.target_variant)?;
    entry.set_variant_model_resource(plan.target_variant, plan.target_model)?;
    entry.set_variant_preview_mapping(plan.target_variant, plan.preview_mapping)?;

    let source_flags = entry.variant_metadata(plan.source_variant)?.flags;
    entry.set_variant_flags(plan.target_variant, slot_variant_flags(source_flags))?;
    entry.set_variant_load_arg1(plan.target_variant, u16::MAX)?;
    entry.set_variant_load_arg2(plan.target_variant, 0)?;

    Ok(entry)
}

fn slot_variant_flags(source_flags: u8) -> u8 {
    const ENABLED: u8 = 0x01;
    const DLC_ENTITLEMENT: u8 = 0x02;
    const BUILDER_SELECTABLE: u8 = 0x04;
    (source_flags | ENABLED | BUILDER_SELECTABLE) & !DLC_ENTITLEMENT
}

#[cfg(test)]
mod tests {
    use oppw4_data_struct::entry3::{
        CostumeStaticEntry, LAYOUT_ACTIVE_COUNT_OFFSET, LAYOUT_FLAGS_4A_OFFSET,
        LAYOUT_PREVIEW_OFFSET, LAYOUT_TARGET_OFFSET, LAYOUT_VARIANTS_OFFSET,
        VARIANT_METADATA_BASE_OFFSET, VARIANT_METADATA_COLOR_OFFSET, VARIANT_METADATA_FLAGS_OFFSET,
        VARIANT_METADATA_MODEL_OFFSET, VARIANT_METADATA_PREVIEW_OFFSET, VARIANT_METADATA_STRIDE,
    };

    use super::*;

    const LAW_ROW_OFFSET: usize = 0x80;

    #[test]
    fn patches_only_law_layout_by_default() {
        let patched = patch_entry3_law_slot(
            CostumeStaticEntry::parse(fixture_entry3()),
            &LawSlotInsertPlan::law_slot5_layout_only(),
        )
        .unwrap();

        let row = patched.find_layout_rows(26, 26).unwrap().remove(0);
        let metadata = patched.variant_metadata(699).unwrap();
        assert_eq!(row.variants[4], 699);
        assert_eq!(row.active_count, 5);
        assert_eq!(metadata.model_resource, 0xffff);
        assert_eq!(metadata.preview_mapping, 0xffff);
        assert_eq!(metadata.flags, 0xff);
    }

    #[test]
    fn patches_law_slot_and_variant_metadata_when_requested() {
        let patched = patch_entry3_law_slot(
            CostumeStaticEntry::parse(fixture_entry3()),
            &LawSlotInsertPlan::law_slot5_from_oni(),
        )
        .unwrap();

        let row = patched.find_layout_rows(26, 26).unwrap().remove(0);
        let metadata = patched.variant_metadata(699).unwrap();
        assert_eq!(row.variants[4], 699);
        assert_eq!(row.active_count, 5);
        assert_eq!(metadata.model_resource, 730);
        assert_eq!(metadata.preview_mapping, 294);
        assert_eq!(metadata.flags & 0x01, 0x01);
        assert_eq!(metadata.flags & 0x02, 0x00);
        assert_eq!(metadata.flags & 0x04, 0x04);
        assert_eq!(
            &metadata.raw[VARIANT_METADATA_COLOR_OFFSET..VARIANT_METADATA_COLOR_OFFSET + 2],
            &[0xff, 0xff]
        );
        assert_eq!(metadata.raw[0x1a], 0x00);
    }

    fn fixture_entry3() -> Vec<u8> {
        let mut bytes = vec![0xff; VARIANT_METADATA_BASE_OFFSET + 700 * VARIANT_METADATA_STRIDE];
        put_u16(&mut bytes, LAW_ROW_OFFSET + LAYOUT_TARGET_OFFSET, 26);
        put_u16(&mut bytes, LAW_ROW_OFFSET + LAYOUT_PREVIEW_OFFSET, 26);
        for (index, variant) in [57u16, 58, 555, 586].into_iter().enumerate() {
            put_u16(
                &mut bytes,
                LAW_ROW_OFFSET + LAYOUT_VARIANTS_OFFSET + index * 2,
                variant,
            );
        }
        bytes[LAW_ROW_OFFSET + LAYOUT_ACTIVE_COUNT_OFFSET] = 4;
        bytes[LAW_ROW_OFFSET + LAYOUT_FLAGS_4A_OFFSET] = 1;

        let source = VARIANT_METADATA_BASE_OFFSET + 57 * VARIANT_METADATA_STRIDE;
        put_u16(&mut bytes, source + VARIANT_METADATA_MODEL_OFFSET, 26);
        put_u16(&mut bytes, source + VARIANT_METADATA_PREVIEW_OFFSET, 26);
        put_u16(&mut bytes, source + VARIANT_METADATA_COLOR_OFFSET, 5);
        bytes[source + VARIANT_METADATA_FLAGS_OFFSET] = 0x03;
        bytes[source + 0x1a] = 12;
        bytes
    }

    fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
}
