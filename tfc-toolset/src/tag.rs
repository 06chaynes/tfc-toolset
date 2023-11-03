use crate::{
    build_request, error::ToolError, set_page_number, settings::Core,
    workspace, Meta, BASE_URL,
};

use async_scoped::AsyncScope;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use surf::{http::Method, Client};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attributes {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    #[serde(rename = "type")]
    pub relationship_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub attributes: Attributes,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Tag {
            relationship_type: "tags".to_string(),
            id: None,
            attributes: Attributes { name },
        }
    }
}

impl From<String> for Tag {
    fn from(name: String) -> Self {
        Tag::new(name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tags {
    pub data: Vec<Tag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}

impl Tags {
    pub fn new(tags: Vec<Tag>) -> Self {
        Tags { data: tags, meta: None }
    }
}

pub async fn list(
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> Result<Tags, ToolError> {
    info!(
        "Retrieving the initial list of tags for workspace {}.",
        workspace_id
    );
    let params = vec![
        ("page[size]", config.pagination.page_size.clone()),
        ("page[number]", config.pagination.start_page.clone()),
    ];
    let url = Url::parse_with_params(
        &format!("{}/workspaces/{}/relationships/tags", BASE_URL, workspace_id),
        &params,
    )?;
    let req = build_request(Method::Get, url.clone(), config, None);
    let mut tag_list: Tags = match client.send(req).await {
        Ok(mut res) => {
            if res.status().is_success() {
                info!("Tags for workspace {} retrieved.", workspace_id);
                match res.body_json().await {
                    Ok(t) => t,
                    Err(e) => {
                        error!("{:#?}", e);
                        return Err(ToolError::General(anyhow::anyhow!(e)));
                    }
                }
            } else {
                error!(
                    "Failed to retrieve tags for workspace {}.",
                    workspace_id
                );
                let error =
                    res.body_string().await.map_err(|e| e.into_inner())?;
                return Err(ToolError::General(anyhow::anyhow!(error)));
            }
        }
        Err(e) => {
            return Err(ToolError::General(e.into_inner()));
        }
    };
    // Need to check pagination
    if let Some(meta) = tag_list.meta.clone() {
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
                    let (_, tag_pages) = AsyncScope::scope_and_block(|s| {
                        for n in next_page..=num_pages {
                            let c = client.clone();
                            let u = url.clone();
                            let proc = || async move {
                                info!("Retrieving tags page {}.", &n);
                                let u = match set_page_number(n, u) {
                                    Some(u) => u,
                                    None => {
                                        error!("Failed to set page number.");
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
                                    Ok(mut r) => {
                                        if r.status().is_success() {
                                            info!(
                                                "Successfully retrieved tags page {}!",
                                                &n
                                            );
                                            let res = match r
                                                .body_json::<Tags>()
                                                .await
                                            {
                                                Ok(r) => r,
                                                Err(e) => {
                                                    error!("{:#?}", e);
                                                    return None;
                                                }
                                            };
                                            Some(res.data)
                                        } else {
                                            let e = r
                                                .body_string()
                                                .await
                                                .unwrap_or(
                                                    "Failed to parse error"
                                                        .to_string(),
                                                );
                                            error!("{:#?}", e);
                                            None
                                        }
                                    }
                                    Err(e) => {
                                        error!(
                                                "Failed to retrieve tags page {} :(",
                                                &n
                                            );
                                        error!("{:#?}", e);
                                        None
                                    }
                                }
                            };
                            s.spawn(proc());
                        }
                    });
                    for mut t in tag_pages.into_iter().flatten() {
                        tag_list.data.append(&mut t);
                    }
                }
            }
        }
    }
    info!("Finished retrieving tags.");
    Ok(tag_list)
}

pub async fn list_by_name(
    workspace_name: &str,
    config: &Core,
    client: Client,
) -> Result<Tags, ToolError> {
    let workspace =
        workspace::show_by_name(workspace_name, config, client.clone()).await?;
    list(&workspace.id, config, client).await
}

pub async fn add(
    workspace_id: &str,
    tags: Vec<String>,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    info!("Tagging workspace {}.", workspace_id);
    let url = Url::parse(&format!(
        "{}/workspaces/{}/relationships/tags",
        BASE_URL, workspace_id
    ))?;
    let tags = tags.into_iter().map(Tag::from).collect::<Vec<Tag>>();
    let req =
        build_request(Method::Post, url, config, Some(json!(Tags::new(tags))));
    match client.send(req).await {
        Ok(mut r) => {
            if r.status().is_success() {
                info!("Workspace {} tagged.", workspace_id);
                Ok(())
            } else {
                error!("Failed to tag workspace {}.", workspace_id);
                let error =
                    r.body_string().await.map_err(|e| e.into_inner())?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn add_by_name(
    workspace_name: &str,
    tags: Vec<String>,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    let workspace =
        workspace::show_by_name(workspace_name, config, client.clone()).await?;
    add(&workspace.id, tags, config, client).await
}

pub async fn remove(
    workspace_id: &str,
    tags: Vec<String>,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    info!("Removing tags from workspace {}.", workspace_id);
    let url = Url::parse(&format!(
        "{}/workspaces/{}/relationships/tags",
        BASE_URL, workspace_id
    ))?;
    let tags = tags.into_iter().map(Tag::from).collect::<Vec<Tag>>();
    let req = build_request(
        Method::Delete,
        url,
        config,
        Some(json!(Tags::new(tags))),
    );
    match client.send(req).await {
        Ok(mut r) => {
            if r.status().is_success() {
                info!("Tags removed from workspace {}.", workspace_id);
                Ok(())
            } else {
                error!(
                    "Failed to remove tags from workspace {}.",
                    workspace_id
                );
                let error =
                    r.body_string().await.map_err(|e| e.into_inner())?;
                Err(ToolError::General(anyhow::anyhow!(error)))
            }
        }
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn remove_by_name(
    workspace_name: &str,
    tags: Vec<String>,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    let workspace =
        workspace::show_by_name(workspace_name, config, client.clone()).await?;
    remove(&workspace.id, tags, config, client).await
}
