use crate::{error::ToolError, settings::Core, BASE_URL};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::{Display, Formatter};
use surf::{http::Method, Client, RequestBuilder};
use url::Url;

// Statuses in Terraform Cloud that indicate a run is in a completed state
pub const COMPLETED_STATUSES: [Status; 5] = [
    Status::Applied,
    Status::Canceled,
    Status::Errored,
    Status::Discarded,
    Status::PlannedAndFinished,
];

// Statuses in Terraform Cloud that indicate a run is in an error state
pub const ERROR_STATUSES: [Status; 6] = [
    Status::Canceled,
    Status::Errored,
    Status::Discarded,
    Status::Failed,
    Status::Unreachable,
    Status::Unknown,
];

// Statuses in Terraform Cloud
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Status {
    Pending,
    PlanQueued,
    Queued,
    ManagedQueued,
    Running,
    Passed,
    Failed,
    Applying,
    Planning,
    Planned,
    Applied,
    Canceled,
    Errored,
    Discarded,
    PlannedAndFinished,
    Unreachable,
    #[default]
    Unknown,
    Finished,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::PlanQueued => write!(f, "plan_queued"),
            Status::Queued => write!(f, "queued"),
            Status::ManagedQueued => write!(f, "managed_queued"),
            Status::Running => write!(f, "running"),
            Status::Passed => write!(f, "passed"),
            Status::Failed => write!(f, "failed"),
            Status::Applying => write!(f, "applying"),
            Status::Planning => write!(f, "planning"),
            Status::Planned => write!(f, "planned"),
            Status::Applied => write!(f, "applied"),
            Status::Canceled => write!(f, "canceled"),
            Status::Errored => write!(f, "errored"),
            Status::Discarded => write!(f, "discarded"),
            Status::PlannedAndFinished => write!(f, "planned_and_finished"),
            Status::Unreachable => write!(f, "unreachable"),
            Status::Unknown => write!(f, "unknown"),
            Status::Finished => write!(f, "finished"),
        }
    }
}

impl From<String> for Status {
    fn from(item: String) -> Self {
        match item.as_str() {
            "pending" => Status::Pending,
            "plan_queued" => Status::PlanQueued,
            "queued" => Status::Queued,
            "managed_queued" => Status::ManagedQueued,
            "running" => Status::Running,
            "passed" => Status::Passed,
            "failed" => Status::Failed,
            "applying" => Status::Applying,
            "planning" => Status::Planning,
            "planned" => Status::Planned,
            "applied" => Status::Applied,
            "canceled" => Status::Canceled,
            "errored" => Status::Errored,
            "discarded" => Status::Discarded,
            "planned_and_finished" => Status::PlannedAndFinished,
            "unreachable" => Status::Unreachable,
            "unknown" => Status::Unknown,
            "finished" => Status::Finished,
            _ => Status::Unknown,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attributes {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "terraform-version")]
    pub terraform_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "plan-only")]
    pub plan_only: Option<bool>,
    #[serde(rename = "target-addrs")]
    pub target_addrs: Vec<String>,
    #[serde(rename = "replace-addrs")]
    pub replace_addrs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "refresh-only")]
    pub refresh_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "auto-apply")]
    pub auto_apply: Option<bool>,
    #[serde(rename = "allow-empty-apply")]
    pub allow_empty_apply: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "is-destroy")]
    pub is_destroy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "created-at")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            message: "Run created by tfc-toolset".to_string(),
            terraform_version: None,
            plan_only: None,
            target_addrs: Vec::new(),
            replace_addrs: Vec::new(),
            refresh: None,
            refresh_only: None,
            auto_apply: None,
            allow_empty_apply: false,
            is_destroy: None,
            created_at: None,
            status: None,
        }
    }
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
    pub workspace: WorkspaceOuter,
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

impl RunRequestOuter {
    fn new(workspace_id: &str, attributes: Option<Attributes>) -> Self {
        Self {
            data: RunRequest {
                attributes: attributes.unwrap_or_default(),
                request_type: "runs".to_string(),
                relationships: Relationships {
                    workspace: WorkspaceOuter {
                        data: Workspace {
                            relationship_type: "workspaces".to_string(),
                            id: workspace_id.to_string(),
                        },
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
pub struct Run {
    pub id: String,
    pub attributes: Attributes,
    pub links: Links,
}

#[derive(Clone, Debug, Deserialize)]
struct RunResponseOuter {
    pub data: Run,
}

fn to_tool_error(e: surf::Error) -> ToolError {
    ToolError::General(e.into_inner())
}

pub async fn create(
    workspace_id: &str,
    attributes: Option<Attributes>,
    config: &Core,
    client: Client,
) -> Result<Run, ToolError> {
    let url = Url::parse(&format!("{}/runs", BASE_URL))?;
    let req = RequestBuilder::new(Method::Post, url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Content-Type", "application/vnd.api+json")
        .body(json!(RunRequestOuter::new(workspace_id, attributes)))
        .build();
    let res = client.recv_string(req).await.map_err(to_tool_error)?;
    let run: RunResponseOuter = serde_json::from_str(&res)?;
    Ok(run.data)
}

pub async fn status(
    run_id: &str,
    config: &Core,
    client: Client,
) -> Result<Run, ToolError> {
    let url = Url::parse(&format!("{}/runs/{}", BASE_URL, run_id))?;
    let req = RequestBuilder::new(Method::Get, url)
        .header("Authorization", format!("Bearer {}", config.token))
        .build();
    let res = client.recv_string(req).await.map_err(to_tool_error)?;
    let run: RunResponseOuter = serde_json::from_str(&res)?;
    Ok(run.data)
}
