use oppw4_data_struct::entry17::Entry17;

use crate::{
    error::{InsertError, Result},
    plan::LawSlotInsertPlan,
};

pub fn patch_entry17_record_binding(
    mut entry: Entry17,
    source_id: u32,
    target_id: u32,
) -> Result<Entry17> {
    let patched = entry.clone_record_candidate(source_id, target_id)?;
    if !patched {
        return Err(InsertError::Entry17RecordBindingNotPatchable {
            source_id,
            target_id,
        });
    }
    Ok(entry)
}

pub fn patch_entry17_law_slot(entry: Entry17, plan: &LawSlotInsertPlan) -> Result<Entry17> {
    const LAW_ONI_VARIANT: u32 = 586;
    patch_entry17_record_binding(entry, LAW_ONI_VARIANT, u32::from(plan.target_variant))
}

#[cfg(test)]
mod tests {
    use oppw4_data_struct::entry17::{Entry17, CANDIDATE_RECORD_ID_OFFSET, CANDIDATE_RECORD_SIZE};

    use super::*;

    #[test]
    fn patches_entry17_by_cloning_source_record_to_target_id() {
        let entry = Entry17::parse(fixture_entry17());

        let patched = patch_entry17_record_binding(entry, 586, 699).unwrap();

        let target = patched.record_candidates_for_id(699).unwrap().remove(0);
        assert_eq!(target.offset, 0x60);
        assert_eq!(target.fields[0], 30);
        assert_eq!(target.fields[4], 5);
        assert_eq!(target.fields[8], 699);
        assert_eq!(target.fields[10], 25);
    }

    fn fixture_entry17() -> Vec<u8> {
        let mut bytes = vec![0xff; 0x60 + CANDIDATE_RECORD_SIZE];
        put_record(
            &mut bytes,
            0x10,
            &[
                30,
                u32::MAX,
                u32::MAX,
                27,
                5,
                65793,
                4294967042,
                769,
                586,
                5,
                25,
            ],
        );
        put_record(
            &mut bytes,
            0x60,
            &[
                u32::MAX,
                u32::MAX,
                u32::MAX,
                0,
                0,
                255,
                u32::MAX,
                256,
                699,
                6,
                36,
            ],
        );
        bytes
    }

    fn put_record(bytes: &mut [u8], offset: usize, fields: &[u32]) {
        for (index, field) in fields.iter().enumerate() {
            let field_offset = offset + index * 4;
            bytes[field_offset..field_offset + 4].copy_from_slice(&field.to_le_bytes());
        }
        assert_eq!(
            u32::from_le_bytes(
                bytes[offset + CANDIDATE_RECORD_ID_OFFSET..offset + CANDIDATE_RECORD_ID_OFFSET + 4]
                    .try_into()
                    .unwrap()
            ),
            fields[8]
        );
    }
}
