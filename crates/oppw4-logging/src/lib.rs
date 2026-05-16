// crates/oppw4-logging/src/lib.rs
mod file_logger;
mod timestamp;

pub use file_logger::{initialize_game_logger, write_line};
