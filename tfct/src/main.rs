#![allow(dead_code)]
mod cli;
mod error;
mod settings;

use clap::Parser;
use cli::{override_config, override_core};
use cli::{
    run::{self, RunCmds},
    tag::{self, TagCmds},
    variable::{self, VariableCmds},
    variable_set::{self, VariableSetCmds},
    workspace::{self, WorkspaceCmds},
    Cli, Commands,
};
use env_logger::Env;
use miette::{IntoDiagnostic, WrapErr};
use settings::Settings;
use tfc_toolset::{error::SETTINGS_ERROR, settings::Core};
use tfc_toolset_extras::default_client;

#[async_std::main]
async fn main() -> miette::Result<()> {
    // Parse cli subcommands and arguments
    let cli = Cli::parse();
    // Get the settings for the run
    let mut core = Core::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
    let mut config =
        Settings::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
    // Override the configs with any cli arguments
    override_core(&mut core, &cli.root)?;
    override_config(&mut config, &cli.root);
    // Initialize the logger
    env_logger::Builder::from_env(Env::default().default_filter_or(&core.log))
        .init();
    let client = default_client(None).into_diagnostic()?;
    // Match on the cli subcommand
    match &cli.command {
        Commands::Workspace(workspace_cmd) => match &workspace_cmd.command {
            WorkspaceCmds::Create(args) => {
                workspace::create(args, &core, &config, client.clone()).await?;
            }
            WorkspaceCmds::Update(args) => {
                workspace::update(args, &core, &config, client.clone()).await?;
            }
            WorkspaceCmds::Delete(args) => {
                workspace::delete(args, &core, client.clone()).await?;
            }
            WorkspaceCmds::List(args) => {
                workspace::list(args, &core, &config, client.clone()).await?;
            }
            WorkspaceCmds::Show(args) => {
                workspace::show(args, &core, &config, client.clone()).await?;
            }
        },
        Commands::Tag(tag_cmd) => match &tag_cmd.command {
            TagCmds::Add(args) => {
                tag::add(args, &core, client.clone()).await?;
            }
            TagCmds::List(args) => {
                tag::list(args, &core, &config, client.clone()).await?;
            }
            TagCmds::Remove(args) => {
                tag::remove(args, &core, client.clone()).await?;
            }
        },
        Commands::Variable(variable_cmd) => match &variable_cmd.command {
            VariableCmds::Create(args) => {
                variable::create(args, &core, &config, client.clone()).await?;
            }
            VariableCmds::Delete(args) => {
                variable::delete(args, &core, client.clone()).await?;
            }
            VariableCmds::List(args) => {
                variable::list(args, &core, &config, client.clone()).await?;
            }
        },
        Commands::VariableSet(variable_set_cmd) => {
            match &variable_set_cmd.command {
                VariableSetCmds::Apply(args) => {
                    variable_set::apply(args, &core, client.clone()).await?;
                }
                VariableSetCmds::Remove(args) => {
                    variable_set::remove(args, &core, client.clone()).await?;
                }
            }
        }
        Commands::Run(run_cmd) => match &run_cmd.command {
            RunCmds::Status(args) => {
                run::status(args, &core, client.clone()).await?;
            }
            RunCmds::Plan(args) => {
                let config = Settings::new()
                    .into_diagnostic()
                    .wrap_err(SETTINGS_ERROR)?;
                run::plan(args, &config, &core, client.clone()).await?;
            }
            RunCmds::Apply(args) => {
                let config = Settings::new()
                    .into_diagnostic()
                    .wrap_err(SETTINGS_ERROR)?;
                run::apply(args, &config, &core, client.clone()).await?;
            }
        },
    }
    Ok(())
}
