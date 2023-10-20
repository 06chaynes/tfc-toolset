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
    tag,
    workspace::{self, Workspace, WorkspaceTags},
};
use tfc_toolset_extras::file::input::tag::TagsFile;
use tfc_toolset_extras::parse_workspace_name;

pub async fn list(
    args: &WorkspaceArgs,
    core: &Core,
    config: &Settings,
    client: Client,
) -> miette::Result<Vec<WorkspaceTags>, ArgError> {
    check_workspace_identifier(args)?;
    let mut workspaces_tags = Vec::new();
    if let Some(workspace_name) = &args.workspace_name {
        parse_workspace_name(workspace_name)?;
        let workspace =
            workspace::show_by_name(workspace_name, core, client.clone())
                .await?;
        process(&mut workspaces_tags, vec![workspace], core, client.clone())
            .await?;
    } else if let Some(workspace_id) = &args.workspace_id {
        let workspace =
            workspace::show(workspace_id, core, client.clone()).await?;
        process(&mut workspaces_tags, vec![workspace], core, client.clone())
            .await?;
    } else if let Some(file_path) = &args.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, core, client.clone()).await?;
        process(&mut workspaces_tags, workspaces, core, client.clone()).await?;
    } else if args.auto_discover_workspaces {
        let workspaces = workspace::list(true, core, client.clone()).await?;
        process(&mut workspaces_tags, workspaces, core, client.clone()).await?;
    }
    info!("{:#?}", &workspaces_tags);
    if core.save_output {
        TagsFile::from(workspaces_tags.clone())
            .save(&core.output, config.pretty_output)?;
    }
    Ok(workspaces_tags)
}

async fn process(
    workspaces_tags: &mut Vec<WorkspaceTags>,
    workspaces: Vec<Workspace>,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    for workspace in workspaces {
        info!("Listing tags for workspace {}.", workspace.id);
        let tags = tag::list(&workspace.id, config, client.clone()).await?;
        workspaces_tags.push(WorkspaceTags { workspace, tags: vec![tags] });
    }
    Ok(())
}
