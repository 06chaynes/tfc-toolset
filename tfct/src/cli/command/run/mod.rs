mod about;
pub(crate) mod plan;
pub(crate) mod spec;
pub(crate) mod status;

pub use plan::plan;
pub use spec::spec;
pub use status::{status, StatusArgs};

use crate::cli::command::common::WorkspaceArgs;
use clap::{Args, Subcommand};
use tfc_toolset::run::{Attributes, QueueOptions};

#[derive(Args, Debug)]
pub(crate) struct Commands {
    #[command(subcommand)]
    pub command: RunCmds,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RunCmds {
    #[clap(about = about::STATUS)]
    Status(StatusArgs),
    #[clap(about = about::PLAN)]
    Plan(DefaultArgs),
    #[clap(about = about::APPLY)]
    Apply(ApplyArgs),
}

#[derive(clap::Args, Debug)]
pub struct ApplyArgs {
    #[clap(flatten)]
    pub default: DefaultArgs,
    #[arg(long, help = about::AUTO_APPLY, default_value = "false")]
    pub auto_apply: Option<bool>,
    #[arg(
    long,
    help = about::ALLOW_EMPTY_APPLY,
    default_value = "false"
    )]
    pub allow_empty_apply: Option<bool>,
    #[arg(long, help = about::IS_DESTROY, default_value = "false")]
    pub is_destroy: Option<bool>,
    #[arg(long, help = about::REFRESH_ONLY, default_value = "false")]
    pub refresh_only: Option<bool>,
}

#[derive(clap::Args, Debug)]
pub struct DefaultArgs {
    #[clap(flatten)]
    pub workspace: WorkspaceArgs,
    #[arg(long, help = about::MESSAGE, default_value = "Run created by tfc-toolset")]
    pub message: Option<String>,
    #[arg(long, help = about::TARGET_ADDRS)]
    pub target_addrs: Option<Vec<String>>,
    #[arg(long, help = about::REPLACE_ADDRS)]
    pub replace_addrs: Option<Vec<String>>,
    #[arg(long, help = about::TERRAFORM_VERSION)]
    pub terraform_version: Option<String>,
    #[arg(long, help = about::SAVE_PLAN, default_value = "false")]
    pub save_plan: Option<bool>,
    #[arg(short = 'q', long, help = about::QUEUE, action, default_value = "false", required = false)]
    pub queue: bool,
    #[arg(long, help = about::MAX_CONCURRENT)]
    pub queue_max_concurrent: Option<usize>,
    #[arg(long, help = about::MAX_ITERATIONS)]
    pub queue_max_iterations: Option<usize>,
    #[arg(long, help = about::STATUS_CHECK_SLEEP_SECONDS)]
    pub queue_status_check_sleep_seconds: Option<u64>,
}

fn set_default_args(args: &mut Attributes, default: &DefaultArgs) {
    if let Some(message) = default.message.clone() {
        args.message = message;
    }
    if let Some(target_addrs) = default.target_addrs.clone() {
        args.target_addrs = target_addrs;
    }
    if let Some(replace_addrs) = default.replace_addrs.clone() {
        args.replace_addrs = replace_addrs;
    }
    if let Some(terraform_version) = default.terraform_version.clone() {
        args.terraform_version = Some(terraform_version);
    }
    if let Some(save_plan) = default.save_plan {
        args.save_plan = Some(save_plan);
    }
}

fn override_queue_options(options: &mut QueueOptions, default: &DefaultArgs) {
    if let Some(max_concurrent) = default.queue_max_concurrent {
        options.max_concurrent = max_concurrent;
    }
    if let Some(max_iterations) = default.queue_max_iterations {
        options.max_iterations = max_iterations;
    }
    if let Some(status_check_sleep_seconds) =
        default.queue_status_check_sleep_seconds
    {
        options.status_check_sleep_seconds = status_check_sleep_seconds;
    }
}

fn set_apply_args(args: &mut Attributes, apply: &ApplyArgs) {
    if let Some(auto_apply) = apply.auto_apply {
        args.auto_apply = Some(auto_apply);
    }
    if let Some(allow_empty_apply) = apply.allow_empty_apply {
        args.allow_empty_apply = allow_empty_apply;
    }
    if let Some(is_destroy) = apply.is_destroy {
        args.is_destroy = Some(is_destroy);
    }
    if let Some(refresh_only) = apply.refresh_only {
        args.refresh_only = Some(refresh_only);
    }
}
