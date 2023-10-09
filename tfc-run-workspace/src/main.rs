mod report;
mod settings;

use async_std::task::JoinHandle;
use clap::{Parser, Subcommand};
use dashmap::DashMap;
use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use settings::Settings;
use std::collections::BTreeMap;
use std::time::Duration;
use surf::Client;
use tfc_toolset::{
    error::{ToolError, SETTINGS_ERROR},
    filter, run,
    settings::Core,
    workspace,
};
use tfc_toolset_extras::default_client;

const ABOUT: &str =
    "A tool for creating multiple Terraform Cloud runs in parallel with provided specifications";
const ABOUT_PLAN: &str =
    "Queues up plan only runs for the workspaces determined by filters";
const ABOUT_APPLY: &str =
    "Queues up plan and apply runs for the workspaces determined by filters";

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

async fn get_workspaces(
    core: &Core,
    client: Client,
) -> miette::Result<Vec<workspace::Workspace>> {
    let mut workspaces =
        workspace::list(core, client.clone()).await.into_diagnostic()?;

    // Filter the workspaces if query tags have been provided
    if core.workspaces.query.tags.is_some() {
        info!("Filtering workspaces with tags query.");
        filter::tag(&mut workspaces, core).into_diagnostic()?;
    }

    if core.workspaces.query.variables.is_some() {
        // Get the variables for each workspace
        let mut workspaces_variables =
            workspace::variables(core, client, workspaces.clone())
                .await
                .into_diagnostic()?;
        // Filter the workspaces if query variables have been provided
        if core.workspaces.query.variables.is_some() {
            info!("Filtering workspaces with variable query.");
            filter::variable(&mut workspaces_variables, core)
                .into_diagnostic()?;
        }

        workspaces.clear();
        for ws in &workspaces_variables {
            workspaces.push(ws.workspace.clone());
        }
    }
    Ok(workspaces)
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
    let client = default_client().into_diagnostic()?;
    // Match on the cli subcommand
    match &cli.command {
        Commands::Plan => {
            info!("Start Plan Only Runs");
            let mut report = report::new();
            report.meta.query = Some(core.workspaces.query.clone());
            report.meta.pagination = Some(core.workspaces.pagination.clone());

            // Get list of workspaces
            let workspaces = get_workspaces(&core, client.clone()).await?;

            // Queue up plan runs for each workspace respecting the max_concurrent setting
            let max_concurrent = config
                .max_concurrent
                .unwrap_or(settings::MAX_CONCURRENT_DEFAULT.into());
            let attributes = run::Attributes {
                plan_only: Some(true),
                terraform_version: Some(core.terraform_version.clone()),
                ..Default::default()
            };

            let mut queue = BTreeMap::new();
            let running = DashMap::with_capacity(max_concurrent);
            let mut results = Vec::with_capacity(workspaces.len());

            for ws in workspaces.iter() {
                queue.insert(ws.id.clone(), ws.clone());
            }

            while !queue.is_empty() {
                let mut handles = Vec::with_capacity(max_concurrent);
                while running.len() < max_concurrent && !queue.is_empty() {
                    let (id, ws) = queue.pop_first().unwrap();
                    info!("Creating run for workspace: {}", &ws.id);
                    let client = client.clone();
                    let attributes = attributes.clone();
                    let core = core.clone();
                    let ws_id = ws.id.clone();
                    let handle: JoinHandle<
                        Result<(String, String), ToolError>,
                    > = async_std::task::spawn(async move {
                        let run = run::create(
                            &id.clone(),
                            Some(attributes),
                            &core,
                            client.clone(),
                        )
                        .await?;
                        let mut run =
                            run::status(&run.id, &core, client.clone()).await?;
                        info!("Run {} created for workspace {}", &run.id, &id);
                        while !run::COMPLETED_STATUSES.contains(
                            &run.attributes
                                .status
                                .clone()
                                .unwrap_or("unknown".to_string())
                                .as_str(),
                        ) {
                            run = run::status(&run.id, &core, client.clone())
                                .await?;
                            debug!(
                                "Run {} status: {}",
                                &run.id,
                                &run.attributes
                                    .status
                                    .clone()
                                    .unwrap_or("unknown".to_string())
                            );
                            if run::COMPLETED_STATUSES.contains(
                                &run.attributes
                                    .status
                                    .clone()
                                    .unwrap_or("unknown".to_string())
                                    .as_str(),
                            ) {
                                break;
                            }
                            async_std::task::sleep(Duration::from_secs(5))
                                .await;
                        }
                        Ok((
                            run.id.clone(),
                            run.attributes
                                .clone()
                                .status
                                .unwrap_or("unknown".to_string()),
                        ))
                    });
                    running.insert(ws_id, ws);
                    handles.push(handle);
                }
                for handle in handles {
                    let result = handle.await.into_diagnostic()?;
                    let run = result.clone();
                    running.remove(run.0.clone().as_str());
                    results.push(result);
                }
            }

            report.data.workspaces = workspaces;
            //debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
        Commands::Apply => {
            info!("Start Plan and Apply Runs");
            let mut report = report::new();
            report.meta.query = Some(core.workspaces.query.clone());
            report.meta.pagination = Some(core.workspaces.pagination.clone());

            // Get list of workspaces
            let workspaces = get_workspaces(&core, client.clone()).await?;

            // Queue up plan runs for each workspace respecting the max_concurrent setting

            report.data.workspaces = workspaces;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
    }
    Ok(())
}
