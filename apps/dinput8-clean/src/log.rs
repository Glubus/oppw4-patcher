use std::path::PathBuf;

pub fn initialize(game_root: PathBuf) {
    oppw4_logging::initialize_game_logger(game_root);
}

pub fn write_line(message: impl AsRef<str>) {
    oppw4_logging::write_line(message);
}
