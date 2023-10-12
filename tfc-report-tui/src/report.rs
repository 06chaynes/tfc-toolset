use serde::{Deserialize, Serialize};
use std::fs;
use tfc_clean_workspace::report::CleanReport;
use tfc_run_workspace::report::RunReport;
use tfc_toolset::error::ToolError;
use tfc_toolset_extras::report::{Report, Reporter};
use tfc_which_workspace::report::WhichReport;

const REPORT_PATH: &str = "./report.json";

#[derive(Debug, Deserialize, Serialize)]
struct Empty {}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TfcReport {
    Clean(CleanReport),
    Which(WhichReport),
    Run(RunReport),
}

impl From<CleanReport> for TfcReport {
    fn from(item: CleanReport) -> Self {
        TfcReport::Clean(item)
    }
}

impl From<WhichReport> for TfcReport {
    fn from(item: WhichReport) -> Self {
        TfcReport::Which(item)
    }
}

impl From<RunReport> for TfcReport {
    fn from(item: RunReport) -> Self {
        TfcReport::Run(item)
    }
}

pub fn read() -> Result<TfcReport, ToolError> {
    let db_content = fs::read_to_string(REPORT_PATH)?;
    // For now we need to do this to check the report type before we try to deserialize the data
    let parsed: Report<Empty, Empty, Empty> =
        serde_json::from_str(&db_content)?;
    match parsed.reporter {
        Reporter::CleanWorkspace => {
            let parsed: CleanReport = serde_json::from_str(&db_content)?;
            Ok(parsed.into())
        }
        Reporter::WhichWorkspace => {
            let parsed: WhichReport = serde_json::from_str(&db_content)?;
            Ok(parsed.into())
        }
        Reporter::RunWorkspace => {
            let parsed: RunReport = serde_json::from_str(&db_content)?;
            Ok(parsed.into())
        }
        _ => unreachable!(),
    }
}
