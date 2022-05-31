use crate::{
    error::ToolError,
    settings::{Core, Query},
    variable, BASE_URL,
};
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
                    for n in next_page..=num_pages {
                        info!("Retrieving workspaces page {}.", &n);
                        url = Url::parse_with_params(
                            url.clone().as_str(),
                            &[("page[number]", &n.to_string())],
                        )?;
                        let req = RequestBuilder::new(Method::Get, url.clone())
                            .header(
                                "Authorization",
                                &format!("Bearer {}", config.token),
                            )
                            .build();
                        match client.recv_string(req).await {
                            Ok(s) => {
                                info!("Successfully retrieved workspaces!");
                                let mut res =
                                    serde_json::from_str::<
                                        WorkspacesResponseOuter,
                                    >(&s)?;
                                workspace_list.data.append(&mut res.data);
                            }
                            Err(e) => {
                                error!("Failed to retrieve workspaces :(");
                                return Err(ToolError::General(e.into_inner()));
                            }
                        }
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
    let mut workspaces_variables: Vec<WorkspaceVariables> = vec![];
    for workspace in workspaces {
        info!(
            "Retrieving variables for workspace {}",
            workspace.attributes.name
        );
        let variables =
            variable::get_variables(&workspace.id, config, client.clone())
                .await?;
        info!("Successfully retrieved variables!");
        workspaces_variables.push(WorkspaceVariables { workspace, variables })
    }
    Ok(workspaces_variables)
}
