mod report;

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use tfc_toolset::{
    error::SETTINGS_ERROR, settings::Core, variable, variable_set, workspace,
};
use tfc_toolset_extras::default_client;

const ABOUT: &str =
    "Tool for managing Terraform Cloud variable sets across multiple workspaces";
const ABOUT_CREATE: &str = "Create a new variable set";
const ABOUT_APPLY: &str = "Apply workspaces to variable set";
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
    Apply(ApplyArgs),
    #[clap(about = ABOUT_CREATE)]
    Create(CreateArgs),
}

#[derive(clap::Args, Debug)]
struct ApplyArgs {
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

#[async_std::main]
async fn main() -> miette::Result<()> {
    // Parse cli subcommands and arguments
    let cli = Cli::parse();
    // Get the settings for the run
    let core = Core::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
    // Initialize the logger
    env_logger::Builder::from_env(Env::default().default_filter_or(&core.log))
        .init();
    let client = default_client().into_diagnostic()?;

    match &cli.command {
        Commands::Apply(args) => {
            info!("Applying workspaces to variable set");
            let mut report = report::new();
            report.meta.query = Some(core.workspaces.query.clone());
            report.meta.pagination = Some(core.workspaces.pagination.clone());

            // check for workspaces in args first
            let workspaces = match args.workspaces.clone() {
                Some(workspaces) => {
                    // Get filtered list of workspaces and filter again by args
                    workspace::list_filtered(&core, client.clone())
                        .await
                        .into_diagnostic()?
                        .into_iter()
                        .filter(|ws| workspaces.contains(&ws.attributes.name))
                        .collect()
                }
                None => {
                    // Get filtered list of workspaces
                    workspace::list_filtered(&core, client.clone())
                        .await
                        .into_diagnostic()?
                }
            };

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
        Commands::Create(args) => {
            info!("Creating variable set");
            let mut report = report::new();
            report.meta.query = Some(core.workspaces.query.clone());
            report.meta.pagination = Some(core.workspaces.pagination.clone());

            let skip_workspace_logic = args.skip_workspace_logic;
            let workspaces: Vec<workspace::Workspace>;
            if !skip_workspace_logic {
                // check for workspaces in args first
                workspaces = match args.workspaces.clone() {
                    Some(workspaces) => {
                        // Get filtered list of workspaces and filter again by args
                        workspace::list_filtered(&core, client.clone())
                            .await
                            .into_diagnostic()?
                            .into_iter()
                            .filter(|ws| {
                                workspaces.contains(&ws.attributes.name)
                            })
                            .collect()
                    }
                    None => {
                        // Get filtered list of workspaces
                        workspace::list_filtered(&core, client.clone())
                            .await
                            .into_diagnostic()?
                    }
                };
            } else {
                workspaces = vec![];
            }
            // the vars are in the format of key=value:description:category:hcl:sensitive
            // we need to parse each one into a variable::Vars
            // description, category, hcl, sensitive are all optional and will be None if not provided
            // to skip a field just use a colon e.g. key=value::::true would only set sensitive
            // accepting the default for the rest
            let mut vars: Option<Vec<variable::Vars>> = None;
            if let Some(v) = args.vars.clone() {
                for var in v.iter() {
                    // the string format is key=value:description:category:hcl:sensitive
                    let var_split: Vec<&str> = var.split(':').collect();
                    let key_val = var_split[0].to_string();
                    let key_val_split: Vec<&str> = key_val.split('=').collect();
                    let key = key_val_split[0].to_string();
                    let value = key_val_split[1].to_string();
                    let description = if var_split[1].is_empty() {
                        None
                    } else {
                        Some(var_split[1].to_string())
                    };
                    let category = if var_split[2].is_empty() {
                        None
                    } else {
                        Some(variable::Category::from(var_split[2].to_string()))
                    };
                    let hcl = if var_split[3].is_empty() {
                        None
                    } else {
                        Some(var_split[3].parse::<bool>().unwrap())
                    };
                    let sensitive = if var_split[4].is_empty() {
                        None
                    } else {
                        Some(var_split[4].parse::<bool>().unwrap())
                    };
                    let var = variable::Vars {
                        relationship_type: "vars".to_string(),
                        attributes: variable::Attributes {
                            key,
                            value: Some(value),
                            description,
                            category,
                            hcl,
                            sensitive,
                        },
                    };
                    match vars {
                        Some(ref mut v) => v.push(var),
                        None => vars = Some(vec![var]),
                    }
                }
            }

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
