use super::{build_options, DefaultArgs};

use crate::{error::ArgError, settings::Settings};
use log::{debug, info};
use miette::IntoDiagnostic;
use surf::Client;
use tfc_toolset::{
    settings::Core,
    workspace::{self, Workspace},
};
use tfc_toolset_extras::WorkspacesFile;

pub async fn create(
    args: &DefaultArgs,
    core: &Core,
    config: &Settings,
    client: Client,
) -> miette::Result<Workspace> {
    if args.name.is_none() {
        return Err(ArgError::MissingWorkspaceName.into());
    }
    info!("Creating Workspace: {}", &args.name.clone().unwrap());
    let options = build_options(args)?;
    debug!("{:#?}", &options);
    let workspace =
        workspace::create(options, core, client).await.into_diagnostic()?;
    info!("{:#?}", &workspace);
    if core.save_output {
        WorkspacesFile::from(vec![workspace.clone()])
            .save(&core.output, config.pretty_output)
            .into_diagnostic()?;
    }
    Ok(workspace)
}
