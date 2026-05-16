// crates/oppw4-debug-tools/src/rdb_io/archive_name.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackedArchiveKind {
    Index,
    Data,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedArchiveMatch {
    pub archive_name: String,
    pub kind: TrackedArchiveKind,
}

impl TrackedArchiveKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Index => "RDB",
            Self::Data => "RDB BIN",
        }
    }
}

pub fn classify_archive_file_name(file_name: &str) -> Option<TrackedArchiveMatch> {
    let lower = file_name.to_ascii_lowercase();
    if let Some(index) = lower.find(".rdb.bin") {
        let suffix = lower[index + ".rdb.bin".len()..].trim_start_matches('_');
        if !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit()) {
            return Some(TrackedArchiveMatch {
                archive_name: file_name[..index].to_string(),
                kind: TrackedArchiveKind::Data,
            });
        }
    }
    if lower.ends_with(".rdb.bin") {
        return Some(TrackedArchiveMatch {
            archive_name: file_name[..file_name.len() - ".rdb.bin".len()].to_string(),
            kind: TrackedArchiveKind::Data,
        });
    }
    lower.ends_with(".rdb").then(|| TrackedArchiveMatch {
        archive_name: file_name[..file_name.len() - ".rdb".len()].to_string(),
        kind: TrackedArchiveKind::Index,
    })
}

#[cfg(test)]
mod tests {
    use crate::rdb_io::{classify_archive_file_name, TrackedArchiveKind};

    #[test]
    fn classifies_rdb_index_and_bin_names() {
        let index = classify_archive_file_name("CharacterEditor.rdb").unwrap();
        let data = classify_archive_file_name("ScreenLayout.rdb.bin10").unwrap();
        let underscored_data = classify_archive_file_name("CharacterEditor.rdb.bin_120").unwrap();

        assert_eq!(index.archive_name, "CharacterEditor");
        assert_eq!(index.kind, TrackedArchiveKind::Index);
        assert_eq!(data.archive_name, "ScreenLayout");
        assert_eq!(data.kind, TrackedArchiveKind::Data);
        assert_eq!(underscored_data.archive_name, "CharacterEditor");
        assert_eq!(underscored_data.kind, TrackedArchiveKind::Data);
        assert!(classify_archive_file_name("not-an-archive.bin").is_none());
    }
}
