use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub max_concurrent: Option<usize>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Set defaults
            .set_default("max_concurrent", 10)?
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("settings.toml").required(false))
            // Add in settings from the environment
            // Eg.. `DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::default())
            .build()?;
        s.try_deserialize()
    }
}
