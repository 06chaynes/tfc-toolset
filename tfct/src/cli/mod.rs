mod command;

use crate::{error::ArgError, settings::Settings};
use clap::{Args, Parser, Subcommand};
pub(super) use command::{run, tag, variable, variable_set, workspace};
use log::warn;
use miette::IntoDiagnostic;
use std::{path::PathBuf, str::FromStr};
use tfc_toolset::settings::{Core, Query, Tag, Variable};

const CLI: &str =
    "A tool to help manage a toolset that helps manage your deployments";
const WORKSPACE: &str = "Manage workspaces";

const VARIABLE: &str = "Manage workspace variables";
const VARIABLE_SET: &str = "Manage variable sets";
const TAG: &str = "Manage workspace tags";
const RUN: &str = "Manage runs";
const ORG: &str = "The name of the organization";
const TOKEN: &str = "The token to use for authentication";
const PROJECT: &str = "The id of the project";
const LOG: &str = "The log level to use";
const OUTPUT: &str = "The location where report output should be written";
const START_PAGE: &str = "The page to start at when retrieving data";
const MAX_DEPTH: &str = "The maximum number of pages to retrieve";
const PAGE_SIZE: &str = "The number of items to retrieve per page";
const SAVE_OUTPUT: &str = "Save the output of the command to a file";
const PRETTY_OUTPUT: &str = "Pretty print the output when saving to a file";
const QUERY_NAME: &str = "The name of the workspace to fuzzy search for";
const QUERY_WILDCARD_NAME: &str =
    "The name of the workspace to wildcard search for";
const QUERY_VARIABLE: &str =
    "The name of the variable to search for, formatted as key:operator:value";
const QUERY_TAG: &str =
    "The name of the tag to search for, formatted as operator:name";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = Some(CLI))]
#[clap(propagate_version = true)]
pub(super) struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(flatten)]
    pub root: RootArgs,
}

#[derive(Subcommand, Debug)]
pub(super) enum Commands {
    #[clap(about = WORKSPACE)]
    Workspace(Box<workspace::Commands>),
    #[clap(about = TAG)]
    Tag(Box<tag::Commands>),
    #[clap(about = VARIABLE)]
    Variable(Box<variable::Commands>),
    #[clap(about = VARIABLE_SET)]
    VariableSet(Box<variable_set::Commands>),
    #[clap(about = RUN)]
    Run(Box<run::Commands>),
}

#[derive(Args, Debug)]
pub struct RootArgs {
    #[arg(long, help = ORG, global = true)]
    pub org: Option<String>,
    #[arg(long, help = TOKEN, global = true)]
    pub token: Option<String>,
    #[arg(long, help = PROJECT, global = true)]
    pub project: Option<String>,
    #[arg(long, help = LOG, global = true)]
    pub log: Option<String>,
    #[arg(long, help = OUTPUT, global = true)]
    pub output: Option<PathBuf>,
    #[arg(long, help = START_PAGE, global = true)]
    pub start_page: Option<String>,
    #[arg(long, help = MAX_DEPTH, global = true)]
    pub max_pages: Option<String>,
    #[arg(long, help = PAGE_SIZE, global = true)]
    pub page_size: Option<String>,
    #[arg(action, long, help = SAVE_OUTPUT, global = true, default_value = "false")]
    pub save_output: bool,
    #[arg(action, long, help = PRETTY_OUTPUT, global = true, default_value = "false")]
    pub pretty_output: bool,
    #[arg(long, help = QUERY_NAME, global = true)]
    pub query_name: Option<String>,
    #[arg(long, help = QUERY_WILDCARD_NAME, global = true)]
    pub query_wildcard_name: Option<String>,
    #[arg(long, help = QUERY_VARIABLE, global = true)]
    pub query_variable: Option<Vec<String>>,
    #[arg(long, help = QUERY_TAG, global = true)]
    pub query_tag: Option<Vec<String>>,
}

pub(crate) fn override_core(
    config: &mut Core,
    args: &RootArgs,
) -> miette::Result<()> {
    if let Some(org) = &args.org {
        config.org = org.clone();
    }
    if let Some(token) = &args.token {
        config.token = token.clone();
    }
    if let Some(project) = &args.project {
        config.project = Some(project.clone());
    }
    if let Some(log) = &args.log {
        config.log = log.clone();
    }
    if let Some(output) = &args.output {
        config.output = output.clone();
    }
    if let Some(start_page) = &args.start_page {
        config.pagination.start_page = start_page.clone();
    }
    if let Some(max_pages) = &args.max_pages {
        config.pagination.max_depth = max_pages.clone();
    }
    if let Some(page_size) = &args.page_size {
        config.pagination.page_size = page_size.clone();
    }
    if let Some(query_name) = &args.query_name {
        match &mut config.workspaces.query {
            Some(query) => query.name = Some(query_name.clone()),
            None => {
                config.workspaces.query = Some(Query {
                    name: Some(query_name.clone()),
                    wildcard_name: None,
                    variables: None,
                    tags: None,
                })
            }
        }
    }
    if let Some(query_wildcard_name) = &args.query_wildcard_name {
        match &mut config.workspaces.query {
            Some(query) => {
                query.wildcard_name = Some(query_wildcard_name.clone())
            }
            None => {
                config.workspaces.query = Some(Query {
                    name: None,
                    wildcard_name: Some(query_wildcard_name.clone()),
                    variables: None,
                    tags: None,
                })
            }
        }
    }
    if let Some(query_variable) = &args.query_variable {
        match &mut config.workspaces.query {
            Some(query) => {
                let mut variables = vec![];
                for variable in query_variable {
                    let entry =
                        Variable::from_str(variable).into_diagnostic()?;
                    variables.push(entry);
                }
                query.variables = Some(variables);
            }
            None => {
                let mut variables = vec![];
                for variable in query_variable {
                    let entry =
                        Variable::from_str(variable).into_diagnostic()?;
                    variables.push(entry);
                }
                config.workspaces.query = Some(Query {
                    name: None,
                    wildcard_name: None,
                    variables: Some(variables),
                    tags: None,
                })
            }
        }
    }
    if let Some(query_tag) = &args.query_tag {
        match &mut config.workspaces.query {
            Some(query) => {
                let mut tags = vec![];
                for tag in query_tag {
                    let entry = Tag::from_str(tag).into_diagnostic()?;
                    tags.push(entry);
                }
                query.tags = Some(tags);
            }
            None => {
                let mut tags = vec![];
                for tag in query_tag {
                    let entry = Tag::from_str(tag).into_diagnostic()?;
                    tags.push(entry);
                }
                config.workspaces.query = Some(Query {
                    name: None,
                    wildcard_name: None,
                    variables: None,
                    tags: Some(tags),
                })
            }
        }
    }
    config.save_output = args.save_output;
    Ok(())
}

pub(crate) fn override_config(config: &mut Settings, args: &RootArgs) {
    config.pretty_output = args.pretty_output;
}

pub(crate) fn validate_core(core: &Core) -> miette::Result<(), ArgError> {
    if core.token.is_empty() {
        return Err(ArgError::MissingToken);
    }
    if core.org.is_empty() {
        warn!(
            "No organization provided, this will likely result in 404 errors"
        );
    }
    Ok(())
}
