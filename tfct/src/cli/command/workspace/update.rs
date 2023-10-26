use super::{build_options, DefaultArgs};
use crate::{
    cli::command::common::{
        check_workspace_identifier_basic, WorkspaceArgsBasic,
    },
    settings::Settings,
};

use clap::Args;
use log::{debug, info};
use miette::IntoDiagnostic;
use surf::Client;
use tfc_toolset::{
    settings::Core,
    workspace::{self, Workspace},
};
use tfc_toolset_extras::WorkspacesFile;

#[derive(Args, Debug)]
pub struct UpdateArgs {
    #[clap(flatten)]
    default: DefaultArgs,
    #[clap(flatten)]
    workspace: WorkspaceArgsBasic,
}

pub async fn update(
    args: &UpdateArgs,
    core: &Core,
    config: &Settings,
    client: Client,
) -> miette::Result<Workspace> {
    check_workspace_identifier_basic(&args.workspace)?;
    let options = build_options(&args.default)?;
    debug!("{:#?}", &options);
    if let Some(workspace_name) = &args.workspace.workspace_name {
        info!("Updating Workspace: {}", workspace_name);
        let workspace =
            workspace::update_by_name(workspace_name, options, core, client)
                .await
                .into_diagnostic()?;
        info!("{:#?}", &workspace);
        if core.save_output {
            WorkspacesFile::from(vec![workspace.clone()])
                .save(&core.output, config.pretty_output)
                .into_diagnostic()?;
        }
        Ok(workspace)
    } else if let Some(workspace_id) = &args.workspace.workspace_id {
        info!("Updating Workspace: {}", workspace_id);
        let workspace = workspace::update(workspace_id, options, core, client)
            .await
            .into_diagnostic()?;
        info!("{:#?}", &workspace);
        if core.save_output {
            WorkspacesFile::from(vec![workspace.clone()])
                .save(&core.output, config.pretty_output)
                .into_diagnostic()?;
        }
        Ok(workspace)
    } else {
        unreachable!()
    }
}
