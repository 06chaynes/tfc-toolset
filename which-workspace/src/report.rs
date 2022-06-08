use serde::{Deserialize, Serialize};
use tfc_toolset::{
    settings::{Core, Pagination, Query},
    workspace::Workspace,
};
use tfc_toolset_extras::report::{Report, Reporter};

pub type WhichReport = Report<Meta, Data, Errors>;

// For now need to keep this updated with best effort :)
const REPORT_VERSION: &str = "0.1.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
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
        meta: Meta { query: None, pagination: None },
        data: Data { workspaces: vec![] },
        errors: Errors {},
    }
}

pub fn build(config: &Core, workspaces: Vec<Workspace>) -> WhichReport {
    let mut report = new();
    report.meta.query = Some(config.query.clone());
    report.meta.pagination = Some(config.pagination.clone());
    report.data.workspaces = workspaces;
    report
}
