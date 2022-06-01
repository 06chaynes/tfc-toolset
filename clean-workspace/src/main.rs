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
use tfc_toolset::{
    error::{ToolError, SETTINGS_ERROR},
    filter,
    settings::Core,
    workspace::{self, Workspace},
};
use url::Url;
use walkdir::WalkDir;

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
    // Build the http client with a cache and governor enabled
    let client = Client::new().with(build_governor().into_diagnostic()?).with(
        Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager::default(),
            options: Some(CacheOptions {
                shared: false,
                cache_heuristic: 0.0,
                immutable_min_time_to_live: Default::default(),
                ignore_cargo_cult: false,
            }),
        }),
    );
    // Match on the cli subcommand
    match &cli.command {
        Commands::Plan => {
            info!("Start Plan Phase");
            let mut report = report::Report {
                query: Some(core.query.clone()),
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
                    for entry in &workspaces_variables {
                        report.workspaces.push(entry.workspace.clone());
                        // Clone git repositories
                        if let Some(repo) = &entry.workspace.attributes.vcs_repo
                        {
                            info!(
                                "Repo detected for workspace: {}",
                                &entry.workspace.attributes.name
                            );
                            let url = Url::parse(&repo.repository_http_url)
                                .into_diagnostic()
                                .wrap_err("Failed to parse repository url")?;
                            let id = match repo.identifier.clone() {
                                Some(i) => i,
                                None => {
                                    let segments = url.path_segments().unwrap();
                                    segments.last().unwrap().to_string()
                                }
                            };
                            let mut base_dir =
                                config.repositories.git_dir.clone();
                            if base_dir.ends_with('/') {
                                base_dir.pop();
                            }
                            let path = format!("{}/{}", base_dir, &id);
                            let mut missing: Vec<Workspace> = vec![];
                            match repo::clone(
                                url.clone(),
                                path.clone(),
                                &entry.workspace,
                                &mut missing,
                            ) {
                                Ok(_) => {}
                                Err(_e) => {}
                            };
                            if config.cleanup.missing_repositories {
                                if let Some(m) =
                                    &mut report.missing_repositories
                                {
                                    m.append(&mut missing);
                                } else {
                                    report.missing_repositories = Some(missing);
                                }
                            }
                            info!("Parsing variable data.");
                            if config.cleanup.unlisted_variables {
                                let walker = WalkDir::new(&path).into_iter();
                                let unlisted =
                                    parse::tf(&config, walker, entry)?;
                                if let Some(u) = unlisted {
                                    if let Some(v) =
                                        &mut report.unlisted_variables
                                    {
                                        v.push(u);
                                    } else {
                                        report.unlisted_variables =
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
