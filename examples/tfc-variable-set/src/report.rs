use std::fs::File;

use log::{error, info};
use serde::{Deserialize, Serialize};
use tfc_report::{Report, Reporter};
use tfc_toolset::{
    error::ToolError,
    settings::{Core, Pagination, Query},
    variable,
    workspace::Workspace,
};

pub type VarSetReport = Report<Meta, Data, Errors>;

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
pub struct Meta {
    pub query: Option<Query>,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub workspaces: Vec<Workspace>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Errors {}

pub fn new() -> VarSetReport {
    Report {
        report_version: REPORT_VERSION.to_string(),
        bin_version: env!("CARGO_PKG_VERSION").to_string(),
        reporter: Reporter::VariableSet,
        meta: Meta { query: None, pagination: None },
        data: Data { workspaces: vec![] },
        errors: Errors {},
    }
}

#[allow(dead_code)]
pub fn load(config: &Core) -> Result<VarSetReport, ToolError> {
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
