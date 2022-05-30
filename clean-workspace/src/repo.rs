use std::path::Path;

use crate::error::CleanError;
use git2::{build::RepoBuilder, ErrorCode, Repository};
use git2_credentials::CredentialHandler;
use log::*;
use tfc_toolset::workspace::Workspace;
use url::Url;

pub fn clone(
    url: Url,
    path: String,
    workspace: &Workspace,
    missing_repos: &mut Vec<Workspace>,
) -> Result<(), CleanError> {
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
                match e.code() {
                    ErrorCode::NotFound => {
                        missing_repos.push(workspace.clone());
                    }
                    _ => return Err(CleanError::Git(e)),
                }
            }
        }
    } else {
        // Clone
        match RepoBuilder::new().fetch_options(fo).clone(url.as_str(), path) {
            Ok(_repo) => info!("Clone successful!"),
            Err(e) => {
                error!("Clone failed :(");
                return Err(CleanError::Git(e));
            }
        }
    }
    Ok(())
}
