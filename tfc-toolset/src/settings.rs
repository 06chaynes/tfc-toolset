use crate::error::ToolError;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

pub const DEFAULT_TERRAFORM_VERSION: &str = "1.5.7";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum Operators {
    Equals,
    NotEquals,
    Contains,
    NotContains,
}

impl FromStr for Operators {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "==" => Ok(Operators::Equals),
            "!=" => Ok(Operators::NotEquals),
            "~=" => Ok(Operators::Contains),
            "!~=" => Ok(Operators::NotContains),
            _ => Err(ToolError::InvalidQueryOperator(s.to_string())),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
    pub key: String,
    pub operator: Operators,
    pub value: String,
}

impl FromStr for Variable {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(ToolError::InvalidVariableQuery(s.to_string()));
        }
        let operator = Operators::from_str(parts[1])?;
        Ok(Variable {
            key: parts[0].to_string(),
            operator,
            value: parts[2].to_string(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    pub operator: Operators,
    pub name: String,
}

impl FromStr for Tag {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ToolError::InvalidTagQuery(s.to_string()));
        }
        let operator = Operators::from_str(parts[0])?;
        Ok(Tag { operator, name: parts[1].to_string() })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pagination {
    pub start_page: String,
    pub max_depth: String,
    pub page_size: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Query {
    pub name: Option<String>,
    pub wildcard_name: Option<String>,
    pub variables: Option<Vec<Variable>>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspaces {
    pub query: Option<Query>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Core {
    pub log: String,
    pub token: String,
    pub org: String,
    pub project: Option<String>,
    pub output: PathBuf,
    pub save_output: bool,
    pub pagination: Pagination,
    pub workspaces: Workspaces,
    pub terraform_version: String,
}

impl Core {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Set defaults
            .set_default("log", "info".to_string())?
            .set_default("token", "".to_string())?
            .set_default("org", "".to_string())?
            .set_default("output", "report.json".to_string())?
            .set_default("save_output", false)?
            .set_default("pagination.start_page", "1".to_string())?
            .set_default("pagination.max_depth", "1".to_string())?
            .set_default("pagination.page_size", "20".to_string())?
            .set_default(
                "terraform_version",
                DEFAULT_TERRAFORM_VERSION.to_string(),
            )?
            .set_default("workspaces.query", None::<String>)?
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("settings.toml").required(false))
            // Add in settings from the environment
            // Eg.. `DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::default())
            .build()?;
        s.try_deserialize()
    }
}
