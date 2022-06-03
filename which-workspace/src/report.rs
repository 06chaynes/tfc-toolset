use serde::{Deserialize, Serialize};
use tfc_toolset::{
    settings::{Core, Pagination, Query},
    workspace::Workspace,
};
use tfc_toolset_extras::report::Report;

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

pub fn new() -> Report<Meta, Data> {
    Report {
        report_version: REPORT_VERSION.to_string(),
        bin_version: env!("CARGO_PKG_VERSION").to_string(),
        reporter: env!("CARGO_PKG_NAME").to_string(),
        meta: Meta { query: None, pagination: None },
        data: Data { workspaces: vec![] },
    }
}

pub fn build(config: &Core, workspaces: Vec<Workspace>) -> Report<Meta, Data> {
    let mut report = new();
    report.meta.query = Some(config.query.clone());
    report.meta.pagination = Some(config.pagination.clone());
    report.data.workspaces = workspaces;
    report
}
