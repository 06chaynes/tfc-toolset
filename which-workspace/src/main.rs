mod report;

use env_logger::Env;
use http_cache_surf::{
    CACacheManager, Cache, CacheMode, CacheOptions, HttpCache,
};
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use surf::Client;
use surf_governor::GovernorMiddleware;
use surf_retry::{ExponentialBackoff, RetryMiddleware};
use tfc_toolset::{
    error::{ToolError, SETTINGS_ERROR},
    filter,
    settings::Core,
    workspace,
};

use crate::report::{Data, Meta};

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

    // Build the http client with a cache, governor, and retry enabled
    let retry = RetryMiddleware::new(
        99,
        ExponentialBackoff::builder().build_with_max_retries(10),
        1,
    );
    let client = Client::new()
        .with(retry)
        .with(build_governor().into_diagnostic()?)
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::default(),
            options: Some(CacheOptions {
                shared: false,
                cache_heuristic: 0.0,
                immutable_min_time_to_live: Default::default(),
                ignore_cargo_cult: false,
            }),
        }));

    // Get list of workspaces
    let mut workspaces =
        workspace::get_workspaces(&config, client.clone()).await?;

    // Filter the workspaces if query tags have been provided
    if config.query.tags.is_some() {
        info!("Filtering workspaces with tags query.");
        filter::tag(&mut workspaces, &config)?;
    }

    if config.query.variables.is_some() {
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

    let report = report::Report {
        meta: Meta {
            query: Some(config.query.clone()),
            pagination: Some(config.pagination.clone()),
        },
        data: Data { workspaces },
        ..Default::default()
    };
    info!("{:#?}", &report);
    report::save(&config, report)?;
    Ok(())
}
