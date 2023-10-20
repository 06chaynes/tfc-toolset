use super::WorkspaceArgs;
use crate::{
    cli::command::common::{check_workspace_identifier, parse_workspace_file},
    error::ArgError,
    settings::Settings,
};

use log::info;
use surf::Client;
use tfc_toolset::{
    settings::Core,
    variable,
    workspace::{self, WorkspaceVariables},
};
use tfc_toolset_extras::{parse_workspace_name, VariablesFile};

pub async fn list(
    args: &WorkspaceArgs,
    core: &Core,
    config: &Settings,
    client: Client,
) -> miette::Result<Vec<WorkspaceVariables>, ArgError> {
    check_workspace_identifier(args)?;
    let mut workspaces_variables = Vec::new();
    if let Some(workspace_name) = &args.workspace_name {
        parse_workspace_name(workspace_name)?;
        info!("Retrieving variables for workspace {}.", workspace_name);
        let workspace =
            workspace::show_by_name(workspace_name, core, client.clone())
                .await?;
        let variables =
            variable::list(&workspace.id, core, client.clone()).await?;
        workspaces_variables =
            vec![WorkspaceVariables { workspace, variables }];
    } else if let Some(workspace_id) = &args.workspace_id {
        info!("Retrieving variables for workspace {}.", workspace_id);
        let workspace =
            workspace::show(workspace_id, core, client.clone()).await?;
        let variables =
            variable::list(workspace_id, core, client.clone()).await?;
        workspaces_variables =
            vec![WorkspaceVariables { workspace, variables }];
    } else if let Some(file_path) = &args.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, core, client.clone()).await?;
        info!("Batch retrieving variables for workspaces: {:#?}", workspaces);
        workspaces_variables =
            variable::list_batch(core, client.clone(), workspaces).await?;
    } else if args.auto_discover_workspaces {
        let workspaces = workspace::list(true, core, client.clone()).await?;
        info!("Batch retrieving variables for workspaces: {:#?}", workspaces);
        workspaces_variables =
            variable::list_batch(core, client.clone(), workspaces).await?;
    }
    info!("{:#?}", &workspaces_variables);
    if core.save_output {
        VariablesFile::from(workspaces_variables.clone())
            .save(&core.output, config.pretty_output)?;
    }
    Ok(workspaces_variables)
}
