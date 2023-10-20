use super::ManageArgs;
use crate::{
    cli::command::common::{check_workspace_identifier, parse_workspace_file},
    error::ArgError,
    settings::Settings,
    variable::check_variable_identifier,
};

use crate::cli::variable::parse_variable_file;
use log::{debug, info};
use std::str::FromStr;
use surf::Client;
use tfc_toolset::{
    settings::Core,
    variable,
    workspace::{self, WorkspaceVariables},
};
use tfc_toolset_extras::{parse_workspace_name, VariablesFile};

pub async fn create(
    args: &ManageArgs,
    core: &Core,
    config: &Settings,
    client: Client,
) -> miette::Result<Vec<WorkspaceVariables>, ArgError> {
    check_workspace_identifier(&args.default)?;
    check_variable_identifier(args)?;
    let mut vars: Option<Vec<variable::Variable>> = None;
    for var_string in args.var.iter() {
        let var = variable::Variable::from_str(var_string)?;
        match vars {
            Some(ref mut vars) => vars.push(var),
            None => vars = Some(vec![var]),
        }
    }
    if let Some(variables_file) = &args.var_file {
        let variables = parse_variable_file(variables_file).await?;
        match vars {
            Some(ref mut vars) => vars.extend(variables),
            None => vars = Some(variables),
        }
    }
    debug!("vars: {:#?}", vars);
    let mut workspaces_variables = Vec::new();
    if let Some(workspace_name) = &args.default.workspace_name {
        parse_workspace_name(workspace_name)?;
        let workspace =
            workspace::show_by_name(workspace_name, core, client.clone())
                .await?;
        process(
            &mut workspaces_variables,
            vec![workspace],
            vars,
            core,
            client.clone(),
        )
        .await?;
    } else if let Some(workspace_id) = &args.default.workspace_id {
        let workspace =
            workspace::show(workspace_id, core, client.clone()).await?;
        process(
            &mut workspaces_variables,
            vec![workspace],
            vars,
            core,
            client.clone(),
        )
        .await?;
    } else if let Some(file_path) = &args.default.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, core, client.clone()).await?;
        process(
            &mut workspaces_variables,
            workspaces,
            vars,
            core,
            client.clone(),
        )
        .await?;
    } else if args.default.auto_discover_workspaces {
        let workspaces = workspace::list(true, core, client.clone()).await?;
        process(
            &mut workspaces_variables,
            workspaces,
            vars,
            core,
            client.clone(),
        )
        .await?;
    }
    info!("{:#?}", &workspaces_variables);
    if core.save_output {
        VariablesFile::from(workspaces_variables.clone())
            .save(&core.output, config.pretty_output)?;
    }
    Ok(workspaces_variables)
}

async fn process(
    workspaces_variables: &mut Vec<WorkspaceVariables>,
    workspaces: Vec<workspace::Workspace>,
    vars: Option<Vec<variable::Variable>>,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    for workspace in workspaces {
        info!("Adding variables to workspace {}.", workspace.id);
        let mut entry = WorkspaceVariables {
            workspace: workspace.clone(),
            variables: vec![],
        };
        if let Some(vars) = vars.clone() {
            for var in vars {
                info!("Adding variable {}", &var.attributes.key);
                let variable = variable::create(
                    &workspace.id,
                    var,
                    config,
                    client.clone(),
                )
                .await?;
                entry.variables.push(variable);
            }
        }
        workspaces_variables.push(entry);
    }
    Ok(())
}
