mod report;
mod settings;

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use settings::Settings;
use surf::Client;
use tfc_toolset::{error::SETTINGS_ERROR, filter, settings::Core, workspace};
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
    let _config = Settings::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
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

            report.data.workspaces = workspaces;
            debug!("{:#?}", &report);
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
