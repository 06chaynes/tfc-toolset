use std::fs::File;
use tfc_toolset::{error::ToolError, settings::Core};

use log::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Report<M, D> {
    pub report_version: String,
    pub bin_version: String,
    pub reporter: String,
    pub meta: M,
    pub data: D,
}

impl<'de, M, D> Report<M, D>
where
    M: Serialize + Deserialize<'de>,
    D: Serialize + Deserialize<'de>,
{
    pub fn save(&self, config: &Core) -> Result<(), ToolError> {
        info!("Saving report to: {}", &config.output);
        match serde_json::to_writer_pretty(&File::create(&config.output)?, self)
        {
            Ok(_) => {
                info!("Report Saved!");
            }
            Err(e) => {
                error!("Failed to save report!");
                return Err(ToolError::Json(e));
            }
        }
        Ok(())
    }
}
