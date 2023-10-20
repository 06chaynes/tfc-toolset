use super::{about, WorkspaceArgs};
use crate::{
    cli::command::common::{check_workspace_identifier, parse_workspace_file},
    error::ArgError,
};

use crate::cli::variable::parse_variable_file;
use clap::Args;
use log::info;
use surf::Client;
use tfc_toolset::{error::ToolError, settings::Core, variable, workspace};
use tfc_toolset_extras::parse_workspace_name;

#[derive(Args, Debug)]
pub struct DeleteArgs {
    #[arg(short = 'k', long, help = about::VARIABLE_KEY)]
    pub var_key: Option<Vec<String>>,
    #[arg(short = 'i', long, help = about::VARIABLE_ID)]
    pub var_id: Option<Vec<String>>,
    #[arg(long, help = about::VAR_FILE)]
    pub var_file: Option<String>,
    #[clap(flatten)]
    default: WorkspaceArgs,
}

pub async fn delete(
    args: &DeleteArgs,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    check_workspace_identifier(&args.default)?;
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
    Ok(())
}

async fn process(
    workspaces: Vec<workspace::Workspace>,
    args: &DeleteArgs,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    for workspace in workspaces {
        info!("Deleting variable from workspace {}.", workspace.id);
        if let Some(vars) = args.var_id.clone() {
            delete_vars_by_id(vars, &workspace.id, config, client.clone())
                .await?;
        } else if let Some(vars) = args.var_key.clone() {
            delete_vars_by_key(vars, &workspace.id, config, client.clone())
                .await?;
        } else if let Some(variables_file) = &args.var_file {
            let vars = parse_variable_file(variables_file).await?;
            delete_vars_by_key(
                vars.iter().map(|v| v.attributes.key.clone()).collect(),
                &workspace.id,
                config,
                client.clone(),
            )
            .await?;
        }
    }
    Ok(())
}

async fn delete_vars_by_id(
    vars: Vec<String>,
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> miette::Result<(), ToolError> {
    for var in vars {
        info!("Deleting variable {}", &var);
        variable::delete(&var, workspace_id, config, client.clone()).await?;
    }
    info!("Finished deleting variables.");
    Ok(())
}

async fn delete_vars_by_key(
    vars: Vec<String>,
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> miette::Result<(), ToolError> {
    for var in vars {
        let variables =
            variable::list(workspace_id, config, client.clone()).await?;
        for variable in variables {
            if variable.attributes.key == var {
                let var_id = variable.id.unwrap();
                info!("Deleting variable {}", &var_id);
                variable::delete(&var_id, workspace_id, config, client.clone())
                    .await?;
            }
        }
    }
    Ok(())
}
