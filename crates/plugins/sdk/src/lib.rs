pub mod abi;
pub mod host;

pub use abi::*;

mod context;
mod entry;
mod error;
mod helpers;
mod host_api;
mod log;
mod plugin;
mod traits;
mod r#unsafe;

pub use context::PluginContext;
pub use entry::{plugin_abi_from_raw, PluginInitError};
pub use error::{PluginError, PluginResult};
pub use helpers::cstring_lossy;
pub use host_api::{
    FileService, GameService, HostApi, LogService, LuaService, MemoryService, ModService,
    OwnedHostApi, PathService,
};
pub use log::LogPolicy;
pub use plugin::{init_plugin, Plugin};
pub use traits::{FileProvider, LuaModule};

#[macro_export]
macro_rules! export_plugin {
    ($plugin:ty) => {
        #[no_mangle]
        pub unsafe extern "system" fn oppw4_plugin_init(api: *const $crate::Oppw4PluginApi) -> i32 {
            $crate::init_plugin::<$plugin>(api)
        }
    };
}
