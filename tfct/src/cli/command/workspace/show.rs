use crate::cli::command::common::{
    check_workspace_identifier_basic, WorkspaceArgsBasic,
};
use crate::settings::Settings;
use log::info;
use miette::IntoDiagnostic;
use surf::Client;
use tfc_toolset::{settings::Core, workspace};
use tfc_toolset_extras::WorkspacesFile;

pub async fn show(
    args: &WorkspaceArgsBasic,
    core: &Core,
    config: &Settings,
    client: Client,
) -> miette::Result<workspace::Workspace> {
    check_workspace_identifier_basic(args)?;
    if let Some(workspace_name) = &args.workspace_name {
        info!("Retrieving Workspace: {}", workspace_name);
        let workspace = workspace::show_by_name(workspace_name, core, client)
            .await
            .into_diagnostic()?;
        info!("{:#?}", &workspace);
        if core.save_output {
            WorkspacesFile::from(vec![workspace.clone()])
                .save(&core.output, config.pretty_output)
                .into_diagnostic()?;
        }
        Ok(workspace)
    } else if let Some(workspace_id) = &args.workspace_id {
        info!("Retrieving Workspace: {}", workspace_id);
        let workspace = workspace::show(workspace_id, core, client)
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
