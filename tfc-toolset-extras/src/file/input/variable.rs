use crate::ExtrasError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tfc_toolset::{
    error::ToolError,
    variable::Attributes,
    workspace::{Workspace, WorkspaceVariables},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub var: Option<String>,
    pub workspace: Option<Workspace>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VariablesFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Variable>>,
}

impl From<Vec<WorkspaceVariables>> for VariablesFile {
    fn from(vec: Vec<WorkspaceVariables>) -> Self {
        let mut variables_file = VariablesFile { variables: None };
        let mut variables_vec = Vec::new();
        for workspace_variables in vec {
            for variable in workspace_variables.variables {
                let entry = Variable {
                    id: variable.id,
                    attributes: Some(variable.attributes),
                    var: None,
                    workspace: Some(workspace_variables.workspace.clone()),
                };
                variables_vec.push(entry);
            }
        }
        variables_file.variables = Some(variables_vec);
        variables_file
    }
}

impl VariablesFile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ExtrasError> {
        let file = File::open(path).map_err(ToolError::Io)?;
        let reader = BufReader::new(file);
        let variables_file: Self =
            serde_json::from_reader(reader).map_err(ToolError::Json)?;
        Ok(variables_file)
    }

    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        pretty: bool,
    ) -> Result<(), ToolError> {
        if pretty {
            serde_json::to_writer_pretty(&File::create(path)?, self)?;
        } else {
            serde_json::to_writer(&File::create(path)?, self)?;
        }
        Ok(())
    }
}
