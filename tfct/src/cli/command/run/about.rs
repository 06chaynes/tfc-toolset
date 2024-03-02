pub(super) const STATUS: &str = "Get the status of a run.";
pub(super) const SPEC: &str = "Queue up speculative plan runs";
pub(super) const PLAN: &str = "Queue up plan and apply runs";
pub(super) const CREATE: &str = "Create a run.";
pub(super) const CANCEL: &str = "Cancel a run.";
pub(super) const RUN_ID: &str = "The id of the run.";
pub(super) const MESSAGE: &str = "A message to include with the run";
pub(super) const TARGET_ADDRS: &str =
    "A list of resource addresses to target for the run";
pub(super) const REPLACE_ADDRS: &str =
    "A list of resource addresses to replace for the run";
pub(super) const AUTO_APPLY: &str =
    "Automatically apply the run if the plan is successful";
pub(super) const ALLOW_EMPTY_APPLY: &str =
    "Apply the run even when the plan contains no changes";
pub(super) const IS_DESTROY: &str =
    "Whether this plan is a destroy plan that will destroy all provisioned resources";
pub(super) const REFRESH_ONLY: &str =
    "Whether this run should refresh the state without modifying any resources";
pub(super) const SAVE_PLAN: &str =
    "Specifies if this should be a saved plan run which can be applied later";
pub(super) const TERRAFORM_VERSION: &str =
    "The version of Terraform to use for this run, overriding the value from settings";
pub(super) const QUEUE: &str =
    "Execute runs in batches with overridable limits";
pub(super) const MAX_CONCURRENT: &str =
    "The maximum number of runs to execute concurrently";
pub(super) const MAX_ITERATIONS: &str = "The maximum number of times to \
    check the status of a run before giving up";
pub(super) const STATUS_CHECK_SLEEP_SECONDS: &str = "The number of seconds to \
    wait between checking the status of a run";
pub(super) const CANCEL_ON_TIMEOUT: &str = "Whether to cancel the run if it \
    reaches the configured limits";
