use super::about;
use crate::error::ArgError;

use clap::Args;
use log::info;
use surf::Client;
use tfc_toolset::{run::Run, settings::Core};

#[derive(Args, Debug)]
pub struct StatusArgs {
    #[arg(short = 'i', long, help = about::RUN_ID)]
    pub run_id: String,
}

pub async fn status(
    args: &StatusArgs,
    config: &Core,
    client: Client,
) -> miette::Result<Run, ArgError> {
    info!("Retrieving status for run: {}", args.run_id);
    let run =
        tfc_toolset::run::status(&args.run_id.clone(), config, client.clone())
            .await?;
    info!("{:#?}", &run);
    Ok(run)
}
