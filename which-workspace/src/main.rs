mod report;

use env_logger::Env;
use http_cache_surf::{
    CACacheManager, Cache, CacheMode, CacheOptions, HttpCache,
};
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use surf::Client;
use surf_governor::GovernorMiddleware;
use tfc_toolset::{
    error::{ToolError, SETTINGS_ERROR},
    filter,
    settings::Core,
    workspace,
};

fn build_governor() -> Result<GovernorMiddleware, ToolError> {
    match GovernorMiddleware::per_second(30) {
        Ok(g) => Ok(g),
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

#[async_std::main]
async fn main() -> miette::Result<()> {
    // Get the settings for the run
    let config = Core::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;

    // Initialize the logger
    env_logger::Builder::from_env(
        Env::default().default_filter_or(&config.log),
    )
    .init();

    // Build the http client with a cache and governor enabled
    let client = Client::new().with(build_governor().into_diagnostic()?).with(
        Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::default(),
            options: Some(CacheOptions {
                shared: false,
                cache_heuristic: 0.0,
                immutable_min_time_to_live: Default::default(),
                ignore_cargo_cult: false,
            }),
        }),
    );

    // Get list of workspaces
    let mut workspaces =
        workspace::get_workspaces(&config, client.clone()).await?;

    // Filter the workspaces if query tags have been provided
    if config.query.tags.is_some() {
        info!("Filtering workspaces with tags query.");
        filter::tag(&mut workspaces, &config)?;
    }

    // Get the variables for each workspace
    let mut workspaces_variables =
        workspace::get_workspaces_variables(&config, client, workspaces)
            .await?;
    // Filter the workspaces if query variables have been provided
    if config.query.variables.is_some() {
        info!("Filtering workspaces with variable query.");
        filter::variable(&mut workspaces_variables, &config)?;
    }

    let report = report::Report {
        query: Some(config.query.clone()),
        workspaces: workspaces_variables
            .iter()
            .map(|entry| entry.workspace.clone())
            .collect(),
        ..Default::default()
    };
    info!("{:#?}", &report);
    report::save(&config, report)?;
    Ok(())
}
