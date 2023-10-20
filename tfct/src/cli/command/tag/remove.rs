use super::ManageArgs;
use crate::{
    cli::command::common::{check_workspace_identifier, parse_workspace_file},
    error::ArgError,
};

use log::info;
use surf::Client;
use tfc_toolset::{
    settings::Core,
    tag,
    workspace::{self, Workspace},
};
use tfc_toolset_extras::{parse_tag_name, parse_workspace_name};

pub async fn remove(
    args: &ManageArgs,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    check_workspace_identifier(&args.default)?;
    for tag in &args.name {
        parse_tag_name(tag)?;
    }
    if let Some(workspace_name) = &args.default.workspace_name {
        parse_workspace_name(workspace_name)?;
        let workspace =
            workspace::show_by_name(workspace_name, config, client.clone())
                .await?;
        process(vec![workspace], args, config, client.clone()).await?;
    } else if let Some(workspace_id) = &args.default.workspace_id {
        let workspace =
            workspace::show(workspace_id, config, client.clone()).await?;
        process(vec![workspace], args, config, client.clone()).await?;
    } else if let Some(file_path) = &args.default.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, config, client.clone()).await?;
        process(workspaces, args, config, client.clone()).await?;
    } else if args.default.auto_discover_workspaces {
        let workspaces = workspace::list(true, config, client.clone()).await?;
        process(workspaces, args, config, client.clone()).await?;
    }
    info!("Finished removing tags.");
    Ok(())
}

async fn process(
    workspaces: Vec<Workspace>,
    args: &ManageArgs,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    for workspace in workspaces {
        info!("Removing tags from workspace {}.", workspace.id);
        tag::remove(&workspace.id, args.name.clone(), config, client.clone())
            .await?;
    }
    Ok(())
}
