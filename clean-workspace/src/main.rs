mod error;
mod parse;
mod repo;
mod report;
mod settings;

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use settings::Settings;
use tfc_toolset::{
    error::SETTINGS_ERROR,
    filter,
    settings::Core,
    variable,
    workspace::{self, VcsRepo},
};
use tfc_toolset_extras::default_client;
use url::Url;
use walkdir::WalkDir;

use crate::report::UnlistedVariables;

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
    let client = default_client()?;
    // Match on the cli subcommand
    match &cli.command {
        Commands::Plan => {
            info!("Start Plan Phase");
            let mut report = report::new();
            report.meta.query = Some(core.query.clone());
            report.meta.pagination = Some(core.pagination.clone());

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
                        &core,
                        client,
                        workspaces.clone(),
                    )
                    .await?;
                // Filter the workspaces if query variables have been provided
                if core.query.variables.is_some() {
                    info!("Filtering workspaces with variable query.");
                    filter::variable(&mut workspaces_variables, &core)?;
                }

                workspaces.clear();
                for ws in &workspaces_variables {
                    workspaces.push(ws.workspace.clone());
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

            report.data.workspaces = workspaces;
            info!("{:#?}", &report);
            report.save(&core)?;
        }
        Commands::Apply => {
            dbg!("apply");
        }
    }
    Ok(())
}
