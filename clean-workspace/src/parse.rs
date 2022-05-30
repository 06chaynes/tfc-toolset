use crate::{report::UnlistedVariables, settings::Settings};
use std::fs;
use tfc_toolset::{
    error::ToolError,
    variable,
    workspace::{Workspace, WorkspaceVariables},
};

use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use walkdir::{DirEntry, IntoIter};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestVariable {
    pub variable: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
    pub variable: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParseResult {
    pub workspace: Workspace,
    pub unlisted_variables: Option<UnlistedVariables>,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false)
}

fn is_tf(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.ends_with("tf")).unwrap_or(false)
}

pub fn tf(
    _config: &Settings,
    walker: IntoIter,
    workspace: &WorkspaceVariables,
) -> Result<Option<UnlistedVariables>, ToolError> {
    let mut unlisted: Option<UnlistedVariables> = None;
    let tfc_variables: Vec<variable::Variable> = workspace.variables.clone();
    let mut found: Vec<variable::Variable> = vec![];
    for file in walker
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
        .filter(is_tf)
    {
        info!("Parsing file: {}", file.path().display());
        match hcl::from_str::<TestVariable>(&fs::read_to_string(file.path())?) {
            Ok(v) => {
                info!("{:#?}", &v);
            }
            Err(_e) => {
                match hcl::from_str::<Variable>(&fs::read_to_string(
                    file.path(),
                )?) {
                    Ok(value) => {
                        for var in &tfc_variables {
                            for (key, _value) in
                                value.variable.as_object().unwrap()
                            {
                                if var.attributes.key == *key {
                                    found.push(var.clone());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error parsing file: {}", file.path().display());
                        warn!("{:#?}", e);
                    }
                }
            }
        }
    }
    debug!("TFC Variables: {:#?}", &tfc_variables);
    debug!("Found Variables: {:#?}", &found);
    let difference: Vec<_> = tfc_variables
        .into_iter()
        .filter(|item| !found.contains(item))
        .collect();
    debug!("Variable Difference: {:#?}", &difference);
    if !difference.is_empty() {
        let mut un = UnlistedVariables {
            workspace: workspace.clone(),
            unlisted_variables: vec![],
        };
        for var in difference {
            un.unlisted_variables.push(var.into())
        }
        unlisted = Some(un);
    }
    Ok(unlisted)
}
