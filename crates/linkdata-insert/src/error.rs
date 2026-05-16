#[derive(Debug)]
pub enum InsertError {
    DataStruct(oppw4_data_struct::DataStructError),
    LinkData(oppw4_rdb::LinkDataError),
    NoLawLayoutRow,
    AmbiguousLawLayoutRows { count: usize },
    SlotNotPatchable { slot_index: usize, found: u16 },
    ActiveCountNotPatchable { found: u8 },
    Entry17RecordBindingNotPatchable { source_id: u32, target_id: u32 },
}

impl std::fmt::Display for InsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataStruct(error) => write!(f, "{error}"),
            Self::LinkData(error) => write!(f, "{error:?}"),
            Self::NoLawLayoutRow => write!(f, "Law layout row not found"),
            Self::AmbiguousLawLayoutRows { count } => {
                write!(f, "expected one Law layout row, found {count}")
            }
            Self::SlotNotPatchable { slot_index, found } => {
                write!(f, "slot {slot_index} is not patchable: found {found}")
            }
            Self::ActiveCountNotPatchable { found } => {
                write!(f, "active_count is not patchable: found {found}")
            }
            Self::Entry17RecordBindingNotPatchable {
                source_id,
                target_id,
            } => write!(
                f,
                "entry17 record binding is not patchable: source={source_id} target={target_id}"
            ),
        }
    }
}

impl std::error::Error for InsertError {}

impl From<oppw4_data_struct::DataStructError> for InsertError {
    fn from(error: oppw4_data_struct::DataStructError) -> Self {
        Self::DataStruct(error)
    }
}

impl From<oppw4_rdb::LinkDataError> for InsertError {
    fn from(error: oppw4_rdb::LinkDataError) -> Self {
        Self::LinkData(error)
    }
}

pub type Result<T> = std::result::Result<T, InsertError>;
