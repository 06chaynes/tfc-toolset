use crate::{
    cli::{
        command::common::{check_workspace_identifier, parse_workspace_file},
        run::{
            override_queue_options, set_apply_args, set_default_args, PlanArgs,
        },
    },
    error::ArgError,
    settings::{self, Settings},
};

use std::{fs::File, io::BufReader, path::Path};

use log::info;
use surf::Client;
use tfc_toolset::{
    error::ToolError,
    run::{work_queue, Attributes, QueueOptions, QueueResult, RunResult},
    settings::Core,
    workspace,
};
use tfc_toolset_extras::{parse_workspace_name, ExtrasError};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct QueueRunResult {
    pub results: Vec<RunResult>,
    pub errors: Vec<RunResult>,
}

impl QueueRunResult {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ExtrasError> {
        let file = File::open(path).map_err(ToolError::Io)?;
        let reader = BufReader::new(file);
        let results_file: Self =
            serde_json::from_reader(reader).map_err(ToolError::Json)?;
        Ok(results_file)
    }

    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        pretty: bool,
    ) -> Result<(), ToolError> {
        if pretty {
            serde_json::to_writer_pretty(&File::create(path)?, self)?;
        } else {
            serde_json::to_writer(&File::create(path)?, self)?;
        }
        Ok(())
    }
}

impl From<QueueResult> for QueueRunResult {
    fn from(queue_result: QueueResult) -> Self {
        Self { results: queue_result.results, errors: queue_result.errors }
    }
}

pub async fn plan(
    args: &PlanArgs,
    config: &Settings,
    core: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    check_workspace_identifier(&args.default.workspace)?;
    let max_concurrent = config
        .run
        .max_concurrent
        .unwrap_or(settings::MAX_CONCURRENT_DEFAULT.into());
    let max_iterations = config
        .run
        .max_iterations
        .unwrap_or(settings::MAX_ITERATIONS_DEFAULT.into());
    let status_check_sleep_seconds = config
        .run
        .status_check_sleep_seconds
        .unwrap_or(settings::STATUS_CHECK_SLEEP_SECONDS_DEFAULT);
    let cancel_on_timeout = config.run.cancel_on_timeout.unwrap_or(false);
    let mut options = QueueOptions {
        max_concurrent,
        max_iterations,
        status_check_sleep_seconds,
        cancel_on_timeout,
    };
    override_queue_options(&mut options, &args.default);
    let mut attributes = Attributes::default();
    set_default_args(&mut attributes, &args.default);
    set_apply_args(&mut attributes, args);

    if let Some(workspace_name) = &args.default.workspace.workspace_name {
        parse_workspace_name(workspace_name)?;
        let workspace =
            workspace::show_by_name(workspace_name, core, client.clone())
                .await?;
        info!("Creating plan run for workspace {}.", workspace_name);
        if args.default.queue {
            let queue_results = work_queue(
                vec![workspace.clone()],
                options,
                attributes,
                client.clone(),
                core,
            )
            .await?;
            info!("{:#?}", &queue_results);
        } else {
            let run = tfc_toolset::run::create(
                &workspace.id,
                Some(attributes),
                core,
                client.clone(),
            )
            .await?;
            info!("{:#?}", &run);
        }
    } else if let Some(workspace_id) = &args.default.workspace.workspace_id {
        let workspace =
            workspace::show(workspace_id, core, client.clone()).await?;
        info!("Creating plan run for workspace {}.", workspace_id);
        if args.default.queue {
            let queue_results = work_queue(
                vec![workspace.clone()],
                options,
                attributes,
                client.clone(),
                core,
            )
            .await?;
            info!("{:#?}", &queue_results);
        } else {
            let run = tfc_toolset::run::create(
                workspace_id,
                Some(attributes),
                core,
                client.clone(),
            )
            .await?;
            info!("{:#?}", &run);
        }
    } else if let Some(file_path) = &args.default.workspace.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, core, client.clone()).await?;
        let queue_results =
            work_queue(workspaces, options, attributes, client.clone(), core)
                .await?;
        info!("{:#?}", &queue_results);
        if core.save_output {
            let queue_run_result = QueueRunResult::from(queue_results);
            queue_run_result.save(&core.output, config.pretty_output)?;
        }
    } else if args.default.workspace.auto_discover_workspaces {
        let workspaces = workspace::list(true, core, client.clone()).await?;
        let queue_results =
            work_queue(workspaces, options, attributes, client.clone(), core)
                .await?;
        info!("{:#?}", &queue_results);
        if core.save_output {
            let queue_run_result = QueueRunResult::from(queue_results);
            queue_run_result.save(&core.output, config.pretty_output)?;
        }
    }
    Ok(())
}
