use crate::{error::ToolError, settings::Core, BASE_URL};
use serde::{Deserialize, Serialize};
use serde_json::json;
use surf::{http::Method, Client, RequestBuilder};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attributes {
    pub message: String,
    #[serde(rename = "terraform-version")]
    pub terraform_version: Option<String>,
    #[serde(rename = "plan-only")]
    pub plan_only: Option<bool>,
    #[serde(rename = "target-addrs")]
    pub target_addrs: Option<Vec<String>>,
    #[serde(rename = "replace-addrs")]
    pub replace_addrs: Option<Vec<String>>,
    pub refresh: Option<bool>,
    #[serde(rename = "refresh-only")]
    pub refresh_only: Option<bool>,
    #[serde(rename = "auto-apply")]
    pub auto_apply: Option<bool>,
    #[serde(rename = "allow-empty-apply")]
    pub allow_empty_apply: Option<bool>,
    #[serde(rename = "is-destroy")]
    pub is_destroy: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkspaceOuter {
    pub data: Workspace,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Workspace {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Relationships {
    pub workspace: Workspace,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunRequest {
    pub attributes: Attributes,
    #[serde(rename = "type")]
    pub request_type: String,
    pub relationships: Relationships,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
struct RunRequestOuter {
    pub data: RunRequest,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            message: "Run created by tfc-toolset".to_string(),
            terraform_version: None,
            plan_only: None,
            target_addrs: None,
            replace_addrs: None,
            refresh: None,
            refresh_only: None,
            auto_apply: None,
            allow_empty_apply: None,
            is_destroy: None,
        }
    }
}

impl RunRequestOuter {
    fn new(workspace_id: &str, attributes: Option<Attributes>) -> Self {
        Self {
            data: RunRequest {
                attributes: attributes.unwrap_or_default(),
                request_type: "runs".to_string(),
                relationships: Relationships {
                    workspace: Workspace {
                        relationship_type: "workspaces".to_string(),
                        id: workspace_id.to_string(),
                    },
                },
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Links {
    #[serde(rename = "self")]
    pub self_link: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunResponse {
    pub id: String,
    #[serde(rename = "created-at")]
    pub created_at: String,
    pub status: String,
    pub links: Links,
}

#[derive(Clone, Debug, Deserialize)]
struct RunResponseOuter {
    pub data: RunResponse,
}

pub async fn create(
    workspace_id: &str,
    attributes: Option<Attributes>,
    config: &Core,
    client: Client,
) -> Result<RunResponse, ToolError> {
    let url = Url::parse(&format!("{}/runs", BASE_URL))?;
    let req = RequestBuilder::new(Method::Post, url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Content-Type", "application/vnd.api+json")
        .body(json!(RunRequestOuter::new(workspace_id, attributes)))
        .build();
    match client.recv_string(req).await {
        Ok(s) => Ok(serde_json::from_str::<RunResponseOuter>(&s)?.data),
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}
