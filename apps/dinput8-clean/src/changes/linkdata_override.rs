use std::path::Path;

use oppw4_rdb::{ReplacementMode, ReplacementSource, VirtualReplacement};

use crate::log;

#[allow(dead_code)]
pub fn load_linkdata_a(config_root: &Path) -> Vec<VirtualReplacement> {
    let override_path = config_root.join("linkdata_override").join("LINKDATA_A.BIN");
    let Ok(metadata) = std::fs::metadata(&override_path) else {
        log::write_line(format!(
            "LinkData override disabled path={} reason=missing",
            override_path.display()
        ));
        return Vec::new();
    };

    log::write_line(format!(
        "LinkData override enabled path={} size=0x{:x}",
        override_path.display(),
        metadata.len()
    ));
    vec![VirtualReplacement {
        archive_name: "LINKDATA_A.BIN".to_string(),
        file_name: "LINKDATA_A.BIN".to_string(),
        source: ReplacementSource::File(override_path),
        mode: ReplacementMode::Internal,
        mod_size: Some(metadata.len()),
        hash: 0,
        rdb_block_offset: 0,
        original_data_offset: 0,
        original_bin_offset: Some(0),
        original_bin_size: Some(metadata.len() as u32),
        virtual_bin_offset: None,
        rdb_tail_offset: None,
        original_tail: None,
        virtual_prefix: None,
    }]
}
