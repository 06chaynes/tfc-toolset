use serde::{Deserialize, Serialize};
use tfc_toolset::{
    settings::{Pagination, Query},
    variable,
    workspace::{Workspace, WorkspaceVariables},
};
use tfc_toolset_extras::report::{Report, Reporter};

pub type CleanReport = Report<Meta, Data>;

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
    }
}
