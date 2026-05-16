use std::collections::HashMap;

use crate::log;

use super::types::{Handle, INVALID_HANDLE_VALUE};

#[derive(Debug, Default)]
pub(crate) struct RdbTracker {
    handles: HashMap<usize, TrackedRdb>,
    total_logs: usize,
}

#[derive(Debug)]
struct TrackedRdb {
    archive_name: String,
    kind: TrackedFileKind,
    read_logs: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrackedFileKind {
    Index,
    Data,
}

impl RdbTracker {
    pub(crate) fn track_open(&mut self, handle: Handle, file_name: &str) {
        if handle == INVALID_HANDLE_VALUE {
            return;
        }
        let Some((archive_name, kind)) = tracked_archive_name(file_name) else {
            return;
        };
        self.handles.insert(
            handle as usize,
            TrackedRdb {
                archive_name,
                kind,
                read_logs: 0,
            },
        );
        log::write_line(format!(
            "Track {} {}: handle=0x{:x}",
            kind.label(),
            self.handles
                .get(&(handle as usize))
                .map(|tracked| tracked.archive_name.as_str())
                .unwrap_or("?"),
            handle as usize
        ));
    }

    pub(crate) fn untrack(&mut self, handle: Handle) {
        if let Some(tracked) = self.handles.remove(&(handle as usize)) {
            log::write_line(format!(
                "Close {} {}",
                tracked.kind.label(),
                tracked.archive_name
            ));
        }
    }

    pub(crate) fn read_event(&mut self, handle: Handle) -> Option<(TrackedRead, bool)> {
        let tracked = self.handles.get_mut(&(handle as usize))?;
        let should_log = tracked.read_logs < 64 && self.total_logs < 640;
        if should_log {
            tracked.read_logs += 1;
            self.total_logs += 1;
        }
        Some((
            TrackedRead {
                archive_name: tracked.archive_name.clone(),
                kind: tracked.kind,
            },
            should_log,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrackedRead {
    pub(crate) archive_name: String,
    pub(crate) kind: TrackedFileKind,
}

impl TrackedFileKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            TrackedFileKind::Index => "RDB",
            TrackedFileKind::Data => "RDB BIN",
        }
    }
}

pub(crate) fn tracked_archive_name(file_name: &str) -> Option<(String, TrackedFileKind)> {
    let lower = file_name.to_ascii_lowercase();
    if let Some(index) = lower.find(".rdb.bin") {
        if lower[index + ".rdb.bin".len()..]
            .chars()
            .all(|character| character.is_ascii_digit())
        {
            return Some((file_name[..index].to_string(), TrackedFileKind::Data));
        }
    }
    if lower.ends_with(".rdb.bin") {
        return Some((
            file_name[..file_name.len() - ".rdb.bin".len()].to_string(),
            TrackedFileKind::Data,
        ));
    }
    if lower.ends_with(".rdb") {
        return Some((
            file_name[..file_name.len() - ".rdb".len()].to_string(),
            TrackedFileKind::Index,
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_rdb_and_rdb_bin_file_names() {
        assert_eq!(
            tracked_archive_name("CharacterEditor.rdb"),
            Some(("CharacterEditor".to_string(), TrackedFileKind::Index))
        );
        assert_eq!(
            tracked_archive_name("MaterialEditor.rdb.bin"),
            Some(("MaterialEditor".to_string(), TrackedFileKind::Data))
        );
        assert_eq!(
            tracked_archive_name("ScreenLayout.rdb.bin10"),
            Some(("ScreenLayout".to_string(), TrackedFileKind::Data))
        );
        assert_eq!(tracked_archive_name("not-an-archive.bin"), None);
    }
}
