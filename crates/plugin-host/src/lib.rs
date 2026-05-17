pub mod host;
mod log;

pub use host::{initialize, set_debug_enabled};
pub use log::set_logger;
