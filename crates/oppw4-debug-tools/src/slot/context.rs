// crates/oppw4-debug-tools/src/slot/context.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LawSlot5Context {
    pub selected_variant: u16,
    pub selected_slot: u16,
    pub model_resource: u16,
    pub mapped_preview: u16,
}

impl LawSlot5Context {
    pub fn is_target_slot5(self) -> bool {
        self.selected_variant == 699
            && self.selected_slot == 4
            && self.model_resource == 292
            && self.mapped_preview == 911
    }
}

#[cfg(test)]
mod tests {
    use crate::slot::LawSlot5Context;

    #[test]
    fn target_context_requires_all_known_slot5_guards() {
        assert!(LawSlot5Context {
            selected_variant: 699,
            selected_slot: 4,
            model_resource: 292,
            mapped_preview: 911,
        }
        .is_target_slot5());
        assert!(!LawSlot5Context {
            selected_variant: 586,
            selected_slot: 3,
            model_resource: 308,
            mapped_preview: 911,
        }
        .is_target_slot5());
    }
}
