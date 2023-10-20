use super::about;
use crate::cli::command::common::{
    check_workspace_identifier_basic, WorkspaceArgsBasic,
};

use clap::Args;
use log::info;
use miette::IntoDiagnostic;
use surf::Client;
use tfc_toolset::{settings::Core, workspace};
use tfc_toolset_extras::parse_workspace_name;

#[derive(Args, Debug)]
pub struct DeleteArgs {
    #[arg(
    short,
    long,
    action,
    help = about::SAFE_DELETE,
    default_value = "false",
    required = false
    )]
    pub safe: bool,
    #[clap(flatten)]
    workspace: WorkspaceArgsBasic,
}

pub async fn delete(
    args: &DeleteArgs,
    config: &Core,
    client: Client,
) -> miette::Result<()> {
    check_workspace_identifier_basic(&args.workspace)?;
    if let Some(workspace_name) = &args.workspace.workspace_name {
        parse_workspace_name(workspace_name).into_diagnostic()?;
        info!("Deleting Workspace: {}", workspace_name);
        workspace::delete_by_name(workspace_name, args.safe, config, client)
            .await
            .into_diagnostic()?;
    } else if let Some(workspace_id) = &args.workspace.workspace_id {
        info!("Deleting Workspace: {}", workspace_id);
        workspace::delete(workspace_id, args.safe, config, client)
            .await
            .into_diagnostic()?;
    }
    Ok(())
}
