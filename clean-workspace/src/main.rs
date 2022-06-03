mod error;
mod parse;
mod repo;
mod report;
mod settings;

use clap::{Parser, Subcommand};
use env_logger::Env;
use http_cache_surf::{
    CACacheManager, Cache, CacheMode, CacheOptions, HttpCache,
};
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use settings::Settings;
use surf::Client;
use surf_governor::GovernorMiddleware;
use surf_retry::{ExponentialBackoff, RetryMiddleware};
use tfc_toolset::{
    error::{ToolError, SETTINGS_ERROR},
    filter,
    settings::Core,
    variable,
    workspace::{self, VcsRepo},
};
use url::Url;
use walkdir::WalkDir;

use crate::report::{Meta, UnlistedVariables};

const ABOUT: &str =
    "Tool for rule based cleanup operations for Terraform workspaces";
const ABOUT_PLAN: &str = "Generates a report that contains information on the actions required to cleanup a workspace based on the provided rules";
const ABOUT_APPLY: &str =
    "Executes the actions described in the previously generated report";

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
    Plan,
    #[clap(about = ABOUT_APPLY)]
    Apply,
}

fn build_governor() -> Result<GovernorMiddleware, ToolError> {
    match GovernorMiddleware::per_second(30) {
        Ok(g) => Ok(g),
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

fn build_path(config: &Settings, vcs: &VcsRepo, url: Url) -> String {
    let id = match vcs.identifier.clone() {
        Some(i) => i,
        None => {
            let segments = url.path_segments().unwrap();
            segments.last().unwrap().to_string()
        }
    };
    let mut base_dir = config.repositories.git_dir.clone();
    if base_dir.ends_with('/') {
        base_dir.pop();
    }
    format!("{}/{}", base_dir, &id)
}

fn repo_url(vcs: &VcsRepo) -> miette::Result<Url> {
    Url::parse(&vcs.repository_http_url)
        .into_diagnostic()
        .wrap_err("Failed to parse repository url")
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
    // Build the http client with a cache, governor, and retry enabled
    let retry = RetryMiddleware::new(
        99,
        ExponentialBackoff::builder().build_with_max_retries(10),
        1,
    );
    let client = Client::new()
        .with(retry)
        .with(build_governor().into_diagnostic()?)
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::default(),
            options: Some(CacheOptions {
                shared: false,
                cache_heuristic: 0.0,
                immutable_min_time_to_live: Default::default(),
                ignore_cargo_cult: false,
            }),
        }));
    // Match on the cli subcommand
    match &cli.command {
        Commands::Plan => {
            info!("Start Plan Phase");
            let mut report = report::Report {
                meta: Meta {
                    query: Some(core.query.clone()),
                    pagination: Some(core.pagination.clone()),
                },
                ..Default::default()
            };
            // Get list of workspaces
            let mut workspaces =
                workspace::get_workspaces(&core, client.clone()).await?;

            // Filter the workspaces if query tags have been provided
            if core.query.tags.is_some() {
                info!("Filtering workspaces with tags query.");
                filter::tag(&mut workspaces, &core)?;
            }

            if core.query.variables.is_some()
                || config.cleanup.unlisted_variables
                || config.cleanup.missing_repositories
            {
                // Get the variables for each workspace
                let mut workspaces_variables =
                    workspace::get_workspaces_variables(
                        &core, client, workspaces,
                    )
                    .await?;
                // Filter the workspaces if query variables have been provided
                if core.query.variables.is_some() {
                    info!("Filtering workspaces with variable query.");
                    filter::variable(&mut workspaces_variables, &core)?;
                }

                if config.cleanup.unlisted_variables
                    || config.cleanup.missing_repositories
                {
                    info!("Cloning workspace repositories.");
                    // First let's clean up the job list to remove duplicates
                    let mut repos: Vec<VcsRepo> = vec![];
                    let mut missing: Vec<VcsRepo> = vec![];
                    let mut detected_variables: Vec<parse::ParseResult> =
                        vec![];
                    for entry in &workspaces_variables {
                        if let Some(vcs) = &entry.workspace.attributes.vcs_repo
                        {
                            if !repos.contains(vcs) {
                                repos.push(vcs.clone());
                                let url = repo_url(vcs)?;
                                let path =
                                    build_path(&config, vcs, url.clone());
                                match repo::clone(
                                    url.clone(),
                                    path.clone(),
                                    vcs,
                                    &mut missing,
                                ) {
                                    Ok(_) => {}
                                    Err(_e) => {}
                                };
                                if config.cleanup.unlisted_variables {
                                    info!("Parsing variable data.");
                                    let url = repo_url(vcs)?;
                                    let path =
                                        build_path(&config, vcs, url.clone());
                                    let walker =
                                        WalkDir::new(&path).into_iter();
                                    detected_variables.push(parse::tf_repo(
                                        &config, walker, vcs,
                                    )?);
                                }
                            }
                        }
                    }
                    for entry in &workspaces_variables {
                        if let Some(vcs) = &entry.workspace.attributes.vcs_repo
                        {
                            info!(
                                "Repo detected for workspace: {}",
                                &entry.workspace.attributes.name
                            );
                            if config.cleanup.missing_repositories
                                && missing.contains(vcs)
                            {
                                if let Some(m) =
                                    &mut report.data.missing_repositories
                                {
                                    m.push(entry.workspace.clone());
                                } else {
                                    report.data.missing_repositories =
                                        Some(vec![entry.workspace.clone()]);
                                }
                            }

                            if config.cleanup.unlisted_variables {
                                let mut found: Vec<variable::Variable> = vec![];
                                let mut unlisted: Option<UnlistedVariables> =
                                    None;
                                for var in &entry.variables {
                                    for detected in &detected_variables {
                                        if &detected.vcs == vcs {
                                            for dv in
                                                &detected.detected_variables
                                            {
                                                if &var.attributes.key == dv {
                                                    found.push(var.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                                debug!(
                                    "TFC Variables: {:#?}",
                                    &entry.variables
                                );
                                debug!("Found Variables: {:#?}", &found);
                                let difference: Vec<_> = entry
                                    .variables
                                    .clone()
                                    .into_iter()
                                    .filter(|item| !found.contains(item))
                                    .collect();
                                debug!(
                                    "Variable Difference: {:#?}",
                                    &difference
                                );
                                if !difference.is_empty() {
                                    let mut un = UnlistedVariables {
                                        workspace: entry.clone(),
                                        unlisted_variables: vec![],
                                    };
                                    for var in difference {
                                        un.unlisted_variables.push(var.into())
                                    }
                                    unlisted = Some(un);
                                }

                                if let Some(u) = unlisted {
                                    if let Some(v) =
                                        &mut report.data.unlisted_variables
                                    {
                                        v.push(u);
                                    } else {
                                        report.data.unlisted_variables =
                                            Some(vec![u]);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            info!("{:#?}", &report);
            report::save(&core, report)?;
        }
        Commands::Apply => {
            dbg!("apply");
        }
    }
    Ok(())
}
