use std::path::Path;

use crate::{error::CleanError, parse, settings::Settings};
use async_scoped::AsyncScope;
use git2::{build::RepoBuilder, ErrorCode, Repository};
use git2_credentials::CredentialHandler;
use log::*;
use miette::{IntoDiagnostic, WrapErr};
use tfc_toolset::workspace::{VcsRepo, WorkspaceVariables};
use url::Url;
use walkdir::WalkDir;

pub struct ProcessResults {
    pub repos: Vec<VcsRepo>,
    pub missing: Vec<VcsRepo>,
    pub failed: Vec<VcsRepo>,
    pub detected_variables: Vec<parse::ParseResult>,
}

pub struct ProcessResult {
    pub missing: Option<VcsRepo>,
    pub failed: Option<VcsRepo>,
    pub detected_variables: Option<parse::ParseResult>,
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
    let mut base_dir = config.repositories.git_dir.clone();
    if base_dir.ends_with('/') {
        base_dir.pop();
    }
    format!("{}/{}", base_dir, &id)
}

fn repo_url(vcs: &VcsRepo) -> miette::Result<Url> {
    Url::parse(&vcs.repository_http_url.clone().expect("No repository url"))
        .into_diagnostic()
        .wrap_err("Failed to parse repository url")
}

pub fn fetch_remote(
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

pub fn process(
    config: &Settings,
    workspaces_variables: &Vec<WorkspaceVariables>,
) -> miette::Result<ProcessResults> {
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
                            let detected =
                                parse::tf_repo(config, walker, vcs).ok();
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
