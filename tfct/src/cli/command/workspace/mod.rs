mod about;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod list;

pub(crate) mod show;
pub(crate) mod update;

use crate::error::ArgError;

pub use create::create;
pub use delete::{delete, DeleteArgs};
pub use list::{list, ListArgs};
pub use show::show;
pub use update::{update, UpdateArgs};

use crate::cli::command::common::WorkspaceArgsBasic;
use clap::{Args, Subcommand};
use tfc_toolset::workspace::{self, Attributes, ExecutionMode, Relationships};
use tfc_toolset_extras::parse_workspace_name as parse_name;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Args, Debug)]
pub(crate) struct Commands {
    #[command(subcommand)]
    pub command: WorkspaceCmds,
}

#[derive(Subcommand, Debug)]
pub(crate) enum WorkspaceCmds {
    #[clap(about = about::CREATE)]
    Create(DefaultArgs),
    #[clap(about = about::UPDATE)]
    Update(UpdateArgs),
    #[clap(about = about::DELETE)]
    Delete(DeleteArgs),
    #[clap(about = about::LIST)]
    List(ListArgs),
    #[clap(about = about::SHOW)]
    Show(WorkspaceArgsBasic),
}

#[derive(Args, Debug)]
pub struct DefaultArgs {
    #[arg(long, help = about::NAME)]
    pub name: Option<String>,
    #[arg(long, help = about::AGENT_POOL_ID)]
    pub agent_pool_id: Option<String>,
    #[arg(long, help = about::ALLOW_DESTROY_PLAN)]
    pub allow_destroy_plan: Option<bool>,
    #[arg(long, help = about::ASSESSMENTS_ENABLED)]
    pub assessments_enabled: Option<bool>,
    #[arg(long, help = about::AUTO_APPLY)]
    pub auto_apply: Option<bool>,
    #[arg(long, help = about::AUTO_DESTROY_AT)]
    pub auto_destroy_at: Option<String>,
    #[arg(long, help = about::DESCRIPTION)]
    pub description: Option<String>,
    #[arg(long, help = about::EXECUTION_MODE)]
    pub execution_mode: Option<ExecutionMode>,
    #[arg(long, help = about::FILE_TRIGGERS_ENABLED)]
    pub file_triggers_enabled: Option<bool>,
    #[arg(long, help = about::GLOBAL_REMOTE_STATE)]
    pub global_remote_state: Option<bool>,
    #[arg(long, help = about::QUEUE_ALL_RUNS)]
    pub queue_all_runs: Option<bool>,
    #[arg(long, help = about::SOURCE_NAME, default_value = "tfc-toolset")]
    pub source_name: Option<String>,
    #[arg(long, help = about::SOURCE_URL)]
    pub source_url: Option<String>,
    #[arg(long, help = about::SPECULATIVE_ENABLED)]
    pub speculative_enabled: Option<bool>,
    #[arg(long, help = about::TAGS_REGEX)]
    pub tags_regex: Option<String>,
    #[arg(long, help = about::TERRAFORM_VERSION)]
    pub terraform_version: Option<String>,
    #[arg(long, help = about::TRIGGER_PATTERNS)]
    pub trigger_patterns: Option<Vec<String>>,
    #[arg(long, help = about::TRIGGER_PREFIXES)]
    pub trigger_prefixes: Option<Vec<String>>,
    #[arg(long, help = about::VCS_BRANCH)]
    pub vcs_branch: Option<String>,
    #[arg(long, help = about::VCS_IDENTIFIER)]
    pub vcs_identifier: Option<String>,
    #[arg(long, help = about::VCS_INGRESS_SUBMODULES)]
    pub vcs_ingress_submodules: Option<bool>,
    #[arg(long, help = about::VCS_OAUTH_TOKEN_ID)]
    pub vcs_oauth_token_id: Option<String>,
    #[arg(long, help = about::VCS_TAGS_REGEX)]
    pub vcs_tags_regex: Option<String>,
    #[arg(long, help = about::WORKING_DIRECTORY)]
    pub working_directory: Option<String>,
    #[arg(long, help = about::PROJECT_ID)]
    pub project_id: Option<String>,
}

pub(crate) fn build_options(
    args: &DefaultArgs,
) -> Result<Attributes, ArgError> {
    let mut options = Attributes::default();
    if let Some(name) = args.name.clone() {
        parse_name(&name).map_err(ArgError::ExtrasError)?;
        options.name = Some(name);
    }
    if let Some(agent_pool_id) = args.agent_pool_id.clone() {
        options.agent_pool_id = Some(agent_pool_id);
    }
    if let Some(allow_destroy_plan) = args.allow_destroy_plan {
        options.allow_destroy_plan = Some(allow_destroy_plan);
    }
    if let Some(assessments_enabled) = args.assessments_enabled {
        options.assessments_enabled = Some(assessments_enabled);
    }
    if let Some(auto_apply) = args.auto_apply {
        options.auto_apply = Some(auto_apply);
    }
    if let Some(auto_destroy_at) = args.auto_destroy_at.clone() {
        let timestamp = OffsetDateTime::parse(&auto_destroy_at, &Rfc3339)
            .map_err(|_| ArgError::BadRFC3339Timestamp)?;
        options.auto_destroy_at = Some(timestamp);
    }
    if let Some(description) = args.description.clone() {
        options.description = Some(description);
    }
    if let Some(execution_mode) = args.execution_mode.clone() {
        options.execution_mode = Some(execution_mode);
    }
    if let Some(file_triggers_enabled) = args.file_triggers_enabled {
        options.file_triggers_enabled = Some(file_triggers_enabled);
    }
    if let Some(global_remote_state) = args.global_remote_state {
        options.global_remote_state = Some(global_remote_state);
    }
    if let Some(queue_all_runs) = args.queue_all_runs {
        options.queue_all_runs = Some(queue_all_runs);
    }
    if let Some(source_name) = args.source_name.clone() {
        options.source_name = Some(source_name);
    }
    if let Some(source_url) = args.source_url.clone() {
        options.source_url = Some(source_url);
    }
    if let Some(speculative_enabled) = args.speculative_enabled {
        options.speculative_enabled = Some(speculative_enabled);
    }
    if let Some(terraform_version) = args.terraform_version.clone() {
        options.terraform_version = Some(terraform_version);
    }
    if let Some(trigger_patterns) = args.trigger_patterns.clone() {
        options.trigger_patterns = Some(trigger_patterns);
    }
    if let Some(trigger_prefixes) = args.trigger_prefixes.clone() {
        options.trigger_prefixes = Some(trigger_prefixes);
    }
    if let Some(working_directory) = args.working_directory.clone() {
        options.working_directory = Some(working_directory);
    }

    if let Some(project_id) = args.project_id.clone() {
        options.relationships = Some(Relationships::new(project_id));
    }
    if let Some(identifier) = args.vcs_identifier.clone() {
        if args.vcs_oauth_token_id.is_none() {
            return Err(ArgError::MissingVcsOauthTokenId);
        }
        let repo = workspace::VcsRepo::new(
            identifier,
            args.vcs_oauth_token_id.clone(),
            args.vcs_branch.clone(),
            args.vcs_ingress_submodules,
            args.vcs_tags_regex.clone(),
        );
        options.vcs_repo = Some(repo);
    }
    Ok(options)
}
