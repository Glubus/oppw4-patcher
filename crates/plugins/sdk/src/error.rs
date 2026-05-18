pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    MissingHostFunction(&'static str),
    HostCallFailed { operation: &'static str, code: i32 },
    InvalidApiVersion { expected: u32, actual: u32 },
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHostFunction(name) => write!(formatter, "missing host function {name}"),
            Self::HostCallFailed { operation, code } => {
                write!(formatter, "{operation} failed with code {code}")
            }
            Self::InvalidApiVersion { expected, actual } => {
                write!(
                    formatter,
                    "invalid plugin api version expected={expected} actual={actual}"
                )
            }
        }
    }
}

impl std::error::Error for PluginError {}
