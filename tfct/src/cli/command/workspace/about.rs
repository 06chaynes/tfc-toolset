pub(super) const CREATE: &str = "Create a new workspace";

pub(super) const UPDATE: &str = "Update an existing workspace";

pub(super) const DELETE: &str = "Delete an existing workspace";

pub(super) const SAFE_DELETE: &str =
    "Delete an existing workspace, but only if it is not managing resources";

pub(super) const LIST: &str = "List all workspaces";

pub(super) const FILTER_LIST: &str =
    "Filter the list of workspaces by given criteria";

pub(super) const SHOW: &str = "Show details about a workspace";

pub(super) const NAME: &str = "The name of the workspace";

pub(super) const AGENT_POOL_ID: &str =
    "The ID of the agent pool belonging to the workspace's organization";

pub(super) const ALLOW_DESTROY_PLAN: &str =
    "Whether destroy plans can be queued on this workspace";

pub(super) const ASSESSMENTS_ENABLED: &str = "Whether or not Terraform Cloud performs health assessments for the workspace";

pub(super) const AUTO_APPLY: &str = "Whether to automatically apply changes when a Terraform plan is successful";

pub(super) const AUTO_DESTROY_AT: &str = "Timestamp (in RFC3339 format) when the next scheduled destroy run will occur";

pub(super) const DESCRIPTION: &str = "A description for the workspace";

pub(super) const EXECUTION_MODE: &str = "Which execution mode to use for the workspace. Valid values are remote, local, and agent";

pub(super) const FILE_TRIGGERS_ENABLED: &str =
    "Whether to filter runs based on the changed files in a VCS push";

pub(super) const GLOBAL_REMOTE_STATE: &str = "Whether the workspace should allow all workspaces in the organization to access its state data during runs";

pub(super) const QUEUE_ALL_RUNS: &str =
    "Whether runs should be queued immediately after workspace creation";

pub(super) const SOURCE_NAME: &str =
    "A friendly name for the application or client creating this workspace";

pub(super) const SOURCE_URL: &str =
    "A URL identifying the application or client creating this workspace";

pub(super) const SPECULATIVE_ENABLED: &str =
    "Whether this workspace allows automatic speculative plans";

pub(super) const TAGS_REGEX: &str =
    "A regular expression used to match Git tags";

pub(super) const TERRAFORM_VERSION: &str =
    "The version of Terraform to use for this workspace";

pub(super) const TRIGGER_PATTERNS: &str = "A list of glob patterns that describe the files Terraform Cloud monitors for changes";

pub(super) const TRIGGER_PREFIXES: &str = "A list of trigger prefixes that describe the paths Terraform Cloud monitors for changes, in addition to the working directory";

pub(super) const VCS_BRANCH: &str =
    "The repository branch that Terraform will execute from";

pub(super) const VCS_IDENTIFIER: &str = "A reference to your VCS repository in the format :org/:repo where :org and :repo refer to the organization and repository in your VCS provider";

pub(super) const VCS_INGRESS_SUBMODULES: &str =
    "Whether to fetch submodules for a VCS repository";

pub(super) const VCS_OAUTH_TOKEN_ID: &str =
    "The VCS Connection (OAuth Connection + Token) to use";

pub(super) const VCS_TAGS_REGEX: &str =
    "A regular expression used to match Git tags";

pub(super) const WORKING_DIRECTORY: &str =
    "A relative path that Terraform will execute within";

pub(super) const PROJECT_ID: &str =
    "The ID of the project to create the workspace in";
