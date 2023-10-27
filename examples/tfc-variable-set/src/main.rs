mod report;

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use std::str::FromStr;
use surf::Client;
use tfc_toolset::error::ToolError;
use tfc_toolset::workspace::Workspace;
use tfc_toolset::{
    error::SETTINGS_ERROR, settings::Core, variable, variable_set, workspace,
};
use tfc_toolset_extras::default_client;

const ABOUT: &str =
    "Tool for managing Terraform Cloud variable sets across multiple workspaces";
const ABOUT_CREATE: &str = "Create a new variable set";
const ABOUT_APPLY: &str = "Apply workspaces to variable set";
const ABOUT_REMOVE: &str = "Remove workspaces from variable set";
const ABOUT_VARSET_ID: &str = "The ID for the Variable Set to be manipulated";
const ABOUT_WORKSPACES: &str = "The workspaces to apply to the variable set";
const ABOUT_VARS: &str = "The variables to apply to the variable set, \
    in the format of 'key=value:description:category:hcl:sensitive'";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = Some(ABOUT))]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(about = ABOUT_APPLY)]
    Apply(ApplyRemoveArgs),
    #[clap(about = ABOUT_CREATE)]
    Create(CreateArgs),
    #[clap(about = ABOUT_REMOVE)]
    Remove(ApplyRemoveArgs),
}

#[derive(clap::Args, Debug)]
struct ApplyRemoveArgs {
    #[arg(short, long, help = ABOUT_VARSET_ID)]
    pub variable_set_id: String,
    #[arg(short, long, help = ABOUT_WORKSPACES)]
    pub workspaces: Option<Vec<String>>,
}

#[derive(clap::Args, Debug)]
struct CreateArgs {
    #[arg(short, long, help = "The name of the variable set")]
    pub name: String,
    #[arg(short, long, help = "The description of the variable set")]
    pub description: String,
    #[arg(
        short,
        long,
        help = "Whether the variable set is global",
        default_value = "false"
    )]
    pub global: Option<bool>,
    #[arg(short, long, help = ABOUT_WORKSPACES)]
    pub workspaces: Option<Vec<String>>,
    #[arg(short, long, help = "The projects to apply to the variable set")]
    pub projects: Option<Vec<String>>,
    #[arg(short, long, help = ABOUT_VARS)]
    pub vars: Option<Vec<String>>,
    #[arg(
        short,
        long,
        action,
        help = "Skip the logic for discovering and filtering workspaces",
        default_value = "false",
        required = false
    )]
    pub skip_workspace_logic: bool,
}

async fn determine_workspaces(
    skip_workspace_logic: bool,
    workspaces: Option<Vec<String>>,
    core: &Core,
    client: Client,
) -> Result<Vec<Workspace>, ToolError> {
    if skip_workspace_logic {
        return Ok(vec![]);
    }
    match workspaces {
        Some(workspaces) => {
            // Get filtered list of workspaces and filter again by args
            Ok(workspace::list(true, core, client)
                .await?
                .into_iter()
                .filter(|ws| {
                    workspaces.contains(&ws.attributes.name.clone().unwrap())
                })
                .collect())
        }
        None => {
            // Get filtered list of workspaces
            Ok(workspace::list(true, core, client).await?)
        }
    }
}

#[async_std::main]
async fn main() -> miette::Result<()> {
    // Parse cli subcommands and arguments
    let cli = Cli::parse();
    // Get the settings for the run
    let core = Core::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
    // Initialize the logger
    env_logger::Builder::from_env(Env::default().default_filter_or(&core.log))
        .init();
    let client = default_client(None).into_diagnostic()?;

    match &cli.command {
        Commands::Apply(args) => {
            info!("Applying workspaces to variable set");
            let mut report = report::new();
            report.meta.query = core.workspaces.query.clone();
            report.meta.pagination = Some(core.pagination.clone());

            // check for workspaces in args first
            let workspaces = determine_workspaces(
                false,
                args.workspaces.clone(),
                &core,
                client.clone(),
            )
            .await
            .into_diagnostic()?;

            // Get variable set id
            let variable_set_id = args.variable_set_id.clone();

            // Apply workspaces to variable set
            variable_set::apply_workspace(
                &variable_set_id,
                workspaces.clone(),
                &core,
                client.clone(),
            )
            .await
            .into_diagnostic()?;

            report.data.workspaces = workspaces;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
        Commands::Remove(args) => {
            info!("Removing workspaces from variable set");
            let mut report = report::new();
            report.meta.query = core.workspaces.query.clone();
            report.meta.pagination = Some(core.pagination.clone());

            // check for workspaces in args first
            let workspaces = determine_workspaces(
                false,
                args.workspaces.clone(),
                &core,
                client.clone(),
            )
            .await
            .into_diagnostic()?;

            // Get variable set id
            let variable_set_id = args.variable_set_id.clone();

            // Remove workspaces from variable set
            variable_set::remove_workspace(
                &variable_set_id,
                workspaces.clone(),
                &core,
                client.clone(),
            )
            .await
            .into_diagnostic()?;

            report.data.workspaces = workspaces;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
        Commands::Create(args) => {
            info!("Creating variable set");
            let mut report = report::new();
            report.meta.query = core.workspaces.query.clone();
            report.meta.pagination = Some(core.pagination.clone());

            let workspaces = determine_workspaces(
                args.skip_workspace_logic,
                args.workspaces.clone(),
                &core,
                client.clone(),
            )
            .await
            .into_diagnostic()?;

            let mut vars: Option<Vec<variable::Variable>> = None;
            if let Some(v) = args.vars.clone() {
                for var_string in v.iter() {
                    let var = variable::Variable::from_str(var_string)
                        .into_diagnostic()?;
                    match vars {
                        Some(ref mut vars) => vars.push(var),
                        None => vars = Some(vec![var]),
                    }
                }
            }
            debug!("vars: {:#?}", vars);

            // Create variable set
            variable_set::create(
                variable_set::VarSetOptions {
                    name: args.name.clone(),
                    description: args.description.clone(),
                    global: args.global,
                    workspaces: Some(workspaces.clone()),
                    projects: args.projects.clone(),
                    vars,
                },
                &core,
                client.clone(),
            )
            .await
            .into_diagnostic()?;

            report.data.workspaces = workspaces;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
    }
    Ok(())
}
