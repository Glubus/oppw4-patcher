// crates/oppw4-debug-tools/src/rdb_io/hex.rs
pub fn hex_preview(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use crate::rdb_io::hex_preview;

    #[test]
    fn formats_compact_lowercase_hex() {
        assert_eq!(hex_preview(&[0, 1, 0xab, 0xff]), "0001abff");
    }
}
