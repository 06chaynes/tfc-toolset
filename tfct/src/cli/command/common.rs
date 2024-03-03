use crate::error::ArgError;
use clap::Args;
use surf::Client;
use tfc_toolset::{
    settings::Core,
    workspace::{self, Workspace},
};
use tfc_toolset_extras::WorkspacesFile;

const WORKSPACE_NAME: &str = "The name of the workspace";

const WORKSPACE_ID: &str = "The ID of the workspace";
const WORKSPACE_FILE: &str = "The path to a file containing a list of \
    workspace names or IDs";

const AUTO_DISCOVER_WORKSPACES: &str = "Automatically discover \
    workspaces given the specified filters";

#[derive(Args, Debug)]
pub struct WorkspaceArgs {
    #[arg(
    short,
    long,
    help = WORKSPACE_NAME,
    conflicts_with = "workspace_id",
    conflicts_with = "workspace_file",
    conflicts_with = "auto_discover_workspaces"
    )]
    pub workspace_name: Option<String>,
    #[arg(
    short = 'i',
    long,
    help = WORKSPACE_ID,
    conflicts_with = "workspace_name",
    conflicts_with = "workspace_file",
    conflicts_with = "auto_discover_workspaces"
    )]
    pub workspace_id: Option<String>,
    #[arg(
    short = 'f',
    long,
    help = WORKSPACE_FILE,
    conflicts_with = "workspace_id",
    conflicts_with = "workspace_name",
    conflicts_with = "auto_discover_workspaces"
    )]
    pub workspace_file: Option<String>,
    #[arg(
    short,
    long,
    action,
    help = AUTO_DISCOVER_WORKSPACES,
    default_value = "false",
    required = false,
    conflicts_with = "workspace_id",
    conflicts_with = "workspace_name",
    conflicts_with = "workspace_file"
    )]
    pub auto_discover_workspaces: bool,
}

pub(crate) fn check_workspace_identifier(
    args: &WorkspaceArgs,
) -> Result<(), ArgError> {
    if args.workspace_name.is_none()
        && args.workspace_id.is_none()
        && args.workspace_file.is_none()
        && !args.auto_discover_workspaces
    {
        return Err(ArgError::MissingWorkspaceIdentifier);
    }
    Ok(())
}

#[derive(Args, Debug)]
pub struct WorkspaceArgsBasic {
    #[arg(
    short,
    long,
    help = WORKSPACE_NAME,
    conflicts_with = "workspace_id"
    )]
    pub workspace_name: Option<String>,
    #[arg(
    short = 'i',
    long,
    help = WORKSPACE_ID,
    conflicts_with = "workspace_name"
    )]
    pub workspace_id: Option<String>,
}

pub fn check_workspace_identifier_basic(
    args: &WorkspaceArgsBasic,
) -> Result<(), ArgError> {
    if args.workspace_name.is_none() && args.workspace_id.is_none() {
        return Err(ArgError::MissingWorkspaceIdentifierBasic);
    }
    Ok(())
}

pub(crate) async fn parse_workspace_file(
    path: &str,
    config: &Core,
    client: Client,
) -> Result<Vec<Workspace>, ArgError> {
    let workspace_file = WorkspacesFile::load(path)?;
    let mut workspaces = vec![];
    if let Some(workspace_entries) = workspace_file.workspaces {
        for workspace_entry in workspace_entries {
            if workspace_entry.id.is_some()
                && workspace_entry.attributes.is_some()
            {
                let workspace = Workspace {
                    id: workspace_entry.id.unwrap(),
                    attributes: workspace_entry.attributes.unwrap(),
                };
                workspaces.push(workspace);
            } else if workspace_entry.id.is_some() {
                let workspace_id = &workspace_entry.id.unwrap();
                let workspace =
                    workspace::show(workspace_id, config, client.clone())
                        .await?;
                workspaces.push(workspace);
            } else if workspace_entry.name.is_some() {
                let workspace_name = &workspace_entry.name.unwrap();
                let workspace = workspace::show_by_name(
                    workspace_name,
                    config,
                    client.clone(),
                )
                .await?;
                workspaces.push(workspace);
            }
        }
    }
    Ok(workspaces)
}
