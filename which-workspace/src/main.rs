mod report;

use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use tfc_toolset::{error::SETTINGS_ERROR, filter, settings::Core, workspace};
use tfc_toolset_extras::default_client;

#[async_std::main]
async fn main() -> miette::Result<()> {
    // Get the settings for the run
    let config = Core::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;

    // Initialize the logger
    env_logger::Builder::from_env(
        Env::default().default_filter_or(&config.log),
    )
    .init();

    let client = default_client()?;

    // Get list of workspaces
    let mut workspaces =
        workspace::get_workspaces(&config, client.clone()).await?;

    // Filter the workspaces if query tags have been provided
    if config.workspaces.query.tags.is_some() {
        info!("Filtering workspaces with tags query.");
        filter::tag(&mut workspaces, &config)?;
    }

    if config.workspaces.query.variables.is_some() {
        // Get the variables for each workspace
        let mut workspaces_variables =
            workspace::get_workspaces_variables(&config, client, workspaces)
                .await?;
        // Filter the workspaces if query variables have been provided
        info!("Filtering workspaces with variable query.");
        filter::variable(&mut workspaces_variables, &config)?;
        workspaces = workspaces_variables
            .iter()
            .map(|entry| entry.workspace.clone())
            .collect();
    }

    let report = report::build(&config, workspaces);
    info!("{:#?}", &report);
    report.save(&config)?;
    Ok(())
}
