use thiserror::Error;

pub const SETTINGS_ERROR: &str = "Uh Oh, looks like a settings issue! By default I look for a settings.toml file and override with env variables.";

pub(crate) fn surf_to_tool_error(e: surf::Error) -> ToolError {
    ToolError::General(e.into_inner())
}

/// A generic “error” type
#[derive(Error, Debug)]
pub enum ToolError {
    /// A general error used as a catch all for other errors via anyhow
    #[error(transparent)]
    General(#[from] anyhow::Error),
    /// URL parsing related errors
    #[error(transparent)]
    Url(#[from] url::ParseError),
    /// JSON Serialization\Deserialization related errors
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Integer parsing related errors
    #[error(transparent)]
    Int(#[from] std::num::ParseIntError),
    /// std IO related errors
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Error parsing boolean value
    #[error(transparent)]
    Bool(#[from] std::str::ParseBoolError),
}
