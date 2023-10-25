use super::ManageArgs;
use crate::{
    cli::{
        command::common::{check_workspace_identifier, parse_workspace_file},
        tag::{check_tag_identifier, parse_tag_file},
    },
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
    check_tag_identifier(args)?;
    let mut tags = Vec::new();
    for tag in &args.name {
        parse_tag_name(tag)?;
        tags.push(tag.clone());
    }
    if let Some(tag_file) = &args.tag_file {
        let tag_file = parse_tag_file(tag_file).await?;
        for tag in tag_file {
            tags.push(tag.attributes.name);
        }
    }
    if let Some(workspace_name) = &args.default.workspace_name {
        parse_workspace_name(workspace_name)?;
        let workspace =
            workspace::show_by_name(workspace_name, core, client.clone())
                .await?;
        process(vec![workspace], tags, core, client.clone()).await?;
    } else if let Some(workspace_id) = &args.default.workspace_id {
        let workspace =
            workspace::show(workspace_id, core, client.clone()).await?;
        process(vec![workspace], tags, core, client.clone()).await?;
    } else if let Some(file_path) = &args.default.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, core, client.clone()).await?;
        process(workspaces, tags, core, client.clone()).await?;
    } else if args.default.auto_discover_workspaces {
        let workspaces = workspace::list(true, core, client.clone()).await?;
        process(workspaces, tags, core, client.clone()).await?;
    }
    info!("Finished adding tags.");
    Ok(())
}

async fn process(
    workspaces: Vec<Workspace>,
    tags: Vec<String>,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    for workspace in workspaces {
        info!("Adding tags to workspace {}.", workspace.id);
        tag::add(&workspace.id, tags.clone(), config, client.clone()).await?;
    }
    Ok(())
}
