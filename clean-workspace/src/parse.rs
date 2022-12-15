use crate::settings::Settings;
use std::fs;
use tfc_toolset::{error::ToolError, workspace::VcsRepo};

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
    pub vcs: VcsRepo,
    pub detected_variables: Vec<String>,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false)
}

fn is_tf(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.ends_with("tf")).unwrap_or(false)
}

pub fn tf_repo(
    _config: &Settings,
    walker: IntoIter,
    vcs: &VcsRepo,
) -> Result<ParseResult, ToolError> {
    let mut detected_variables: Vec<String> = vec![];
    for file in walker
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
        .filter(is_tf)
    {
        info!("Parsing file: {}", file.path().display());
        match hcl::from_str::<TestVariable>(&fs::read_to_string(file.path())?) {
            Ok(v) => {
                info!("{:#?}", &v);
                if let Some(value) = v.variable {
                    for (key, _value) in value.as_object().unwrap() {
                        detected_variables.push(key.clone());
                    }
                }
            }
            Err(_e) => {
                match hcl::from_str::<Variable>(&fs::read_to_string(
                    file.path(),
                )?) {
                    Ok(value) => {
                        for (key, _value) in value.variable.as_object().unwrap()
                        {
                            detected_variables.push(key.clone());
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
    Ok(ParseResult { vcs: vcs.clone(), detected_variables })
}
