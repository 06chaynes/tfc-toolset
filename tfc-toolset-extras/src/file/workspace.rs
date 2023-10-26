use crate::{parse_workspace_name, ExtrasError};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tfc_toolset::{error::ToolError, workspace::Attributes};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkspacesFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<Vec<Workspace>>,
}

impl From<Vec<tfc_toolset::workspace::Workspace>> for WorkspacesFile {
    fn from(workspaces: Vec<tfc_toolset::workspace::Workspace>) -> Self {
        let mut workspaces_file = WorkspacesFile { workspaces: None };
        let mut workspaces_vec = Vec::new();
        for workspace in workspaces {
            let entry = Workspace {
                name: None,
                id: Some(workspace.id),
                attributes: Some(workspace.attributes),
            };
            workspaces_vec.push(entry);
        }
        workspaces_file.workspaces = Some(workspaces_vec);
        workspaces_file
    }
}

impl WorkspacesFile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ExtrasError> {
        let file = File::open(path).map_err(ToolError::Io)?;
        let reader = BufReader::new(file);
        let workspaces_file: Self =
            serde_json::from_reader(reader).map_err(ToolError::Json)?;
        if let Some(workspaces) = workspaces_file.clone().workspaces {
            for workspace in workspaces {
                if workspace.name.is_none() && workspace.id.is_none() {
                    return Err(ExtrasError::InvalidWorkspacesFile);
                }
                if let Some(name) = workspace.name {
                    parse_workspace_name(&name)?;
                }
            }
        }
        Ok(workspaces_file)
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
