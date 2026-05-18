#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogPolicy {
    pub host: bool,
}

impl LogPolicy {
    pub const HOST: Self = Self { host: true };
    pub const SILENT: Self = Self { host: false };
}
