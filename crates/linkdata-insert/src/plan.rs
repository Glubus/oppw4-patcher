#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchMode {
    InPlace,
    RebuildCompressed,
    RebuildRaw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry3PatchScope {
    LayoutOnly,
    LayoutAndVariantMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawSlotInsertPlan {
    pub owner: u16,
    pub preview: u16,
    pub slot_index: usize,
    pub source_variant: u16,
    pub target_variant: u16,
    pub source_costume_suffix: u16,
    pub target_costume_suffix: u16,
    pub source_model: u16,
    pub target_model: u16,
    pub preview_mapping: u16,
    pub active_count: u8,
    pub mode: PatchMode,
    pub entry3_scope: Entry3PatchScope,
    pub linked_costume_binding: bool,
    pub entry17_record_binding: bool,
}

impl LawSlotInsertPlan {
    pub fn law_slot5_layout_only() -> Self {
        Self {
            owner: 26,
            preview: 26,
            slot_index: 4,
            source_variant: 57,
            target_variant: 699,
            source_costume_suffix: 131,
            target_costume_suffix: 699,
            source_model: 26,
            target_model: 730,
            preview_mapping: 294,
            active_count: 5,
            mode: PatchMode::InPlace,
            entry3_scope: Entry3PatchScope::LayoutOnly,
            linked_costume_binding: false,
            entry17_record_binding: false,
        }
    }

    pub fn law_slot5_from_oni() -> Self {
        Self {
            entry3_scope: Entry3PatchScope::LayoutAndVariantMetadata,
            ..Self::law_slot5_layout_only()
        }
    }

    pub fn law_slot5_clone_base() -> Self {
        Self {
            source_variant: 57,
            source_costume_suffix: 57,
            source_model: 252,
            target_model: 252,
            preview_mapping: u16::MAX,
            entry3_scope: Entry3PatchScope::LayoutAndVariantMetadata,
            linked_costume_binding: true,
            entry17_record_binding: false,
            ..Self::law_slot5_layout_only()
        }
    }
}
