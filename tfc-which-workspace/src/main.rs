mod report;

use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use tfc_toolset::{error::SETTINGS_ERROR, settings::Core, workspace};
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

    let client = default_client().into_diagnostic()?;

    // Get filtered list of workspaces
    let workspaces = workspace::list_filtered(&config, client.clone())
        .await
        .into_diagnostic()?;

    let report = report::build(&config, workspaces);
    info!("{:#?}", &report);
    report.save(&config).into_diagnostic()?;
    Ok(())
}
