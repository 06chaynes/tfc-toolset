use crate::{
    error::{surf_to_tool_error, ToolError},
    settings::Core,
    workspace, BASE_URL,
};
use async_std::task::{self, JoinHandle};
use dashmap::DashMap;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use surf::{http::Method, Client, RequestBuilder};
use url::Url;

// Statuses in Terraform Cloud that indicate a run is in a completed state
pub const COMPLETED_STATUSES: [Status; 6] = [
    Status::Applied,
    Status::Canceled,
    Status::ForceCanceled,
    Status::Errored,
    Status::Discarded,
    Status::PlannedAndFinished,
];

// Statuses in Terraform Cloud that indicate a run is in an error state
pub const ERROR_STATUSES: [Status; 8] = [
    Status::Canceled,
    Status::ForceCanceled,
    Status::Errored,
    Status::Discarded,
    Status::Failed,
    Status::Unreachable,
    Status::Unknown,
    Status::PolicySoftFailed,
];

// Statuses in Terraform Cloud
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Status {
    ApplyQueued,
    Pending,
    PlanQueued,
    Queuing,
    Queued,
    Fetching,
    CostEstimating,
    CostEstimated,
    FetchingCompleted,
    PrePlanRunning,
    PrePlanCompleted,
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
    PlannedAndSaved,
    PolicyChecking,
    PolicyOverride,
    PolicySoftFailed,
    Unreachable,
    ForceCanceled,
    #[default]
    Unknown,
    Finished,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::ApplyQueued => write!(f, "apply_queued"),
            Status::Pending => write!(f, "pending"),
            Status::PlanQueued => write!(f, "plan_queued"),
            Status::Queuing => write!(f, "queuing"),
            Status::Queued => write!(f, "queued"),
            Status::Fetching => write!(f, "fetching"),
            Status::CostEstimating => write!(f, "cost_estimating"),
            Status::CostEstimated => write!(f, "cost_estimated"),
            Status::FetchingCompleted => write!(f, "fetching_completed"),
            Status::PrePlanRunning => write!(f, "pre_plan_running"),
            Status::PrePlanCompleted => write!(f, "pre_plan_completed"),
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
            Status::PlannedAndSaved => write!(f, "planned_and_saved"),
            Status::PolicyChecking => write!(f, "policy_checking"),
            Status::PolicyOverride => write!(f, "policy_override"),
            Status::PolicySoftFailed => write!(f, "policy_soft_failed"),
            Status::Unreachable => write!(f, "unreachable"),
            Status::ForceCanceled => write!(f, "force_canceled"),
            Status::Unknown => write!(f, "unknown"),
            Status::Finished => write!(f, "finished"),
        }
    }
}

