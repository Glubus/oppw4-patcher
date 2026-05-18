use crate::{
    abi::{Oppw4FileProvider, Oppw4PluginApi},
    error::PluginError,
    host_api::r#unsafe,
    PluginResult,
};

#[derive(Clone, Copy)]
pub struct FileService<'api> {
    abi: &'api Oppw4PluginApi,
}

impl<'api> FileService<'api> {
    pub(super) const fn new(abi: &'api Oppw4PluginApi) -> Self {
        Self { abi }
    }

    pub fn register_provider(self, provider: &Oppw4FileProvider) -> PluginResult<()> {
        let register = self
            .abi
            .register_file_provider
            .ok_or(PluginError::MissingHostFunction("register_file_provider"))?;
        let code = r#unsafe::register_file_provider(self.abi.host_context, register, provider);
        if code == 0 {
            Ok(())
        } else {
            Err(PluginError::HostCallFailed {
                operation: "register_file_provider",
                code,
            })
        }
    }
}
