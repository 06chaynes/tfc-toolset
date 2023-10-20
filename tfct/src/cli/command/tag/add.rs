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

pub async fn add(
    args: &ManageArgs,
    core: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    check_workspace_identifier(&args.default)?;
    for tag in &args.name {
        parse_tag_name(tag)?;
    }
    if let Some(workspace_name) = &args.default.workspace_name {
        parse_workspace_name(workspace_name)?;
        let workspace =
            workspace::show_by_name(workspace_name, core, client.clone())
                .await?;
        process(vec![workspace], args, core, client.clone()).await?;
    } else if let Some(workspace_id) = &args.default.workspace_id {
        let workspace =
            workspace::show(workspace_id, core, client.clone()).await?;
        process(vec![workspace], args, core, client.clone()).await?;
    } else if let Some(file_path) = &args.default.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, core, client.clone()).await?;
        process(workspaces, args, core, client.clone()).await?;
    } else if args.default.auto_discover_workspaces {
        let workspaces = workspace::list(true, core, client.clone()).await?;
        process(workspaces, args, core, client.clone()).await?;
    }
    info!("Finished adding tags.");
    Ok(())
}

async fn process(
    workspaces: Vec<Workspace>,
    args: &ManageArgs,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    for workspace in workspaces {
        info!("Adding tags to workspace {}.", workspace.id);
        tag::add(&workspace.id, args.name.clone(), config, client.clone())
            .await?;
    }
    Ok(())
}
