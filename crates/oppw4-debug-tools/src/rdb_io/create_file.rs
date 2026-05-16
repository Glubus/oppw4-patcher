// crates/oppw4-debug-tools/src/rdb_io/create_file.rs
pub fn is_interesting_create_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("oppw4")
        || lower.contains("op4")
        || lower.contains(".rdb")
        || lower.contains(".g1")
        || lower.contains("0x")
}

#[cfg(test)]
mod tests {
    use crate::rdb_io::is_interesting_create_path;

    #[test]
    fn create_file_filter_keeps_game_assets_only() {
        assert!(is_interesting_create_path("D:\\OPPW4\\CharacterEditor.rdb"));
        assert!(is_interesting_create_path("MPLC026_Law.g1m"));
        assert!(!is_interesting_create_path("C:\\Windows\\notepad.exe"));
    }
}
