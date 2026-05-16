mod global;
mod log;
mod win;

pub use global::{install_main_module_hooks, publish_replacements};
pub use log::set_logger;
