use serde::{Deserialize, Serialize};
use tfc_report::{Report, Reporter};
use tfc_toolset::{
    settings::{Core, Pagination, Query},
    workspace::Workspace,
};

pub type WhichReport = Report<Meta, Data, Errors>;

// For now need to keep this updated with best effort :)
const REPORT_VERSION: &str = "0.1.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    pub workspaces: Workspaces,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspaces {
    pub query: Option<Query>,
    pub pagination: Option<Pagination>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    pub workspaces: Vec<Workspace>,
}

// Could probably actually put stuff here
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Errors {}

pub fn new() -> WhichReport {
    Report {
        report_version: REPORT_VERSION.to_string(),
        bin_version: env!("CARGO_PKG_VERSION").to_string(),
        reporter: Reporter::WhichWorkspace,
        meta: Meta { workspaces: Workspaces { query: None, pagination: None } },
        data: Data { workspaces: vec![] },
        errors: Errors {},
    }
}

pub fn build(config: &Core, workspaces: Vec<Workspace>) -> WhichReport {
    let mut report = new();
    report.meta.workspaces.query = config.workspaces.query.clone();
    report.meta.workspaces.pagination = Some(config.pagination.clone());
    report.data.workspaces = workspaces;
    report
}
