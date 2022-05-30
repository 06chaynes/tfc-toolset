use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cleanup {
    pub unlisted_variables: bool,
    pub missing_repositories: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Repositories {
    pub git_dir: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub cleanup: Cleanup,
    pub repositories: Repositories,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Set defaults
            .set_default("cleanup.unlisted_variables", true)?
            .set_default("cleanup.missing_repositories", true)?
            .set_default("repositories.git_dir", "./git_repos".to_string())?
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("settings.toml").required(false))
            // Add in settings from the environment
            // Eg.. `DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::default())
            .build()?;
        s.try_deserialize()
    }
}
