use crate::{
    error::ToolError,
    settings::{Core, Query},
    variable, BASE_URL,
};
use async_scoped::AsyncScope;
use log::*;
use serde::{Deserialize, Serialize};
use surf::{http::Method, Client, RequestBuilder};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilteredResultInner {
    pub workspaces: Vec<WorkspaceVariables>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilteredResultOuter {
    pub query: Query,
    pub result: FilteredResultInner,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkspaceVariables {
    pub workspace: Workspace,
    pub variables: Vec<variable::Variable>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VcsRepo {
    pub branch: String,
    pub identifier: Option<String>,
    #[serde(rename = "repository-http-url")]
    pub repository_http_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attributes {
    pub name: String,
    #[serde(rename = "vcs-repo")]
    pub vcs_repo: Option<VcsRepo>,
    #[serde(rename = "tag-names")]
    pub tag_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspace {
    pub id: String,
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pagination {
    #[serde(rename = "current-page")]
    pub current_page: u32,
    #[serde(rename = "page-size")]
    pub page_size: u32,
    #[serde(rename = "prev-page")]
    pub prev_page: Option<u32>,
    #[serde(rename = "next-page")]
    pub next_page: Option<u32>,
    #[serde(rename = "total-pages")]
    pub total_pages: u32,
    #[serde(rename = "total-count")]
    pub total_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    pub pagination: Pagination,
}

#[derive(Clone, Debug, Deserialize)]
struct WorkspacesResponseOuter {
    pub data: Vec<Workspace>,
    pub meta: Option<Meta>,
}

pub async fn get_workspaces(
    config: &Core,
    client: Client,
) -> Result<Vec<Workspace>, ToolError> {
    info!("Retrieving the initial list of workspaces.");
    let mut url = Url::parse_with_params(
        &format!("{}/organizations/{}/workspaces/", BASE_URL, config.org),
        &[
            ("page[number]", config.pagination.start_page.clone()),
            ("page[size]", config.pagination.page_size.clone()),
        ],
    )?;
    if let Some(name) = config.query.name.clone() {
        url = Url::parse_with_params(url.as_str(), &[("search[name]", name)])?
    }
    let req = RequestBuilder::new(Method::Get, url.clone())
        .header("Authorization", &format!("Bearer {}", config.token))
        .build();
    let mut workspace_list: WorkspacesResponseOuter =
        match client.recv_string(req).await {
            Ok(s) => {
                info!("Successfully retrieved workspaces!");
                serde_json::from_str::<WorkspacesResponseOuter>(&s)?
            }
            Err(e) => {
                error!("Failed to retrieve workspaces :(");
                return Err(ToolError::General(e.into_inner()));
            }
        };
    // Need to check pagination
    if let Some(meta) = workspace_list.meta {
        let max_depth = config.pagination.max_depth.parse::<u32>()?;
        if max_depth > 1 || max_depth == 0 {
            let current_depth: u32 = 1;
            if let Some(next_page) = meta.pagination.next_page {
                if max_depth == 0 || current_depth < max_depth {
                    let num_pages: u32 = if max_depth
                        >= meta.pagination.total_pages
                        || max_depth == 0
                    {
                        meta.pagination.total_pages
                    } else {
                        max_depth
                    };

                    // Get the next page and merge the result
                    let (_, workspace_pages) = AsyncScope::scope_and_block(
                        |s| {
                            for n in next_page..=num_pages {
                                let c = client.clone();
                                let u = url.clone();
                                let proc = || async move {
                                    info!("Retrieving workspaces page {}.", &n);
                                    let u = match Url::parse_with_params(
                                        u.clone().as_str(),
                                        &[("page[number]", &n.to_string())],
                                    ) {
                                        Ok(u) => u,
                                        Err(e) => {
                                            error!("{:#?}", e);
                                            return None;
                                        }
                                    };
                                    let req = RequestBuilder::new(
                                        Method::Get,
                                        u.clone(),
                                    )
                                    .header(
                                        "Authorization",
                                        &format!("Bearer {}", config.token),
                                    )
                                    .build();
                                    match c.recv_string(req).await {
                                        Ok(s) => {
                                            info!("Successfully retrieved workspaces page {}!", &n);
                                            let res = match serde_json::from_str::<
                                                WorkspacesResponseOuter,
                                            >(
                                                &s
                                            ) {
                                                Ok(r) => r,
                                                Err(e) => {
                                                    error!("{:#?}", e);
                                                    return None;
                                                }
                                            };
                                            Some(res.data)
                                        }
                                        Err(e) => {
                                            error!("Failed to retrieve workspaces page {} :(", &n);
                                            error!("{:#?}", e);
                                            None
                                        }
                                    }
                                };
                                s.spawn(proc());
                            }
                        },
                    );
                    for mut ws in workspace_pages.into_iter().flatten() {
                        workspace_list.data.append(&mut ws);
                    }
                }
            }
        }
    }
    info!("Finished retrieving workspaces.");
    Ok(workspace_list.data)
}

pub async fn get_workspaces_variables(
    config: &Core,
    client: Client,
    workspaces: Vec<Workspace>,
) -> Result<Vec<WorkspaceVariables>, ToolError> {
    // Get the variables for each workspace
    let (_, workspaces_variables) = AsyncScope::scope_and_block(|s| {
        for workspace in workspaces {
            let c = client.clone();
            let proc = || async move {
                match variable::get_variables(&workspace.id, config, c).await {
                    Ok(variables) => {
                        info!(
                            "Successfully retrieved variables for workspace {}",
                            workspace.attributes.name
                        );
                        Some(WorkspaceVariables { workspace, variables })
                    }
                    Err(e) => {
                        error!(
                            "Unable to retrieve variables for workspace {}",
                            workspace.attributes.name
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
