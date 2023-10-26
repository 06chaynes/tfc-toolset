use crate::{
    cli::{
        command::common::{check_workspace_identifier, parse_workspace_file},
        variable_set::ManageWorkspaceArgs,
    },
    error::ArgError,
};

use log::info;
use surf::Client;
use tfc_toolset::{settings::Core, variable_set::remove_workspace, workspace};
use tfc_toolset_extras::parse_workspace_name;

pub async fn remove(
    args: &ManageWorkspaceArgs,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    check_workspace_identifier(&args.default)?;
    if let Some(workspace_name) = &args.default.workspace_name {
        parse_workspace_name(workspace_name)?;
        info!("Removing variable set from workspace {}.", workspace_name);
        let workspace =
            workspace::show_by_name(workspace_name, config, client.clone())
                .await?;
        remove_var_set_by_id(
            args.var_set_id.clone(),
            vec![workspace],
            config,
            client.clone(),
        )
        .await?;
    } else if let Some(workspace_id) = &args.default.workspace_id {
        info!("Removing variable set from workspace {}.", workspace_id);
        let workspace =
            workspace::show(workspace_id, config, client.clone()).await?;
        remove_var_set_by_id(
            args.var_set_id.clone(),
            vec![workspace],
            config,
            client.clone(),
        )
        .await?;
    } else if let Some(file_path) = &args.default.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, config, client.clone()).await?;
        remove_var_set_by_id(
            args.var_set_id.clone(),
            workspaces,
            config,
            client.clone(),
        )
        .await?;
    } else if args.default.auto_discover_workspaces {
        let workspaces = workspace::list(true, config, client.clone()).await?;
        remove_var_set_by_id(
            args.var_set_id.clone(),
            workspaces,
            config,
            client.clone(),
        )
        .await?;
    }
    info!("Finished removing variable set from workspace/s.");
    Ok(())
}

async fn remove_var_set_by_id(
    var_set_ids: Vec<String>,
    workspaces: Vec<workspace::Workspace>,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    for var_set_id in var_set_ids {
        remove_workspace(
            &var_set_id,
            workspaces.clone(),
            config,
            client.clone(),
        )
        .await?;
    }
    Ok(())
}
