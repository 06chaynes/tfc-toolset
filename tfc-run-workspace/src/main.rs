mod report;
mod settings;

use async_std::task::JoinHandle;
use clap::{Parser, Subcommand};
use dashmap::DashMap;
use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use report::RunResult;
use settings::Settings;
use std::collections::BTreeMap;
use std::time::Duration;
use surf::Client;
use tfc_toolset::{
    error::{ToolError, SETTINGS_ERROR},
    run,
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
const ABOUT_AUTO_APPLY: &str =
    "Automatically apply the run if the plan is successful";

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
    Apply(ApplyArgs),
}

#[derive(clap::Args, Debug)]
struct ApplyArgs {
    #[arg(long, help = ABOUT_AUTO_APPLY, default_value = "false")]
    pub auto_apply: Option<bool>,
}

struct QueueOptions {
    pub max_concurrent: usize,
    pub max_iterations: usize,
    pub status_check_sleep_seconds: u64,
}

struct QueueResult {
    pub results: Vec<RunResult>,
    pub errors: Vec<RunResult>,
}

async fn work_queue(
    queue: &mut BTreeMap<String, workspace::Workspace>,
    options: QueueOptions,
    attributes: run::Attributes,
    client: Client,
    core: &Core,
) -> Result<QueueResult, ToolError> {
    let running = DashMap::with_capacity(options.max_concurrent);
    let mut results = Vec::with_capacity(queue.len());
    let mut errors = Vec::new();
    while !queue.is_empty() {
        let mut handles = Vec::with_capacity(options.max_concurrent);
        while running.len() < options.max_concurrent && !queue.is_empty() {
            let (id, ws) = queue.pop_first().unwrap();
            info!("Creating run for workspace: {}", &ws.id);
            let client = client.clone();
            let attributes = attributes.clone();
            let will_auto_apply = attributes.auto_apply.unwrap_or(false);
            let core = core.clone();
            let ws_id = ws.id.clone();
            let mut iterations = 0;
            let handle: JoinHandle<Result<RunResult, ToolError>> =
                async_std::task::spawn(async move {
                    let mut run = run::create(
                        &id.clone(),
                        Some(attributes),
                        &core,
                        client.clone(),
                    )
                    .await?;
                    info!(
                        "Run {} created for workspace {}",
                        &run.id,
                        &id.clone()
                    );
                    while !run::COMPLETED_STATUSES.contains(
                        &run.attributes
                            .status
                            .clone()
                            .unwrap_or(run::Status::Unknown),
                    ) {
                        run =
                            run::status(&run.id, &core, client.clone()).await?;
                        let status = run
                            .attributes
                            .status
                            .clone()
                            .unwrap_or(run::Status::Unknown);
                        info!("Run {} status: {}", &run.id, &status);
                        if run::COMPLETED_STATUSES.contains(&status)
                            || !will_auto_apply
                                && status == run::Status::Planned
                        {
                            // If auto_apply is false and status is planned, then we can break out of the loop
                            // because the run will require confirmation before applying
                            // If completed, then we can also break out of the loop
                            break;
                        }
                        iterations += 1;
                        if iterations >= options.max_iterations {
                            error!(
                                    "Run {} for workspace {} has been in status {} too long.",
                                    &run.id, &id.clone(), &status.clone()
                                );
                            if status == run::Status::Pending {
                                error!(
                                    "There is likely previous run pending. Please check the workspace in the UI."
                                );
                            } else {
                                error!(
                                    "This is likely some error. Please check the run in the UI."
                                );
                            }
                            break;
                        }
                        async_std::task::sleep(Duration::from_secs(
                            options.status_check_sleep_seconds,
                        ))
                        .await;
                    }
                    Ok(RunResult {
                        id: run.id,
                        status: run
                            .attributes
                            .status
                            .unwrap_or(run::Status::Unknown)
                            .to_string(),
                        workspace_id: id,
                    })
                });
            running.insert(ws_id, ws);
            handles.push(handle);
        }
        for handle in handles {
            let result = handle.await?;
            running.remove(result.id.clone().as_str());
            if run::ERROR_STATUSES
                .contains(&run::Status::from(result.status.clone()))
            {
                errors.push(result.clone());
            }
            results.push(result);
        }
    }
    Ok(QueueResult { results, errors })
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
    let max_concurrent = config
        .max_concurrent
        .unwrap_or(settings::MAX_CONCURRENT_DEFAULT.into());
    let max_iterations = config
        .max_iterations
        .unwrap_or(settings::MAX_ITERATIONS_DEFAULT.into());
    let status_check_sleep_seconds = config
        .status_check_sleep_seconds
        .unwrap_or(settings::STATUS_CHECK_SLEEP_SECONDS_DEFAULT);
    let client = default_client().into_diagnostic()?;
    // Match on the cli subcommand
    match &cli.command {
        Commands::Plan => {
            info!("Start Plan Only Runs");
            let mut report = report::new();
            report.meta.query = Some(core.workspaces.query.clone());
            report.meta.pagination = Some(core.workspaces.pagination.clone());

            // Get filtered list of workspaces
            let workspaces = workspace::list_filtered(&core, client.clone())
                .await
                .into_diagnostic()?;

            // Queue up plan runs for each workspace respecting the max_concurrent setting
            let attributes = run::Attributes {
                plan_only: Some(true),
                terraform_version: Some(core.terraform_version.clone()),
                ..Default::default()
            };

            let mut queue = BTreeMap::new();

            for ws in workspaces.iter() {
                queue.insert(ws.id.clone(), ws.clone());
            }

            let queue_results = work_queue(
                &mut queue,
                QueueOptions {
                    max_concurrent,
                    max_iterations,
                    status_check_sleep_seconds,
                },
                attributes,
                client.clone(),
                &core,
            )
            .await
            .into_diagnostic()?;

            report.data.workspaces = workspaces;
            report.data.runs = queue_results.results;
            report.errors.runs = queue_results.errors;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
        Commands::Apply(args) => {
            info!("Start Plan and Apply Runs");
            let mut report = report::new();
            report.meta.query = Some(core.workspaces.query.clone());
            report.meta.pagination = Some(core.workspaces.pagination.clone());

            // Get filtered list of workspaces
            let workspaces = workspace::list_filtered(&core, client.clone())
                .await
                .into_diagnostic()?;

            // Queue up plan runs for each workspace respecting the max_concurrent setting
            let auto_apply = args.auto_apply.unwrap_or_default();
            let attributes = run::Attributes {
                auto_apply: Some(auto_apply),
                ..Default::default()
            };

            let mut queue = BTreeMap::new();

            for ws in workspaces.iter() {
                queue.insert(ws.id.clone(), ws.clone());
            }

            let queue_results = work_queue(
                &mut queue,
                QueueOptions {
                    max_concurrent,
                    max_iterations,
                    status_check_sleep_seconds,
                },
                attributes,
                client.clone(),
                &core,
            )
            .await
            .into_diagnostic()?;

            report.data.workspaces = workspaces;
            report.data.runs = queue_results.results;
            report.errors.runs = queue_results.errors;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
    }
    Ok(())
}
