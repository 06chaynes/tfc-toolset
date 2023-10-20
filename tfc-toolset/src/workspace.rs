use crate::{
    build_request,
    error::{surf_to_tool_error, ToolError},
    filter, set_page_number,
    settings::{Core, Operators, Query, Tag},
    tag, variable, variable_set, Meta, BASE_URL,
};
use async_scoped::AsyncScope;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use surf::{http::Method, Client};
use time::OffsetDateTime;
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
pub struct WorkspaceTags {
    pub workspace: Workspace,
    pub tags: Vec<tag::Tags>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkspaceVariableSets {
    pub workspace: Workspace,
    pub variables: Vec<variable_set::VarSets>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspace {
    pub id: String,
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Deserialize)]
struct Workspaces {
    pub data: Vec<Workspace>,
    pub meta: Option<Meta>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct VcsRepo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress_submodules: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_token_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags_regex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_http_url: Option<String>,
}

impl VcsRepo {
    pub fn new(
        identifier: String,
        oauth_token_id: Option<String>,
        branch: Option<String>,
        ingress_submodules: Option<bool>,
        tags_regex: Option<String>,
    ) -> Self {
        Self {
            identifier: Some(identifier),
            oauth_token_id,
            branch,
            ingress_submodules,
            tags_regex,
            repository_http_url: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectOuter {
    pub data: Project,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Relationships {
    pub project: ProjectOuter,
}

impl Relationships {
    pub fn new(project_id: String) -> Self {
        Self { project: ProjectOuter { data: Project { id: project_id } } }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Remote,
    Local,
    Agent,
    #[default]
    Unknown,
}

impl Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMode::Remote => write!(f, "remote"),
            ExecutionMode::Local => write!(f, "local"),
            ExecutionMode::Agent => write!(f, "agent"),
            ExecutionMode::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<String> for ExecutionMode {
    fn from(item: String) -> Self {
        match item.as_str() {
            "remote" => ExecutionMode::Remote,
            "local" => ExecutionMode::Local,
            "agent" => ExecutionMode::Agent,
            "unknown" => ExecutionMode::Unknown,
            _ => ExecutionMode::Unknown,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Attributes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_pool_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_destroy_plan: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessments_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_apply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub auto_destroy_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<ExecutionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_triggers_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_remote_state: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_all_runs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speculative_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terraform_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_prefixes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcs_repo: Option<VcsRepo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<Relationships>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_names: Option<Vec<String>>,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            name: None,
            agent_pool_id: None,
            allow_destroy_plan: None,
            assessments_enabled: None,
            auto_apply: None,
            auto_destroy_at: None,
            description: None,
            execution_mode: None,
            file_triggers_enabled: None,
            global_remote_state: None,
            queue_all_runs: None,
            source_name: Some("tfc-toolset".to_string()),
            source_url: None,
            speculative_enabled: None,
            terraform_version: None,
            trigger_patterns: None,
            trigger_prefixes: None,
            vcs_repo: None,
            working_directory: None,
            relationships: None,
            tag_names: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceRequest {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WorkspaceOuter {
    pub data: WorkspaceRequest,
}

impl WorkspaceOuter {
    pub fn new(options: Attributes) -> Self {
        Self {
            data: WorkspaceRequest {
                relationship_type: "workspaces".to_string(),
                attributes: options,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WorkspaceResponseOuter {
    pub data: Workspace,
}

async fn send_show_req(
    url: Url,
    config: &Core,
    client: Client,
) -> Result<Workspace, ToolError> {
    let req = build_request(Method::Get, url, config, None);
    match client.send(req).await {
        Ok(mut r) => {
            if r.status().is_success() {
                info!("Successfully retrieved workspace!");
                let res = r
                    .body_json::<WorkspaceResponseOuter>()
                    .await
                    .map_err(surf_to_tool_error)?;
                Ok(res.data)
            } else {
                error!("Failed to retrieve workspace :(");
                let error =
                    r.body_string().await.map_err(surf_to_tool_error)?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn show(
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> Result<Workspace, ToolError> {
    info!("Retrieving workspace {}.", workspace_id);
    let url = Url::parse(&format!("{}/workspaces/{}", BASE_URL, workspace_id))?;
    send_show_req(url, config, client).await
}

pub async fn show_by_name(
    workspace_name: &str,
    config: &Core,
    client: Client,
) -> Result<Workspace, ToolError> {
    info!("Retrieving workspace {}.", workspace_name);
    let url = Url::parse(&format!(
        "{}/organizations/{}/workspaces/{}",
        BASE_URL, config.org, workspace_name
    ))?;
    send_show_req(url, config, client).await
}

pub async fn create(
    options: Attributes,
    config: &Core,
    client: Client,
) -> Result<Workspace, ToolError> {
    let name = options.name.clone().expect("Workspace name is required.");
    info!("Creating workspace {}.", name);
    let url = Url::parse(&format!(
        "{}/organizations/{}/workspaces/",
        BASE_URL, config.org
    ))?;
    let req = build_request(
        Method::Post,
        url,
        config,
        Some(json!(WorkspaceOuter::new(options))),
    );
    match client.send(req).await {
        Ok(mut r) => {
            if r.status().is_success() {
                info!("Successfully created workspace!");
                let res = r
                    .body_json::<WorkspaceResponseOuter>()
                    .await
                    .map_err(surf_to_tool_error)?;
                Ok(res.data)
            } else {
                error!("Failed to create workspace :(");
                let error =
                    r.body_string().await.map_err(surf_to_tool_error)?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn update(
    workspace_id: &str,
    options: Attributes,
    config: &Core,
    client: Client,
) -> Result<Workspace, ToolError> {
    info!("Updating workspace {}.", workspace_id);
    let url = Url::parse(&format!("{}/workspaces/{}", BASE_URL, workspace_id))?;
    let req = build_request(
        Method::Patch,
        url,
        config,
        Some(json!(WorkspaceOuter::new(options))),
    );
    match client.send(req).await {
        Ok(mut r) => {
            if r.status().is_success() {
                info!("Successfully updated workspace!");
                let res = r
                    .body_json::<WorkspaceResponseOuter>()
                    .await
                    .map_err(surf_to_tool_error)?;
                Ok(res.data)
            } else {
                error!("Failed to update workspace :(");
                let error =
                    r.body_string().await.map_err(surf_to_tool_error)?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn update_by_name(
    workspace_name: &str,
    options: Attributes,
    config: &Core,
    client: Client,
) -> Result<Workspace, ToolError> {
    info!("Updating workspace {}.", workspace_name);
    let url = Url::parse(&format!(
        "{}/organizations/{}/workspaces/{}",
        BASE_URL, config.org, workspace_name
    ))?;
    let req = build_request(
        Method::Patch,
        url,
        config,
        Some(json!(WorkspaceOuter::new(options))),
    );
    match client.send(req).await {
        Ok(mut r) => {
            if r.status().is_success() {
                info!("Successfully updated workspace!");
                let res = r
                    .body_json::<WorkspaceResponseOuter>()
                    .await
                    .map_err(surf_to_tool_error)?;
                Ok(res.data)
            } else {
                error!("Failed to update workspace :(");
                let error =
                    r.body_string().await.map_err(surf_to_tool_error)?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn delete(
    workspace_id: &str,
    safe_delete: bool,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    info!("Deleting workspace {}.", workspace_id);
    let mut url =
        Url::parse(&format!("{}/workspaces/{}", BASE_URL, workspace_id))?;
    let mut method = Method::Delete;
    if safe_delete {
        url = Url::parse(&format!("{}/actions/safe-delete", url))?;
        method = Method::Post;
    }
    let req = build_request(method, url, config, None);
    match client.send(req).await {
        Ok(mut r) => {
            if r.status().is_success() {
                info!("Successfully deleted workspace!");
                Ok(())
            } else {
                error!("Failed to delete workspace :(");
                let error =
                    r.body_string().await.map_err(surf_to_tool_error)?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn delete_by_name(
    workspace_name: &str,
    safe_delete: bool,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    info!("Deleting workspace {}.", workspace_name);
    let mut url = Url::parse(&format!(
        "{}/organizations/{}/workspaces/{}",
        BASE_URL, config.org, workspace_name
    ))?;
    let mut method = Method::Delete;
    if safe_delete {
        url = Url::parse(&format!("{}/actions/safe-delete", url))?;
        method = Method::Post;
    }
    let req = build_request(method, url, config, None);
    match client.send(req).await {
        Ok(mut r) => {
            if r.status().is_success() {
                info!("Successfully deleted workspace!");
                Ok(())
            } else {
                error!("Failed to delete workspace :(");
                let error =
                    r.body_string().await.map_err(surf_to_tool_error)?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn list(
    filter: bool,
    config: &Core,
    client: Client,
) -> Result<Vec<Workspace>, ToolError> {
    info!("Retrieving the initial list of workspaces.");
    let mut params = vec![
        ("page[size]", config.pagination.page_size.clone()),
        ("page[number]", config.pagination.start_page.clone()),
    ];
    if let Some(project) = config.project.clone() {
        params.push(("filter[project][id]", project))
    }
    let mut url = Url::parse_with_params(
        &format!("{}/organizations/{}/workspaces/", BASE_URL, config.org),
        &params,
    )?;
    if filter {
        if let Some(query) = config.workspaces.query.clone() {
            if let Some(name) = query.name {
                url = Url::parse_with_params(
                    url.as_str(),
                    &[("search[name]", name)],
                )?
            }
            if let Some(wildcard_name) = query.wildcard_name {
                url = Url::parse_with_params(
                    url.as_str(),
                    &[("search[wildcard-name]", wildcard_name)],
                )?
            }
            if let Some(tags) = query.tags {
                let mut include_tags: Vec<Tag> = Vec::new();
                let mut exclude_tags: Vec<Tag> = Vec::new();
                for tag in tags {
                    match tag.operator {
                        Operators::Equals => {
                            include_tags.push(tag);
                        }
                        Operators::NotEquals => {
                            exclude_tags.push(tag);
                        }
                        _ => {}
                    }
                }
                if !include_tags.is_empty() {
                    url = Url::parse_with_params(
                        url.as_str(),
                        &[(
                            "search[tags]",
                            include_tags
                                .iter()
                                .map(|t| t.name.clone())
                                .collect::<Vec<String>>()
                                .join(","),
                        )],
                    )?
                }
                if !exclude_tags.is_empty() {
                    url = Url::parse_with_params(
                        url.as_str(),
                        &[(
                            "search[exclude-tags]",
                            exclude_tags
                                .iter()
                                .map(|t| t.name.clone())
                                .collect::<Vec<String>>()
                                .join(","),
                        )],
                    )?
                }
            }
        }
    }
    let req = build_request(Method::Get, url.clone(), config, None);
    let mut workspace_list: Workspaces = match client.send(req).await {
        Ok(mut s) => {
            info!("Successfully retrieved workspaces!");
            match s.body_json().await {
                Ok(r) => r,
                Err(e) => {
                    error!("{:#?}", e);
                    return Err(ToolError::General(anyhow::anyhow!(e)));
                }
            }
        }
        Err(e) => {
            error!("Failed to retrieve workspaces :(");
            error!("{:#?}", e);
            return Err(ToolError::General(anyhow::anyhow!(e)));
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
                                    let u = match set_page_number(n, u) {
                                        Some(u) => u,
                                        None => {
                                            error!(
                                                "Failed to set page number."
                                            );
                                            return None;
                                        }
                                    };
                                    let req = build_request(
                                        Method::Get,
                                        u.clone(),
                                        config,
                                        None,
                                    );
                                    match c.send(req).await {
                                        Ok(mut s) => {
                                            info!(
                                                "Successfully retrieved workspaces page {}!",
                                                &n
                                            );
                                            let res = match s
                                                .body_json::<Workspaces>()
                                                .await
                                            {
                                                Ok(r) => r,
                                                Err(e) => {
                                                    error!("{:#?}", e);
                                                    return None;
                                                }
                                            };
                                            Some(res.data)
                                        }
                                        Err(e) => {
                                            error!(
                                                "Failed to retrieve workspaces page {} :(",
                                                &n
                                            );
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
    let mut workspaces = workspace_list.data;
    info!("Finished retrieving workspaces.");
    if filter {
        if let Some(query) = config.workspaces.query.clone() {
            // Filter the workspaces if query tags have been provided
            if query.tags.is_some() {
                info!("Filtering workspaces with tags query.");
                filter::workspace::by_tag(&mut workspaces, config)?;
            }

            if query.variables.is_some() {
                // Get the variables for each workspace
                let mut workspaces_variables =
                    variable::list_batch(config, client, workspaces.clone())
                        .await?;
                // Filter the workspaces if query variables have been provided
                if query.variables.is_some() {
                    info!("Filtering workspaces with variable query.");
                    filter::workspace::by_variable(
                        &mut workspaces_variables,
                        config,
                    )?;
                }

                workspaces.clear();
                for ws in &workspaces_variables {
                    workspaces.push(ws.workspace.clone());
                }
            }
        }
    }
    Ok(workspaces)
}
