mod report;
mod settings;

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use settings::Settings;
use tfc_toolset::{error::SETTINGS_ERROR, run, settings::Core, workspace};
use tfc_toolset_extras::default_client;

const ABOUT: &str =
    "A tool for creating multiple Terraform Cloud runs in parallel with provided specifications";
const ABOUT_PLAN: &str =
    "Queues up plan only runs for the workspaces determined by filters";
const ABOUT_APPLY: &str =
    "Queues up plan and apply runs for the workspaces determined by filters";
const ABOUT_MESSAGE: &str = "A message to include with the run";
const ABOUT_TARGET_ADDRS: &str =
    "A list of resource addresses to target for the run";
const ABOUT_REPLACE_ADDRS: &str =
    "A list of resource addresses to replace for the run";
const ABOUT_AUTO_APPLY: &str =
    "Automatically apply the run if the plan is successful";
const ABOUT_ALLOW_EMPTY_APPLY: &str =
    "Apply the run even when the plan contains no changes";
const ABOUT_IS_DESTROY: &str =
    "Whether this plan is a destroy plan that will destroy all provisioned resources";
const ABOUT_REFRESH_ONLY: &str =
    "Whether this run should refresh the state without modifying any resources";
const ABOUT_SAVE_PLAN: &str =
    "Specifies if this should be a saved plan run which can be applied later";
const ABOUT_TERRAFORM_VERSION: &str =
    "The version of Terraform to use for this run, overriding the value from settings";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = Some(ABOUT))]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(about = ABOUT_PLAN)]
    Plan(DefaultArgs),
    #[clap(about = ABOUT_APPLY)]
    Apply(ApplyArgs),
}

#[derive(clap::Args, Debug)]
struct DefaultArgs {
    #[arg(long, help = ABOUT_MESSAGE, default_value = "Run created by tfc-toolset")]
    pub message: Option<String>,
    #[arg(long, help = ABOUT_TARGET_ADDRS)]
    pub target_addrs: Option<Vec<String>>,
    #[arg(long, help = ABOUT_REPLACE_ADDRS)]
    pub replace_addrs: Option<Vec<String>>,
    #[arg(long, help = ABOUT_TERRAFORM_VERSION)]
    pub terraform_version: Option<String>,
    #[arg(long, help = ABOUT_SAVE_PLAN, default_value = "false")]
    pub save_plan: Option<bool>,
}

#[derive(clap::Args, Debug)]
struct ApplyArgs {
    #[clap(flatten)]
    pub default: DefaultArgs,
    #[arg(long, help = ABOUT_AUTO_APPLY, default_value = "false")]
    pub auto_apply: Option<bool>,
    #[arg(
        long,
        help = ABOUT_ALLOW_EMPTY_APPLY,
        default_value = "false"
    )]
    pub allow_empty_apply: Option<bool>,
    #[arg(long, help = ABOUT_IS_DESTROY, default_value = "false")]
    pub is_destroy: Option<bool>,
    #[arg(long, help = ABOUT_REFRESH_ONLY, default_value = "false")]
    pub refresh_only: Option<bool>,
}

fn set_default_args(args: &mut run::Attributes, default: &DefaultArgs) {
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

fn set_apply_args(args: &mut run::Attributes, apply: &ApplyArgs) {
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

#[async_std::main]
async fn main() -> miette::Result<()> {
    // Parse cli subcommands and arguments
    let cli = Cli::parse();
    // Get the settings for the run
    let core = Core::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
    let config = Settings::new().into_diagnostic().wrap_err(SETTINGS_ERROR)?;
    // Initialize the logger
    env_logger::Builder::from_env(Env::default().default_filter_or(&core.log))
        .init();
    let max_concurrent = config
        .max_concurrent
        .unwrap_or(settings::MAX_CONCURRENT_DEFAULT.into());
    let max_iterations = config
        .max_iterations
        .unwrap_or(settings::MAX_ITERATIONS_DEFAULT.into());
    let status_check_sleep_seconds = config
        .status_check_sleep_seconds
        .unwrap_or(settings::STATUS_CHECK_SLEEP_SECONDS_DEFAULT);
    let client = default_client(None).into_diagnostic()?;
    // Match on the cli subcommand
    match &cli.command {
        Commands::Plan(args) => {
            info!("Start Plan Only Runs");
            let mut report = report::new();
            report.meta.query = core.workspaces.query.clone();
            report.meta.pagination = Some(core.pagination.clone());

            // Get filtered list of workspaces
            let workspaces = workspace::list(true, &core, client.clone())
                .await
                .into_diagnostic()?;

            // Queue up plan runs for each workspace respecting the max_concurrent setting
            let mut attributes = run::Attributes {
                plan_only: Some(true),
                terraform_version: Some(core.terraform_version.clone()),
                ..Default::default()
            };
            if let Some(save_plan) = args.save_plan {
                if save_plan {
                    attributes.plan_only = None;
                    attributes.terraform_version = None;
                }
            }
            set_default_args(&mut attributes, args);

            let queue_results = run::work_queue(
                workspaces.clone(),
                run::QueueOptions {
                    max_concurrent,
                    max_iterations,
                    status_check_sleep_seconds,
                },
                attributes,
                client.clone(),
                &core,
            )
            .await
            .into_diagnostic()?;

            report.data.workspaces = workspaces;
            report.data.runs = queue_results.results;
            report.errors.runs = queue_results.errors;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
        Commands::Apply(args) => {
            info!("Start Plan and Apply Runs");
            let mut report = report::new();
            report.meta.query = core.workspaces.query.clone();
            report.meta.pagination = Some(core.pagination.clone());

            // Get filtered list of workspaces
            let workspaces = workspace::list(true, &core, client.clone())
                .await
                .into_diagnostic()?;

            // Queue up plan runs for each workspace respecting the max_concurrent setting
            let mut attributes = run::Attributes::default();
            set_default_args(&mut attributes, &args.default);
            set_apply_args(&mut attributes, args);

            let queue_results = run::work_queue(
                workspaces.clone(),
                run::QueueOptions {
                    max_concurrent,
                    max_iterations,
                    status_check_sleep_seconds,
                },
                attributes,
                client.clone(),
                &core,
            )
            .await
            .into_diagnostic()?;

            report.data.workspaces = workspaces;
            report.data.runs = queue_results.results;
            report.errors.runs = queue_results.errors;
            debug!("{:#?}", &report);
            report.save(&core).into_diagnostic()?;
        }
    }
    Ok(())
}
