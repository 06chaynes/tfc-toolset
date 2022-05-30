use miette::Diagnostic;
use thiserror::Error;

pub const SETTINGS_ERROR: &str = "Uh Oh, looks like a settings issue! By default I look for a settings.toml file and override with env variables.";

/// A generic “error” type
#[derive(Error, Diagnostic, Debug)]
pub enum ToolError {
    /// A general error used as a catch all for other errors via anyhow
    #[error(transparent)]
    #[diagnostic(code(tfc_toolset::general))]
    General(#[from] anyhow::Error),
    /// URL parsing related errors
    #[error(transparent)]
    #[diagnostic(
        code(tfc_toolset::url),
        help("Oops, something went wrong building the URL!")
    )]
    Url(#[from] url::ParseError),
    /// JSON Serialization\Deserialization related errors
    #[error(transparent)]
    #[diagnostic(
        code(tfc_toolset::json),
        help("Aw snap, ran into an issue parsing the json response!")
    )]
    Json(#[from] serde_json::Error),
    /// Integer parsing related errors
    #[error(transparent)]
    #[diagnostic(
        code(tfc_toolset::int),
        help("Oh no, ran into an issue parsing an integer!")
    )]
    Int(#[from] std::num::ParseIntError),
    /// std IO related errors
    #[error(transparent)]
    #[diagnostic(code(tfc_toolset::io), help("Dangit, IO issue!"))]
    Io(#[from] std::io::Error),
}
