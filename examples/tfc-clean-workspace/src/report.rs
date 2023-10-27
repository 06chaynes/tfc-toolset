use std::fs::File;

use log::{error, info};
use serde::{Deserialize, Serialize};
use tfc_report::{Report, Reporter};
use tfc_toolset::{
    error::ToolError,
    settings::{Core, Pagination, Query},
    variable,
    workspace::{VcsRepo, Workspace, WorkspaceVariables},
};

pub type CleanReport = Report<Meta, Data, Errors>;

// For now need to keep this updated with best effort :)
const REPORT_VERSION: &str = "0.1.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
    pub id: String,
    pub key: String,
}

impl From<variable::Variable> for Variable {
    fn from(item: variable::Variable) -> Self {
        Variable { id: item.id.unwrap(), key: item.attributes.key }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnlistedVariables {
    pub workspace: WorkspaceVariables,
    pub unlisted_variables: Vec<Variable>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    pub query: Option<Query>,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub missing_repositories: Option<Vec<Workspace>>,
    pub unlisted_variables: Option<Vec<UnlistedVariables>>,
    pub workspaces: Vec<Workspace>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParsingFailures {
    pub repos: Vec<VcsRepo>,
    pub workspaces: Vec<Workspace>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Errors {
    pub parsing_failures: Option<ParsingFailures>,
}

pub fn new() -> CleanReport {
    Report {
        report_version: REPORT_VERSION.to_string(),
        bin_version: env!("CARGO_PKG_VERSION").to_string(),
        reporter: Reporter::CleanWorkspace,
        meta: Meta { query: None, pagination: None },
        data: Data {
            missing_repositories: None,
            unlisted_variables: None,
            workspaces: vec![],
        },
        errors: Errors { parsing_failures: None },
    }
}

pub fn load(config: &Core) -> Result<CleanReport, ToolError> {
    info!("Loading report from: {}", &config.output.display());
    match serde_json::from_reader(&File::open(&config.output)?) {
        Ok(report) => {
            info!("Report Loaded!");
            Ok(report)
        }
        Err(e) => {
            error!("Failed to load report!");
            Err(ToolError::Json(e))
        }
    }
}
