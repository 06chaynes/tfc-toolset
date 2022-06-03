pub mod report;

use http_cache_surf::{
    CACacheManager, Cache, CacheMode, CacheOptions, HttpCache,
};
use miette::IntoDiagnostic;
use surf::Client;
use surf_governor::GovernorMiddleware;
use surf_retry::{ExponentialBackoff, RetryMiddleware};
use tfc_toolset::error::ToolError;

pub fn build_governor() -> Result<GovernorMiddleware, ToolError> {
    match GovernorMiddleware::per_second(30) {
        Ok(g) => Ok(g),
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub fn default_client() -> miette::Result<Client> {
    // Build the http client with a cache, governor, and retry enabled
    let retry = RetryMiddleware::new(
        99,
        ExponentialBackoff::builder().build_with_max_retries(10),
        1,
    );
    Ok(Client::new()
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
        })))
}
