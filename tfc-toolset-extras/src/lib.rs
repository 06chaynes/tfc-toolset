pub mod report;

use http_cache_surf::{
    CACacheManager, Cache, CacheMode, CacheOptions, HttpCache, HttpCacheOptions,
};
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

pub fn build_retry() -> RetryMiddleware<ExponentialBackoff> {
    RetryMiddleware::new(
        99,
        ExponentialBackoff::builder().build_with_max_retries(10),
        1,
    )
}

pub fn build_cache_options() -> HttpCacheOptions {
    HttpCacheOptions {
        cache_options: Some(CacheOptions {
            shared: false,
            cache_heuristic: 0.0,
            immutable_min_time_to_live: Default::default(),
            ignore_cargo_cult: false,
        }),
        cache_key: None,
    }
}

pub fn default_client() -> Result<Client, ToolError> {
    // Build the http client with a cache, governor, and retry enabled
    Ok(Client::new().with(build_retry()).with(build_governor()?).with(Cache(
        HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::default(),
            options: build_cache_options(),
        },
    )))
}
