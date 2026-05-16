mod global;
mod log;
mod memory;
mod win;

pub use global::{install_main_module_hooks, register_file_provider};
pub use log::set_logger;
pub use memory::{module_base, read_memory, scan_memory, write_memory};