impl From<String> for Status {
    fn from(item: String) -> Self {
        match item.as_str() {
            "apply_queued" => Status::ApplyQueued,
            "pending" => Status::Pending,
            "plan_queued" => Status::PlanQueued,
            "queuing" => Status::Queuing,
            "queued" => Status::Queued,
            "fetching" => Status::Fetching,
            "cost_estimating" => Status::CostEstimating,
            "cost_estimated" => Status::CostEstimated,
            "fetching_completed" => Status::FetchingCompleted,
            "pre_plan_running" => Status::PrePlanRunning,
            "pre_plan_completed" => Status::PrePlanCompleted,
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
            "planned_and_saved" => Status::PlannedAndSaved,
            "policy_checking" => Status::PolicyChecking,
            "policy_override" => Status::PolicyOverride,
            "policy_soft_failed" => Status::PolicySoftFailed,
            "unreachable" => Status::Unreachable,
            "force_canceled" => Status::ForceCanceled,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "save-plan")]
    pub save_plan: Option<bool>,
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
            save_plan: None,
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

pub struct QueueOptions {
    pub max_concurrent: usize,
    pub max_iterations: usize,
    pub status_check_sleep_seconds: u64,
}

pub struct QueueResult {
    pub results: Vec<RunResult>,
    pub errors: Vec<RunResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunResult {
    pub id: RunId,
    pub status: String,
    pub workspace_id: String,
}

pub type RunId = String;

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
    let res = client.recv_string(req).await.map_err(surf_to_tool_error)?;
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
    let res = client.recv_string(req).await.map_err(surf_to_tool_error)?;
    let run: RunResponseOuter = serde_json::from_str(&res)?;
    Ok(run.data)
}

pub async fn work_queue(
    queue: &mut BTreeMap<String, workspace::Workspace>,
    options: QueueOptions,
    attributes: Attributes,
    client: Client,
    core: &Core,
) -> Result<QueueResult, ToolError> {
    let running = DashMap::with_capacity(options.max_concurrent);
    let mut results = Vec::with_capacity(queue.len());
    let mut errors = Vec::new();
    while !queue.is_empty() {
        let mut handles = Vec::with_capacity(options.max_concurrent);
        while running.len() < options.max_concurrent && !queue.is_empty() {
            let (id, ws) = queue.pop_first().unwrap();
            info!("Creating run for workspace: {}", &ws.id);
            let client = client.clone();
            let attributes = attributes.clone();
            let will_auto_apply = attributes.auto_apply.unwrap_or(false);
            let will_save_plan = attributes.save_plan.unwrap_or(false);
            let core = core.clone();
            let ws_id = ws.id.clone();
            let mut iterations = 0;
            let handle: JoinHandle<Result<RunResult, ToolError>> = task::spawn(
                async move {
                    let mut run = create(
                        &id.clone(),
                        Some(attributes),
                        &core,
                        client.clone(),
                    )
                    .await?;
                    info!(
                        "Run {} created for workspace {}",
                        &run.id,
                        &id.clone()
                    );
                    while !COMPLETED_STATUSES.contains(
                        &run.attributes
                            .status
                            .clone()
                            .unwrap_or(Status::Unknown),
                    ) {
                        run = status(&run.id, &core, client.clone()).await?;
                        let status = run
                            .attributes
                            .status
                            .clone()
                            .unwrap_or(Status::Unknown);
                        info!("Run {} status: {}", &run.id, &status);
                        if COMPLETED_STATUSES.contains(&status)
                            || !will_auto_apply && status == Status::Planned
                            || will_save_plan
                                && status == Status::PlannedAndSaved
                        {
                            // If auto_apply is false and status is planned, then we can break out of the loop
                            // because the run will require confirmation before applying
                            // If save_plan is true and status is planned_and_saved, then we can break out of the loop
                            // If completed, then we can also break out of the loop
                            break;
                        }
                        iterations += 1;
                        if iterations >= options.max_iterations {
                            error!(
                                    "Run {} for workspace {} has been in status {} too long.",
                                    &run.id, &id.clone(), &status.clone()
                                );
                            if status == Status::Pending {
                                error!(
                                    "There is likely previous run pending. Please check the workspace in the UI."
                                );
                            } else {
                                error!(
                                    "This is likely some error. Please check the run in the UI."
                                );
                            }
                            break;
                        }
                        async_std::task::sleep(Duration::from_secs(
                            options.status_check_sleep_seconds,
                        ))
                        .await;
                    }
                    Ok(RunResult {
                        id: run.id,
                        status: run
                            .attributes
                            .status
                            .unwrap_or(Status::Unknown)
                            .to_string(),
                        workspace_id: id,
                    })
                },
            );
            running.insert(ws_id, ws);
            handles.push(handle);
        }
        for handle in handles {
            let result = handle.await?;
            running.remove(result.id.clone().as_str());
            if ERROR_STATUSES.contains(&Status::from(result.status.clone())) {
                errors.push(result.clone());
            }
            results.push(result);
        }
    }
    Ok(QueueResult { results, errors })
}
