mod about;

pub(crate) mod workspace;

pub use workspace::workspace;

use crate::{
    cli::command::common::WorkspaceArgs, error::CleanError, settings::Settings,
};
use async_scoped::AsyncScope;
use clap::{Args, Subcommand};
use git2::{build::RepoBuilder, ErrorCode, Repository};
use git2_credentials::CredentialHandler;
use log::{error, info, warn};
use miette::{IntoDiagnostic, WrapErr};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tfc_toolset::{
    error::ToolError,
    workspace::{VcsRepo, WorkspaceVariables},
};
use url::Url;
use walkdir::{DirEntry, IntoIter, WalkDir};

#[derive(Args, Debug)]
pub(crate) struct Commands {
    #[command(subcommand)]
    pub command: CleanCmds,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CleanCmds {
    #[clap(about = about::WORKSPACE)]
    Workspace(CleanWorkspaceArgs),
}

#[derive(Args, Debug)]
pub struct CleanWorkspaceArgs {
    #[clap(flatten)]
    pub workspace: WorkspaceArgs,
    #[arg(short, long, help = about::DRY_RUN, default_value = "true")]
    pub dry_run: Option<bool>,
    #[arg(long, help = about::GIT_DIR)]
    pub git_dir: Option<PathBuf>,
    #[arg(short, long, help = about::UNLISTED_VARIABLES, default_value = "true")]
    pub unlisted_variables: Option<bool>,
    #[arg(short, long, help = about::MISSING_REPOSITORIES, default_value = "false")]
    pub missing_repositories: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestVariable {
    pub variable: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variable {
    pub variable: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParseResult {
    pub vcs: VcsRepo,
    pub detected_variables: Vec<String>,
}

pub struct ProcessResults {
    pub repos: Vec<VcsRepo>,
    pub missing: Vec<VcsRepo>,
    pub failed: Vec<VcsRepo>,
    pub detected_variables: Vec<ParseResult>,
}

pub struct ProcessResult {
    pub missing: Option<VcsRepo>,
    pub failed: Option<VcsRepo>,
    pub detected_variables: Option<ParseResult>,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false)
}

fn is_tf(entry: &DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.ends_with("tf")).unwrap_or(false)
}

pub(super) fn tf_repo(
    _config: &Settings,
    walker: IntoIter,
    vcs: &VcsRepo,
) -> Result<ParseResult, ToolError> {
    let mut detected_variables: Vec<String> = vec![];
    for file in walker
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
        .filter(is_tf)
    {
        info!("Parsing file: {}", file.path().display());
        match hcl::from_str::<TestVariable>(&fs::read_to_string(file.path())?) {
            Ok(v) => {
                info!("{:#?}", &v);
                if let Some(value) = v.variable {
                    for (key, _value) in value.as_object().unwrap() {
                        detected_variables.push(key.clone());
                    }
                }
            }
            Err(_e) => {
                match hcl::from_str::<Variable>(&fs::read_to_string(
                    file.path(),
                )?) {
                    Ok(value) => {
                        for (key, _value) in value.variable.as_object().unwrap()
                        {
                            detected_variables.push(key.clone());
                        }
                    }
                    Err(e) => {
                        warn!("Error parsing file: {}", file.path().display());
                        warn!("{:#?}", e);
                    }
                }
            }
        }
    }
    Ok(ParseResult { vcs: vcs.clone(), detected_variables })
}

fn build_path(config: &Settings, vcs: &VcsRepo, url: Url) -> String {
    let mut id = match vcs.identifier.clone() {
        Some(i) => i,
        None => {
            let segments = url.path_segments().unwrap();
            segments.last().unwrap().to_string()
        }
    };
    if let Some(branch) = &vcs.branch {
        if !branch.is_empty() {
            id = format!("{}_{}", &id, &branch);
        }
    }
    let mut base_dir = config.cleanup.repositories.git_dir.clone();
    if base_dir.to_str().unwrap().ends_with('/') {
        base_dir.pop();
    }
    format!("{}/{}", base_dir.display(), &id)
}

fn repo_url(vcs: &VcsRepo) -> miette::Result<Url> {
    Url::parse(&vcs.repository_http_url.clone().expect("No repository url"))
        .into_diagnostic()
        .wrap_err("Failed to parse repository url")
}

pub(super) fn fetch_remote(
    url: Url,
    path: String,
    vcs: &VcsRepo,
) -> Result<Option<VcsRepo>, CleanError> {
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default().unwrap();
    let mut ch = CredentialHandler::new(git_config);
    cb.credentials(move |url, username, allowed| {
        ch.try_next_credential(url, username, allowed)
    });
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb).update_fetchhead(true);

