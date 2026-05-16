// crates/oppw4-logging/src/file_logger.rs
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use tracing_appender::non_blocking::WorkerGuard;

use crate::timestamp::current_log_file_name;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn initialize_game_logger(game_root: impl AsRef<Path>) {
    let log_dir = game_root.as_ref().join("mods").join("_oppw4").join("logs");
    let _ = create_dir_all(&log_dir);

    let log_file_name = current_log_file_name();
    let log_path = log_dir.join(&log_file_name);
    let file_appender = tracing_appender::rolling::never(&log_dir, &log_file_name);
    let (writer, guard) = tracing_appender::non_blocking(file_appender);
    let _ = LOG_GUARD.set(guard);
    let _ = LOG_PATH.set(log_path.clone());

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .with_target(false)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    tracing::info!(log_path = %log_path.display(), "OPPW4 Rust proxy loaded");
}

pub fn write_line(message: impl AsRef<str>) {
    tracing::info!("{}", message.as_ref());
}
