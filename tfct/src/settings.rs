use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

pub const MAX_CONCURRENT_DEFAULT: u16 = 10;
pub const MAX_ITERATIONS_DEFAULT: u16 = 10;
pub const STATUS_CHECK_SLEEP_SECONDS_DEFAULT: u64 = 5;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub run: Run,
    pub pretty_output: bool,
    pub cleanup: Cleanup,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Run {
    pub max_concurrent: Option<usize>,
    pub max_iterations: Option<usize>,
    pub status_check_sleep_seconds: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Repositories {
    pub git_dir: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Cleanup {
    pub dry_run: bool,
    pub unlisted_variables: bool,
    pub missing_repositories: bool,
    pub repositories: Repositories,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Set defaults
            .set_default("pretty_output", false)?
            .set_default("run.max_concurrent", MAX_CONCURRENT_DEFAULT)?
            .set_default("run.max_iterations", MAX_ITERATIONS_DEFAULT)?
            .set_default(
                "run.status_check_sleep_seconds",
                STATUS_CHECK_SLEEP_SECONDS_DEFAULT,
            )?
            .set_default("cleanup.dry_run", true)?
            .set_default("cleanup.unlisted_variables", true)?
            .set_default("cleanup.missing_repositories", false)?
            .set_default(
                "cleanup.repositories.git_dir",
                dirs::cache_dir()
                    .unwrap_or("./".into())
                    .join("tfc-toolset")
                    .to_str(),
            )?
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("settings.toml").required(false))
            // Add in settings from the environment
            // Eg.. `DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::default())
            .build()?;
        s.try_deserialize()
    }
}
