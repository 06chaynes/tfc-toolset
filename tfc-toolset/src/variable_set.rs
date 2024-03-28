use crate::{
    build_request,
    error::{surf_to_tool_error, ToolError},
    set_page_number,
    settings::Core,
    variable::{Variable, VariablesOuter},
    workspace::Workspace,
    Meta, BASE_URL,
};

use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use surf::{http::Method, Client};
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
pub struct Relationships {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<WorkspacesOuter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<ProjectsOuter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vars: Option<VariablesOuter>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VarSet {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub attributes: Attributes,
    pub relationships: Relationships,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VarSets {
    pub data: Vec<VarSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}

impl VarSets {
    pub fn new(var_sets: Vec<VarSet>) -> Self {
        VarSets { data: var_sets, meta: None }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VarSetOuter {
    pub data: VarSet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VarSetOptions {
    pub name: String,
    pub description: String,
    pub global: Option<bool>,
    pub workspaces: Option<Vec<Workspace>>,
    pub projects: Option<Vec<String>>,
    pub vars: Option<Vec<Variable>>,
}

impl VarSetOuter {
    pub fn new(options: VarSetOptions) -> Self {
        Self {
            data: VarSet {
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
                    vars: options.vars.map(|v| VariablesOuter { data: v }),
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

async fn check_pagination(
    meta: Meta,
    var_set_list: &mut VarSets,
    url: Url,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    let max_depth = config.pagination.max_depth.parse::<u32>()?;
    if max_depth > 1 || max_depth == 0 {
        let current_depth: u32 = 1;
        if let Some(next_page) = meta.pagination.next_page {
            if max_depth == 0 || current_depth < max_depth {
                let num_pages: u32 = if max_depth >= meta.pagination.total_pages
                    || max_depth == 0
                {
                    meta.pagination.total_pages
                } else {
                    max_depth
                };

                // Get the next page and merge the result
                for n in next_page..=num_pages {
                    let u = url.clone();
                    info!("Retrieving variable set page {}.", &n);
                    let u = match set_page_number(n, u) {
                        Some(u) => u,
                        None => {
                            error!("Failed to set page number.");
                            return Err(ToolError::Pagination(
                                "Failed to set page number.".to_string(),
                            ));
                        }
                    };
                    let req =
                        build_request(Method::Get, u.clone(), config, None);
                    let mut response =
                        client.send(req).await.map_err(surf_to_tool_error)?;
                    if response.status().is_success() {
                        let var_set_pages: Result<VarSets, ToolError> =
                            response
                                .body_json()
                                .await
                                .map_err(surf_to_tool_error);
                        match var_set_pages {
                            Ok(mut t) => {
                                var_set_list.data.append(&mut t.data);
                            }
                            Err(e) => {
                                error!("{:#?}", e);
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn show(
    variable_set_id: &str,
    config: &Core,
    client: Client,
) -> Result<VarSet, ToolError> {
    let url = Url::parse(&format!("{}/varsets/{}", BASE_URL, variable_set_id))?;
    let req = build_request(Method::Get, url, config, None);
    let mut res = client.send(req).await.map_err(surf_to_tool_error)?;
    if res.status().is_success() {
        let var_set: VarSetOuter =
            res.body_json().await.map_err(surf_to_tool_error)?;
        Ok(var_set.data)
    } else {
        error!("Failed to show variable set");
        let error = res.body_string().await.map_err(surf_to_tool_error)?;
        Err(ToolError::General(anyhow::anyhow!(error)))
    }
}

pub async fn list_by_org(
    config: &Core,
    client: Client,
) -> Result<VarSets, ToolError> {
    info!(
        "Retrieving the initial list of variable sets for org {}.",
        config.org
    );
    let params = vec![
        ("page[size]", config.pagination.page_size.clone()),
        ("page[number]", config.pagination.start_page.clone()),
    ];
    let url = Url::parse_with_params(
        &format!("{}/organizations/{}/varsets", BASE_URL, config.org),
        &params,
    )?;
    let req = build_request(Method::Get, url.clone(), config, None);
    let mut var_set_list: VarSets = match client.send(req).await {
        Ok(mut res) => {
            if res.status().is_success() {
                info!("Variable sets for org {} retrieved.", config.org);
                match res.body_json().await {
                    Ok(t) => t,
                    Err(e) => {
                        error!("{:#?}", e);
                        return Err(ToolError::General(anyhow::anyhow!(e)));
                    }
                }
            } else {
                error!("Failed to fetch variable sets for org {}.", config.org);
                let error =
                    res.body_string().await.map_err(surf_to_tool_error)?;
                return Err(ToolError::General(anyhow::anyhow!(error)));
            }
        }
        Err(e) => {
            return Err(ToolError::General(anyhow::anyhow!(e)));
        }
    };
    // Need to check pagination
    if let Some(meta) = var_set_list.meta.clone() {
        check_pagination(meta, &mut var_set_list, url, config, client).await?;
    }
    info!("Finished retrieving variable sets.");
    Ok(var_set_list)
}

pub async fn list_by_project(
    config: &Core,
    client: Client,
) -> Result<VarSets, ToolError> {
    if config.project.clone().is_none() {
        return Err(ToolError::General(anyhow::anyhow!(
            "No project specified in config"
        )));
    }
    let project_id = config.project.clone().unwrap();
    info!(
        "Retrieving the initial list of variable sets for project {}.",
        project_id
    );
    let params = vec![
        ("page[size]", config.pagination.page_size.clone()),
        ("page[number]", config.pagination.start_page.clone()),
    ];
    let url = Url::parse_with_params(
        &format!("{}/projects/{}/varsets", BASE_URL, project_id),
        &params,
    )?;
    let req = build_request(Method::Get, url.clone(), config, None);
    let mut var_set_list: VarSets = match client.send(req).await {
        Ok(mut res) => {
            if res.status().is_success() {
                info!("Variable sets for project {} retrieved.", project_id);
                match res.body_json().await {
                    Ok(t) => t,
                    Err(e) => {
                        error!("{:#?}", e);
                        return Err(ToolError::General(anyhow::anyhow!(e)));
                    }
                }
            } else {
                error!(
                    "Failed to fetch variable sets for project {}.",
                    project_id
                );
                let error =
                    res.body_string().await.map_err(surf_to_tool_error)?;
                return Err(ToolError::General(anyhow::anyhow!(error)));
            }
        }
        Err(e) => {
            return Err(ToolError::General(anyhow::anyhow!(e)));
        }
    };
    // Need to check pagination
    if let Some(meta) = var_set_list.meta.clone() {
        check_pagination(meta, &mut var_set_list, url, config, client).await?;
    }
    info!("Finished retrieving variable sets.");
    Ok(var_set_list)
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
    let req = build_request(
        Method::Post,
        url,
        config,
        Some(json!(VarSetOuter::new(options))),
    );
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
    let req = build_request(
        Method::Post,
        url,
        config,
        Some(json!(ApplyVarSetOuter::from(workspaces))),
    );
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
    let req = build_request(
        Method::Delete,
        url,
        config,
        Some(json!(WorkspacesOuter::from(workspaces))),
    );
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
