use thiserror::Error;

/// A generic “error” type
#[derive(Error, Debug)]
pub enum ExtrasError {
    /// Regex related errors
    #[error(transparent)]
    Regex(#[from] regex::Error),
    /// Error parsing workspace name
    #[error("Workspace name {0} contains invalid characters. Only alphanumeric, dashes, and underscores are allowed.")]
    BadWorkspaceName(String),
    /// Error parsing tag name
    #[error("Tag name {0} contains invalid characters. Only alphanumeric, colons, dashes, and underscores are allowed.")]
    BadTagName(String),
    /// Error parsing workspaces file
    #[error("Workspaces file is invalid. Each workspace entry must contain a name or ID.")]
    InvalidWorkspacesFile,
    /// Error from core library
    #[error(transparent)]
    ToolError(#[from] tfc_toolset::error::ToolError),
}
