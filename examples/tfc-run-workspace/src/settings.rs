use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

pub const MAX_CONCURRENT_DEFAULT: u16 = 10;
pub const MAX_ITERATIONS_DEFAULT: u16 = 10;
pub const STATUS_CHECK_SLEEP_SECONDS_DEFAULT: u64 = 5;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub max_concurrent: Option<usize>,
    pub max_iterations: Option<usize>,
    pub status_check_sleep_seconds: Option<u64>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Set defaults
            .set_default("max_concurrent", MAX_CONCURRENT_DEFAULT)?
            .set_default("max_iterations", MAX_ITERATIONS_DEFAULT)?
            .set_default(
                "status_check_sleep_seconds",
                STATUS_CHECK_SLEEP_SECONDS_DEFAULT,
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
