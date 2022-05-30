use std::fs::File;
use tfc_toolset::{
    error::ToolError,
    settings::{Core, Query},
    workspace::Workspace,
};

use log::*;
use serde::{Deserialize, Serialize};

// For now need to keep this updated with best effort :)
const REPORT_VERSION: &str = "0.1.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Report {
    pub report_version: String,
    pub bin_version: String,
    pub query: Option<Query>,
    pub workspaces: Vec<Workspace>,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            report_version: REPORT_VERSION.to_string(),
            bin_version: env!("CARGO_PKG_VERSION").to_string(),
            query: None,
            workspaces: vec![],
        }
    }
}

pub fn save(config: &Core, report: Report) -> Result<(), ToolError> {
    info!("Saving report to: {}", &config.output);
    match serde_json::to_writer_pretty(&File::create(&config.output)?, &report)
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
