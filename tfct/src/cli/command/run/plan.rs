use crate::{
    cli::{
        command::common::{check_workspace_identifier, parse_workspace_file},
        run::{override_queue_options, set_default_args, DefaultArgs},
    },
    error::ArgError,
    settings::{self, Settings},
};

use log::info;
use std::collections::BTreeMap;
use surf::Client;
use tfc_toolset::{
    run::{work_queue, Attributes, QueueOptions},
    settings::Core,
    workspace,
};
use tfc_toolset_extras::parse_workspace_name;

pub async fn plan(
    args: &DefaultArgs,
    config: &Settings,
    core: &Core,
    client: Client,
) -> miette::Result<(), ArgError> {
    check_workspace_identifier(&args.workspace)?;
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
    let mut options = QueueOptions {
        max_concurrent,
        max_iterations,
        status_check_sleep_seconds,
    };
    override_queue_options(&mut options, args);
    let mut attributes = Attributes {
        plan_only: Some(true),
        terraform_version: Some(core.terraform_version.clone()),
        ..Default::default()
    };
    if let Some(save_plan) = args.save_plan {
        if save_plan {
            attributes.plan_only = None;
            attributes.terraform_version = None;
        }
    }
    set_default_args(&mut attributes, args);

    if let Some(workspace_name) = &args.workspace.workspace_name {
        parse_workspace_name(workspace_name)?;
        let workspace =
            workspace::show_by_name(workspace_name, core, client.clone())
                .await?;
        info!("Creating plan run for workspace {}.", workspace_name);
        if args.queue {
            let mut queue = BTreeMap::new();
            queue.insert(workspace.id.clone(), workspace.clone());
            let queue_results = work_queue(
                &mut queue,
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
    } else if let Some(workspace_id) = &args.workspace.workspace_id {
        let workspace =
            workspace::show(workspace_id, core, client.clone()).await?;
        info!("Creating plan run for workspace {}.", workspace_id);
        if args.queue {
            let mut queue = BTreeMap::new();
            queue.insert(workspace_id.clone(), workspace.clone());
            let queue_results = work_queue(
                &mut queue,
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
    } else if let Some(file_path) = &args.workspace.workspace_file {
        let workspaces =
            parse_workspace_file(file_path, core, client.clone()).await?;
        let mut queue = BTreeMap::new();
        for ws in workspaces.iter() {
            queue.insert(ws.id.clone(), ws.clone());
        }
        let queue_results =
            work_queue(&mut queue, options, attributes, client.clone(), core)
                .await?;
        info!("{:#?}", &queue_results);
    } else if args.workspace.auto_discover_workspaces {
        let workspaces = workspace::list(true, core, client.clone()).await?;
        let mut queue = BTreeMap::new();
        for ws in workspaces.iter() {
            queue.insert(ws.id.clone(), ws.clone());
        }
        let queue_results =
            work_queue(&mut queue, options, attributes, client.clone(), core)
                .await?;
        info!("{:#?}", &queue_results);
    }
    Ok(())
}
