use crate::{
    build_request,
    error::ToolError,
    settings::Core,
    workspace::{Workspace, WorkspaceVariables},
    BASE_URL,
};
use async_scoped::AsyncStdScope;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};
use surf::{http::Method, Client};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Variable {
    #[serde(rename = "type")]
    pub relationship_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub attributes: Attributes,
}

// the vars are in the format of key=value:description:category:hcl:sensitive
// we need to parse each one into a variable::Variable
// description, category, hcl, sensitive are all optional and will be None if not provided
// to skip a field just use a colon e.g. key=value::::true would only set key, value, and sensitive
// accepting the default for the rest
impl FromStr for Variable {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(':') {
            let var_split: Vec<&str> = s.split(':').collect();
            let key_val = var_split[0].to_string();
            let key_val_split: Vec<&str> = key_val.split('=').collect();
            let key = key_val_split[0].to_string();
            let value = key_val_split[1].to_string();
            let description = if var_split[1].is_empty() {
                None
            } else {
                Some(var_split[1].to_string())
            };
            let category = if var_split[2].is_empty() {
                Category::default()
            } else {
                Category::from(var_split[2].to_string())
            };
            let hcl = if var_split[3].is_empty() {
                None
            } else {
                Some(var_split[3].parse::<bool>()?)
            };
            let sensitive = if var_split[4].is_empty() {
                None
            } else {
                Some(var_split[4].parse::<bool>()?)
            };
            Ok(Variable {
                relationship_type: "vars".to_string(),
                id: None,
                attributes: Attributes {
                    key,
                    value: Some(value),
                    description,
                    category,
                    hcl,
                    sensitive,
                },
            })
        } else {
            let key_val_split = s.split('=').collect::<Vec<&str>>();
            let key = key_val_split[0].to_string();
            let value = key_val_split[1].to_string();
            Ok(Variable {
                relationship_type: "vars".to_string(),
                id: None,
                attributes: Attributes {
                    key,
                    value: Some(value),
                    description: None,
                    category: Category::default(),
                    hcl: None,
                    sensitive: None,
                },
            })
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    #[default]
    Terraform,
    Env,
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Terraform => write!(f, "terraform"),
            Category::Env => write!(f, "env"),
        }
    }
}

impl From<String> for Category {
    fn from(s: String) -> Self {
        match s.as_str() {
            "terraform" => Category::Terraform,
            "env" => Category::Env,
            _ => Category::Terraform,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Attributes {
    pub key: String,
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hcl: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VariablesOuter {
    pub data: Vec<Variable>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct VariableOuter {
    pub data: Variable,
}

pub async fn create(
    workspace_id: &str,
    var: Variable,
    config: &Core,
    client: Client,
) -> Result<Variable, ToolError> {
    info!(
        "Creating variable: {} in workspace: {}",
        var.attributes.key, workspace_id
    );
    let url =
        Url::parse(&format!("{}/workspaces/{}/vars/", BASE_URL, workspace_id))?;
    let req = build_request(
        Method::Post,
        url,
        config,
        Some(json!(VariableOuter { data: var })),
    );
    match client.send(req).await {
        Ok(mut res) => {
            if res.status().is_success() {
                let body: VariableOuter =
                    res.body_json().await.map_err(|e| e.into_inner())?;
                Ok(body.data)
            } else {
                error!("Failed to create variable :(");
                let error =
                    res.body_string().await.map_err(|e| e.into_inner())?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}
pub async fn list(
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> Result<Vec<Variable>, ToolError> {
    let url =
        Url::parse(&format!("{}/workspaces/{}/vars/", BASE_URL, workspace_id))?;
    let req = build_request(Method::Get, url, config, None);
    match client.send(req).await {
        Ok(mut res) => {
            if res.status().is_success() {
                info!("Successfully retrieved variables!");
                let body: VariablesOuter =
                    res.body_json().await.map_err(|e| e.into_inner())?;
                Ok(body.data)
            } else {
                error!("Failed to list variables :(");
                let error =
                    res.body_string().await.map_err(|e| e.into_inner())?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn list_batch(
    config: &Core,
    client: Client,
    workspaces: Vec<Workspace>,
) -> Result<Vec<WorkspaceVariables>, ToolError> {
    // Get the variables for each workspace
    let (_, workspaces_variables) = AsyncStdScope::scope_and_block(|s| {
        for workspace in workspaces {
            let c = client.clone();
            let proc = || async move {
                match list(&workspace.id, config, c).await {
                    Ok(variables) => {
                        info!(
                            "Successfully retrieved variables for workspace {}",
                            workspace.attributes.name.clone().unwrap()
                        );
                        Some(WorkspaceVariables { workspace, variables })
                    }
                    Err(e) => {
                        error!(
                            "Unable to retrieve variables for workspace {}",
                            workspace.attributes.name.unwrap()
                        );
                        error!("{:#?}", e);
                        None
                    }
                }
            };
            s.spawn(proc());
        }
    });
    Ok(workspaces_variables.into_iter().flatten().collect())
}

pub async fn delete(
    variable_id: &str,
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    info!(
        "Deleting variable: {} from workspace: {}",
        variable_id, workspace_id
    );
    let url = Url::parse(&format!(
        "{}/workspaces/{}/vars/{}",
        BASE_URL, workspace_id, variable_id
    ))?;
    let req = build_request(Method::Delete, url, config, None);
    match client.send(req).await {
        Ok(mut res) => {
            if res.status().is_success() {
                info!("Successfully deleted variable!");
                Ok(())
            } else {
                error!("Failed to delete variable :(");
                let error =
                    res.body_string().await.map_err(|e| e.into_inner())?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}
