use super::about;
use crate::error::ArgError;

use clap::Args;
use log::info;
use surf::Client;
use tfc_toolset::settings::Core;

#[derive(Args, Debug)]
pub struct CancelArgs {
    #[arg(short = 'i', long, help = about::RUN_ID)]
    pub run_id: String,
}

pub async fn cancel(
    args: &CancelArgs,
    config: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    info!("Cancelling run: {}", args.run_id);
    tfc_toolset::run::cancel(&args.run_id.clone(), config, client.clone())
        .await?;
    Ok(())
}
