use super::about;
use crate::settings::Settings;

use clap::Args;
use log::info;
use miette::IntoDiagnostic;
use surf::Client;
use tfc_toolset::{
    settings::Core,
    workspace::{self, Workspace},
};
use tfc_toolset_extras::WorkspacesFile;

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(
    short,
    long,
    action,
    help = about::FILTER_LIST,
    default_value = "false",
    required = false
    )]
    pub filter: bool,
}

pub async fn list(
    args: &ListArgs,
    core: &Core,
    config: &Settings,
    client: Client,
) -> miette::Result<Vec<Workspace>> {
    info!("Retrieving Workspaces.");
    let workspaces =
        workspace::list(args.filter, core, client).await.into_diagnostic()?;
    info!("{:#?}", &workspaces);
    if core.save_output {
        WorkspacesFile::from(workspaces.clone())
            .save(&core.output, config.pretty_output)
            .into_diagnostic()?;
    }
    Ok(workspaces)
}
