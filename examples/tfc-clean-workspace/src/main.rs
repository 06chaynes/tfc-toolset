mod error;
mod parse;
mod repo;
mod report;
mod settings;

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use settings::Settings;
use tfc_toolset::{
    error::SETTINGS_ERROR, filter, settings::Core, variable, workspace,
};
use tfc_toolset_extras::default_client;

use crate::report::{ParsingFailures, UnlistedVariables};

const ABOUT: &str =
    "Tool for rule based cleanup operations for Terraform workspaces";
const ABOUT_PLAN: &str = "Generates a report that contains information on the actions required to cleanup a workspace based on the provided rules";
const ABOUT_APPLY: &str =
    "Executes the actions described in the previously generated report";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = Some(ABOUT))]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(about = ABOUT_PLAN)]
    Plan,
    #[clap(about = ABOUT_APPLY)]
    Apply,
}

#[async_std::main]
async fn main() -> miette::Result<()> {
    // Parse cli subcommands and arguments
    let cli = Cli::parse();
    // Get the settings for the run
    let core = Core::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
    let config = Settings::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
    // Initialize the logger
    env_logger::Builder::from_env(Env::default().default_filter_or(&core.log))
        .init();
    let client = default_client(None).into_diagnostic()?;
    // Match on the cli subcommand
    match &cli.command {
        Commands::Plan => {
            info!("Start Plan Phase");
            let mut report = report::new();
            report.meta.query = core.workspaces.query.clone();
            report.meta.pagination = Some(core.pagination.clone());

            // Get list of workspaces
            let mut workspaces = workspace::list(false, &core, client.clone())
                .await
                .into_diagnostic()?;

            // Filter the workspaces if query tags have been provided
            if let Some(query) = core.workspaces.query.clone() {
                if query.tags.is_some() {
                    info!("Filtering workspaces with tags query.");
                    filter::workspace::by_tag(&mut workspaces, &core)
                        .into_diagnostic()?;
                }

                if query.variables.is_some()
                    || config.cleanup.unlisted_variables
                    || config.cleanup.missing_repositories
                {
                    // Get the variables for each workspace
                    let mut workspaces_variables =
                        variable::list_batch(&core, client, workspaces.clone())
                            .await
                            .into_diagnostic()?;
                    // Filter the workspaces if query variables have been provided
                    if query.variables.is_some() {
                        info!("Filtering workspaces with variable query.");
                        filter::workspace::by_variable(
                            &mut workspaces_variables,
                            &core,
                        )
                        .into_diagnostic()?;
                    }

                    workspaces.clear();
                    for ws in &workspaces_variables {
                        workspaces.push(ws.workspace.clone());
                    }

                    if config.cleanup.unlisted_variables
                        || config.cleanup.missing_repositories
                    {
                        info!("Cloning workspace repositories.");
                        // First let's clean up the job list to remove duplicates
                        let process_results =
                            repo::process(&config, &workspaces_variables)?;
                        for entry in &workspaces_variables {
                            if let Some(vcs) =
                                &entry.workspace.attributes.vcs_repo
                            {
                                info!(
                                    "Repo detected for workspace: {}",
                                    &entry
                                        .workspace
                                        .attributes
                                        .name
                                        .clone()
                                        .unwrap()
                                );
                                if config.cleanup.missing_repositories
                                    && process_results.missing.contains(vcs)
                                {
                                    if let Some(m) =
                                        &mut report.data.missing_repositories
                                    {
                                        m.push(entry.workspace.clone());
                                    } else {
                                        report.data.missing_repositories =
                                            Some(vec![entry.workspace.clone()]);
                                    }
                                }

                                if config.cleanup.unlisted_variables {
                                    if process_results.failed.contains(vcs)
                                        || process_results.missing.contains(vcs)
                                    {
                                        if let Some(failed) =
                                            &mut report.errors.parsing_failures
                                        {
                                            if !failed.repos.contains(vcs) {
                                                failed.repos.push(vcs.clone());
                                            }
                                            failed
                                                .workspaces
                                                .push(entry.workspace.clone());
                                        } else {
                                            report.errors.parsing_failures =
                                                Some(ParsingFailures {
                                                    repos: vec![vcs.clone()],
                                                    workspaces: vec![entry
                                                        .workspace
                                                        .clone()],
                                                });
                                        }
                                    } else {
                                        let mut found: Vec<variable::Variable> =
                                            vec![];
                                        let mut unlisted: Option<
                                            UnlistedVariables,
                                        > = None;
                                        let mut variables =
                                            entry.variables.clone();
                                        // Only keep terraform variables
                                        variables.retain(|var| {
                                            var.attributes.category
                                                == variable::Category::Terraform
                                        });
                                        for var in &variables {
                                            for detected in &process_results
                                                .detected_variables
                                            {
                                                if &detected.vcs == vcs {
                                                    for dv in &detected
                                                        .detected_variables
                                                    {
                                                        if &var.attributes.key
                                                            == dv
                                                        {
                                                            found.push(
                                                                var.clone(),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        debug!(
                                            "TFC Variables: {:#?}",
                                            &entry.variables
                                        );
                                        debug!(
                                            "Found Variables: {:#?}",
                                            &found
                                        );
                                        let difference: Vec<_> = variables
                                            .into_iter()
                                            .filter(|item| {
                                                !found.contains(item)
                                            })
                                            .collect();
                                        debug!(
                                            "Variable Difference: {:#?}",
                                            &difference
                                        );
                                        if !difference.is_empty() {
                                            let mut un = UnlistedVariables {
                                                workspace: entry.clone(),
                                                unlisted_variables: vec![],
                                            };
                                            for var in difference {
                                                un.unlisted_variables
                                                    .push(var.into())
                                            }
                                            unlisted = Some(un);
                                        }

                                        if let Some(u) = unlisted {
                                            if let Some(v) = &mut report
                                                .data
                                                .unlisted_variables
                                            {
                                                v.push(u);
                                            } else {
                                                report
                                                    .data
                                                    .unlisted_variables =
                                                    Some(vec![u]);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            report.data.workspaces = workspaces;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
        Commands::Apply => {
            info!("Start Apply");
            let report = report::load(&core).into_diagnostic()?;
            debug!("{:#?}", &report);
            if config.cleanup.unlisted_variables {
                info!("Start Unlisted Variables Cleanup Phase");
                if let Some(unlisted) = &report.data.unlisted_variables {
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
                            info!("Deleting unlisted variable: {}", &var.key);
                            variable::delete(
                                &var.id,
                                &entry.workspace.workspace.id,
                                &core,
                                client.clone(),
                            )
                            .await
                            .into_diagnostic()?;
                        }
                    }
                }
                info!("Finished Unlisted Variables Cleanup Phase");
            }
        }
    }
    Ok(())
}
