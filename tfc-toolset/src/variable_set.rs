use crate::{
    error::{surf_to_tool_error, ToolError},
    settings::Core,
    variable::Vars,
    workspace::Workspace,
    BASE_URL,
};
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use surf::{http::Method, Client, RequestBuilder};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attributes {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspaces {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkspacesOuter {
    pub data: Vec<Workspaces>,
}

impl From<Vec<Workspace>> for WorkspacesOuter {
    fn from(workspaces: Vec<Workspace>) -> Self {
        Self {
            data: workspaces
                .into_iter()
                .map(|ws| Workspaces {
                    relationship_type: "workspaces".to_string(),
                    id: ws.id,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Projects {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectsOuter {
    pub data: Vec<Projects>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VarsOuter {
    pub data: Vec<Vars>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Relationships {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<WorkspacesOuter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<ProjectsOuter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vars: Option<VarsOuter>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct VarSetRequest {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub attributes: Attributes,
    pub relationships: Relationships,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VarSetRequestOuter {
    pub data: VarSetRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VarSetOptions {
    pub name: String,
    pub description: String,
    pub global: Option<bool>,
    pub workspaces: Option<Vec<Workspace>>,
    pub projects: Option<Vec<String>>,
    pub vars: Option<Vec<Vars>>,
}

impl VarSetRequestOuter {
    pub fn new(options: VarSetOptions) -> Self {
        Self {
            data: VarSetRequest {
                relationship_type: "vars".to_string(),
                attributes: Attributes {
                    name: options.name.to_string(),
                    description: options.description.to_string(),
                    global: options.global,
                },
                relationships: Relationships {
                    workspaces: options.workspaces.map(|ws| WorkspacesOuter {
                        data: ws
                            .into_iter()
                            .map(|ws| Workspaces {
                                relationship_type: "workspaces".to_string(),
                                id: ws.id,
                            })
                            .collect(),
                    }),
                    projects: options.projects.map(|p| ProjectsOuter {
                        data: p
                            .into_iter()
                            .map(|p| Projects {
                                relationship_type: "projects".to_string(),
                                id: p,
                            })
                            .collect(),
                    }),
                    vars: options.vars.map(|v| VarsOuter { data: v }),
                },
            },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ApplyVarSet {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub id: String,
}

#[derive(Clone, Debug, Serialize)]
struct ApplyVarSetOuter {
    pub data: Vec<ApplyVarSet>,
}

impl From<Vec<Workspace>> for ApplyVarSetOuter {
    fn from(workspaces: Vec<Workspace>) -> Self {
        Self {
            data: workspaces
                .into_iter()
                .map(|ws| ApplyVarSet {
                    relationship_type: "workspaces".to_string(),
                    id: ws.id,
                })
                .collect(),
        }
    }
}

pub async fn create(
    options: VarSetOptions,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    let url = Url::parse(&format!(
        "{}/organizations/{}/varsets",
        BASE_URL, config.org
    ))?;
    let req = RequestBuilder::new(Method::Post, url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Content-Type", "application/vnd.api+json")
        .body(json!(VarSetRequestOuter::new(options)))
        .build();
    let mut res = client.send(req).await.map_err(surf_to_tool_error)?;
    if res.status().is_success() {
        info!("Successfully created variable set");
    } else {
        error!("Failed to create variable set");
        let error = res.body_string().await.map_err(surf_to_tool_error)?;
        return Err(ToolError::General(anyhow::anyhow!(error)));
    }
    Ok(())
}

pub async fn apply_workspace(
    variable_set_id: &str,
    workspaces: Vec<Workspace>,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    let url = Url::parse(&format!(
        "{}/varsets/{}/relationships/workspaces",
        BASE_URL, variable_set_id
    ))?;
    let req = RequestBuilder::new(Method::Post, url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Content-Type", "application/vnd.api+json")
        .body(json!(ApplyVarSetOuter::from(workspaces)))
        .build();
    let mut res = client.send(req).await.map_err(surf_to_tool_error)?;
    if res.status().is_success() {
        info!("Successfully applied workspaces to variable set");
    } else {
        error!("Failed to apply workspaces to variable set");
        let error = res.body_string().await.map_err(surf_to_tool_error)?;
        return Err(ToolError::General(anyhow::anyhow!(error)));
    }
    Ok(())
}

pub async fn remove_workspace(
    variable_set_id: &str,
    workspaces: Vec<Workspace>,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    let url = Url::parse(&format!(
        "{}/varsets/{}/relationships/workspaces",
        BASE_URL, variable_set_id
    ))?;
    let req = RequestBuilder::new(Method::Delete, url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Content-Type", "application/vnd.api+json")
        .body(json!(WorkspacesOuter::from(workspaces)))
        .build();
    let mut res = client.send(req).await.map_err(surf_to_tool_error)?;
    if res.status().is_success() {
        info!("Successfully removed workspace from variable set");
    } else {
        error!("Failed to remove workspace from variable set");
        let error = res.body_string().await.map_err(surf_to_tool_error)?;
        return Err(ToolError::General(anyhow::anyhow!(error)));
    }
    Ok(())
}
