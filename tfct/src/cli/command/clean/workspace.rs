use super::{process, CleanWorkspaceArgs};
use crate::{
    cli::command::common::{check_workspace_identifier, parse_workspace_file},
    error::ArgError,
    settings::Settings,
};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::Path};
use surf::Client;
use tfc_toolset::{
    error::ToolError,
    variable::Variable,
    workspace::{VcsRepo, WorkspaceVariables},
};
use tfc_toolset::{
    settings::Core,
    variable,
    workspace::{self, Workspace},
};
use tfc_toolset_extras::{parse_workspace_name, ExtrasError};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnlistedVariables {
    pub workspace: WorkspaceVariables,
    pub unlisted_variables: Vec<Variable>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Errors {
    pub parsing_failures: Option<ParsingFailures>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParsingFailures {
    pub repos: Vec<VcsRepo>,
    pub workspaces: Vec<Workspace>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CleanupResult {
    pub missing_repositories: Option<Vec<Workspace>>,
    pub unlisted_variables: Option<Vec<UnlistedVariables>>,
    pub workspaces: Vec<Workspace>,
    pub errors: Errors,
}

impl CleanupResult {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ExtrasError> {
        let file = File::open(path).map_err(ToolError::Io)?;
        let reader = BufReader::new(file);
        let variables_file: Self =
            serde_json::from_reader(reader).map_err(ToolError::Json)?;
        Ok(variables_file)
    }

    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        pretty: bool,
    ) -> Result<(), ToolError> {
        if pretty {
            serde_json::to_writer_pretty(&File::create(path)?, self)?;
        } else {
            serde_json::to_writer(&File::create(path)?, self)?;
        }
        Ok(())
    }
}

pub async fn workspace(
    args: &CleanWorkspaceArgs,
    config: &Settings,
    core: &Core,
    client: Client,
) -> miette::Result<CleanupResult, ArgError> {
    check_workspace_identifier(&args.workspace)?;
    let mut workspaces = vec![];
    if let Some(workspace_name) = &args.workspace.workspace_name {
        parse_workspace_name(workspace_name)?;
        let ws = workspace::show_by_name(workspace_name, core, client.clone())
            .await?;
        workspaces.push(ws);
    } else if let Some(workspace_id) = &args.workspace.workspace_id {
        let ws = workspace::show(workspace_id, core, client.clone()).await?;
        workspaces.push(ws);
    } else if let Some(file_path) = &args.workspace.workspace_file {
        let ws = parse_workspace_file(file_path, core, client.clone()).await?;
        workspaces.extend(ws);
    } else if args.workspace.auto_discover_workspaces {
        let ws = workspace::list(true, core, client.clone()).await?;
        workspaces.extend(ws);
    }
    let result = cleanup_workspaces(workspaces, config, core, client).await?;
    if core.save_output {
        result.save(&core.output, config.pretty_output)?;
    }
    Ok(result)
}

async fn cleanup_workspaces(
    workspaces: Vec<Workspace>,
    config: &Settings,
    core: &Core,
    client: Client,
) -> miette::Result<CleanupResult, ToolError> {
    let mut cleanup_result = CleanupResult {
        missing_repositories: None,
        unlisted_variables: None,
        workspaces: workspaces.clone(),
        errors: Errors { parsing_failures: None },
    };
    if config.cleanup.unlisted_variables || config.cleanup.missing_repositories
    {
        // Get the variables for each workspace
        let workspaces_variables =
            variable::list_batch(core, client.clone(), workspaces.clone())
                .await?;
        info!("Cloning workspace repositories.");
        let process_results = process(config, &workspaces_variables)?;
        for entry in &workspaces_variables {
            if let Some(vcs) = &entry.workspace.attributes.vcs_repo {
                info!(
                    "Repo detected for workspace: {}",
                    &entry.workspace.attributes.name.clone().unwrap()
                );
                if config.cleanup.missing_repositories
                    && process_results.missing.contains(vcs)
                {
                    if let Some(m) = &mut cleanup_result.missing_repositories {
                        m.push(entry.workspace.clone());
                    } else {
                        cleanup_result.missing_repositories =
                            Some(vec![entry.workspace.clone()]);
                    }
                }

                if config.cleanup.unlisted_variables {
                    if process_results.failed.contains(vcs)
                        || process_results.missing.contains(vcs)
                    {
                        if let Some(failed) =
                            &mut cleanup_result.errors.parsing_failures
                        {
                            if !failed.repos.contains(vcs) {
                                failed.repos.push(vcs.clone());
                            }
                            failed.workspaces.push(entry.workspace.clone());
                        } else {
                            cleanup_result.errors.parsing_failures =
                                Some(ParsingFailures {
                                    repos: vec![vcs.clone()],
                                    workspaces: vec![entry.workspace.clone()],
                                });
                        }
                    } else {
                        let mut found: Vec<Variable> = vec![];
                        let mut unlisted: Option<UnlistedVariables> = None;
                        let mut variables = entry.variables.clone();
                        // Only keep terraform variables
                        variables.retain(|var| {
                            var.attributes.category
                                == variable::Category::Terraform
                        });
                        for var in &variables {
                            for detected in &process_results.detected_variables
                            {
                                if &detected.vcs == vcs {
                                    for dv in &detected.detected_variables {
                                        if &var.attributes.key == dv {
                                            found.push(var.clone());
                                        }
                                    }
                                }
                            }
                        }
                        debug!("TFC Variables: {:#?}", &entry.variables);
                        debug!("Found Variables: {:#?}", &found);
                        let difference: Vec<_> = variables
                            .into_iter()
                            .filter(|item| !found.contains(item))
                            .collect();
                        debug!("Variable Difference: {:#?}", &difference);
                        if !difference.is_empty() {
                            let mut un = UnlistedVariables {
                                workspace: entry.clone(),
                                unlisted_variables: vec![],
                            };
                            for var in difference {
                                un.unlisted_variables.push(var)
                            }
                            unlisted = Some(un);
                        }

                        if let Some(u) = unlisted {
                            if let Some(v) =
                                &mut cleanup_result.unlisted_variables
                            {
                                v.push(u);
                            } else {
                                cleanup_result.unlisted_variables =
                                    Some(vec![u]);
                            }
                        }
                    }
                }
            }
        }
    }
    info!("{:#?}", &cleanup_result);
    if config.cleanup.dry_run {
        info!("Dry run enabled, no changes will be made.");
    } else {
        info!("Dry run disabled, continuing with cleanup.");
        if config.cleanup.unlisted_variables {
            info!("Removing unlisted variables.");
            if let Some(unlisted) = &cleanup_result.unlisted_variables {
                for entry in unlisted {
                    info!(
                        "Processing unlisted variables for workspace: {}",
                        &entry
                            .workspace
                            .workspace
                            .attributes
                            .name
                            .clone()
                            .unwrap()
                    );
                    for var in &entry.unlisted_variables {
                        info!(
                            "Deleting unlisted variable: {}",
                            &var.attributes.key
                        );
                        variable::delete(
                            &var.id.clone().unwrap(),
                            &entry.workspace.workspace.id,
                            core,
                            client.clone(),
                        )
                        .await?;
                    }
                }
            }
            info!("Finished unlisted variables removal.");
        }
    }
    Ok(cleanup_result)
}
