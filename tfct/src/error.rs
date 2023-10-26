use miette::Diagnostic;
use thiserror::Error;

/// An error type for argument parsing
#[derive(Error, Diagnostic, Debug)]
pub enum ArgError {
    /// Missing VCS OAuth Token ID
    #[error("VCS OAuth Token ID is required when VCS Identifier is provided")]
    #[diagnostic(
    code(tfct::workspace::create::missing_vcs_oauth_token_id),
    help("If `--vcs-identifier` is passed, you must also pass --vcs-oauth-token-id")
    )]
    MissingVcsOauthTokenId,
    /// Unable to parse to RFC3339 timestamp
    #[error("Unable to parse to RFC3339 timestamp")]
    #[diagnostic(
    code(tfct::workspace::parse::bad_rfc3339_timestamp),
    help("Must be a valid RFC3339 timestamp, see https://www.rfc-editor.org/rfc/rfc3339")
    )]
    BadRFC3339Timestamp,
    /// Missing workspace name
    #[error("Missing workspace name")]
    #[diagnostic(
        code(tfct::tag::missing_workspace_name),
        help("Must provide a name `--name`")
    )]
    MissingWorkspaceName,
    /// Missing variable identifier
    #[error("Missing variable identifier")]
    #[diagnostic(
    code(tfct::tag::missing_variable_identifier),
    help("Must provide either `--var-key` (-k) or `--var-id` (-v) or `--var-file`")
    )]
    MissingVariableIdentifier,
    /// Missing variable identifier basic version
    #[error("Missing variable identifier")]
    #[diagnostic(
        code(tfct::tag::missing_variable_identifier),
        help("Must provide either `--var` or `--var-file`")
    )]
    MissingVariableIdentifierBasic,
    /// Missing tag identifier
    #[error("Missing tag identifier")]
    #[diagnostic(
        code(tfct::tag::missing_tag_identifier),
        help("Must provide either `--name` or `--tag-file`")
    )]
    MissingTagIdentifier,
    /// Missing workspace identifier
    #[error("Missing workspace identifier")]
    #[diagnostic(
        code(tfct::tag::missing_workspace_identifier),
        help("Must provide either `--workspace-name` (-w) or `--workspace-id` (-i) or `--workspace-file` (-f) or `--auto-discover-workspaces` (-a)")
    )]
    MissingWorkspaceIdentifier,
    /// Missing workspace identifier basic version
    #[error("Missing workspace identifier")]
    #[diagnostic(
        code(tfct::tag::missing_workspace_identifier),
        help("Must provide either `--workspace-name` (-w) or `--workspace-id` (-i)")
    )]
    MissingWorkspaceIdentifierBasic,
    /// Errors from tfc-toolset
    #[error(transparent)]
    #[diagnostic(code(tfct::tfc_toolset::tool_error))]
    ToolSetError(#[from] tfc_toolset::error::ToolError),
    /// Errors from tfc-toolset-extras
    #[error(transparent)]
    #[diagnostic(code(tfct::tfc_toolset_extras::extras_error))]
    ExtrasError(#[from] tfc_toolset_extras::error::ExtrasError),
}