    info!("Cloning repo: {} into {}", url.as_str(), &path);
    let path = Path::new(&path);
    if path.is_dir() {
        info!("Repo already exists locally, updating instead.");
        // Open and Update
        match Repository::open(path) {
            Ok(repo) => {
                info!("Updating repo.");
                match repo.checkout_head(None) {
                    Ok(_) => info!("Repo at HEAD!"),
                    Err(e) => {
                        error!("Checkout HEAD failed :(");
                        return Err(CleanError::Git(e));
                    }
                }
            }
            Err(e) => {
                error!("Open failed :(");
                error!("{:#?}", &e);
                return Err(CleanError::Git(e));
            }
        }
    } else {
        // Clone
        let mut builder = RepoBuilder::new();
        if let Some(branch) = &vcs.branch {
            if !branch.is_empty() {
                builder.branch(branch.as_str());
            }
        }
        match builder.fetch_options(fo).clone(url.as_str(), path) {
            Ok(_repo) => info!("Clone successful!"),
            Err(e) => {
                error!("Clone failed :(");
                error!("{:#?}", &e);
                match e.code() {
                    ErrorCode::NotFound => {
                        return Ok(Some(vcs.clone()));
                    }
                    ErrorCode::GenericError => {
                        if e.message() == "unexpected http status code: 404" {
                            return Ok(Some(vcs.clone()));
                        }
                    }
                    _ => return Err(CleanError::Git(e)),
                }
            }
        }
    }
    Ok(None)
}

pub(super) fn process(
    config: &Settings,
    workspaces_variables: &Vec<WorkspaceVariables>,
) -> miette::Result<ProcessResults, ToolError> {
    let mut repos: Vec<VcsRepo> = vec![];
    for entry in workspaces_variables {
        if let Some(vcs) = &entry.workspace.attributes.vcs_repo {
            if !repos.contains(vcs) {
                repos.push(vcs.clone());
            }
        }
    }
    // This doesn't actually need to be async but it's easier to just have it match
    let (_, process_result_vec) = AsyncScope::scope_and_block(|s| {
        for vcs in &repos {
            let proc = || async move {
                let mut result = ProcessResult {
                    missing: None,
                    failed: None,
                    detected_variables: None,
                };
                match repo_url(vcs) {
                    Ok(url) => {
                        let path = build_path(config, vcs, url.clone());
                        match fetch_remote(url.clone(), path, vcs) {
                            Ok(r) => {
                                if let Some(missing) = r {
                                    result.missing = Some(missing);
                                }
                            }
                            Err(_e) => {
                                result.failed = Some(vcs.clone());
                            }
                        };
                        if config.cleanup.unlisted_variables {
                            info!("Parsing variable data.");
                            let path = build_path(config, vcs, url);
                            let walker = WalkDir::new(path).into_iter();
                            let detected = tf_repo(config, walker, vcs).ok();
                            result.detected_variables = detected;
                        }
                    }
                    Err(_e) => {}
                }
                result
            };
            s.spawn(proc());
        }
    });

    let mut results = ProcessResults {
        repos,
        missing: vec![],
        failed: vec![],
        detected_variables: vec![],
    };
    for res in process_result_vec {
        if let Some(m) = res.missing {
            results.missing.push(m);
        }
        if let Some(f) = res.failed {
            results.failed.push(f);
        }
        if let Some(v) = res.detected_variables {
            results.detected_variables.push(v);
        }
    }

    Ok(results)
}
