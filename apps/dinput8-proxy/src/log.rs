use std::{fs::create_dir_all, path::PathBuf, sync::OnceLock};

use crate::time;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOG_STAMP: OnceLock<String> = OnceLock::new();
static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

pub fn initialize(game_root: PathBuf) {
    let log_dir = game_root.join("mods").join("_oppw4").join("logs");
    let _ = create_dir_all(&log_dir);
    let stamp = time::file_timestamp();
    let log_file_name = format!("{stamp}.log");
    let log_path = log_dir.join(&log_file_name);
    let file_appender = tracing_appender::rolling::never(&log_dir, &log_file_name);
    let (writer, guard) = tracing_appender::non_blocking(file_appender);
    let _ = LOG_GUARD.set(guard);
    let _ = LOG_STAMP.set(stamp);
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

pub fn session_stamp() -> Option<String> {
    LOG_STAMP.get().cloned()
}

pub fn write_line(message: impl AsRef<str>) {
    tracing::info!("{}", message.as_ref());
}
