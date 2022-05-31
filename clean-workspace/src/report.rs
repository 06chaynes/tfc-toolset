use std::fs::File;
use tfc_toolset::{
    error::ToolError,
    settings::{Core, Query},
    variable,
    workspace::{Workspace, WorkspaceVariables},
};

use log::*;
use serde::{Deserialize, Serialize};

// For now need to keep this updated with best effort :)
const REPORT_VERSION: &str = "0.1.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
    pub id: String,
    pub key: String,
}

impl From<variable::Variable> for Variable {
    fn from(item: variable::Variable) -> Self {
        Variable { id: item.id, key: item.attributes.key }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnlistedVariables {
    pub workspace: WorkspaceVariables,
    pub unlisted_variables: Vec<Variable>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Report {
    pub report_version: String,
    pub bin_version: String,
    pub reporter: String,
    pub query: Option<Query>,
    pub missing_repositories: Vec<Workspace>,
    pub unlisted_variables: Vec<UnlistedVariables>,
    pub workspaces: Vec<Workspace>,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            report_version: REPORT_VERSION.to_string(),
            bin_version: env!("CARGO_PKG_VERSION").to_string(),
            reporter: env!("CARGO_PKG_NAME").to_string(),
            query: None,
            missing_repositories: vec![],
            unlisted_variables: vec![],
            workspaces: vec![],
        }
    }
}

pub fn save(config: &Core, report: Report) -> Result<(), ToolError> {
    info!("Saving report to: {}", &config.output);
    match serde_json::to_writer_pretty(&File::create(&config.output)?, &report)
    {
        Ok(_) => {
            info!("Report Saved!");
        }
        Err(e) => {
            error!("Failed to save report!");
            return Err(ToolError::Json(e));
        }
    }
    Ok(())
}
