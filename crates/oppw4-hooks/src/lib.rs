mod global;
mod log;
mod win;

pub use global::{
    clear_virtual_replacements, commit_virtual_replacements, install_main_module_hooks,
    publish_replacements, register_virtual_replacement,
};
pub use log::set_logger;
