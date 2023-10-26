mod about;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod list;

pub use create::create;
pub use delete::delete;
pub use list::list;
use std::str::FromStr;

use super::common::WorkspaceArgs;

use crate::error::ArgError;
use clap::{Args, Subcommand};
use delete::DeleteArgs;
use tfc_toolset::variable::Variable;
use tfc_toolset_extras::VariablesFile;

#[derive(Args, Debug)]
pub(crate) struct Commands {
    #[command(subcommand)]
    pub command: VariableCmds,
}

#[derive(Subcommand, Debug)]
pub(crate) enum VariableCmds {
    #[clap(about = about::CREATE)]
    Create(ManageArgs),
    #[clap(about = about::DELETE)]
    Delete(DeleteArgs),
    #[clap(about = about::LIST)]
    List(WorkspaceArgs),
}

#[derive(Args, Debug)]
pub struct ManageArgs {
    #[arg(short, long, help = about::VARIABLE)]
    pub var: Vec<String>,
    #[arg(long, help = about::VAR_FILE)]
    pub var_file: Option<String>,
    #[clap(flatten)]
    default: WorkspaceArgs,
}

pub(crate) fn check_variable_identifier_basic(
    args: &ManageArgs,
) -> Result<(), ArgError> {
    if args.var.is_empty() && args.var_file.is_none() {
        Err(ArgError::MissingVariableIdentifierBasic)
    } else {
        Ok(())
    }
}

pub(crate) fn check_variable_identifier(
    args: &DeleteArgs,
) -> Result<(), ArgError> {
    if args.var_key.is_empty() && args.var_id.is_empty() && args.var_file.is_none() {
        Err(ArgError::MissingVariableIdentifier)
    } else {
        Ok(())
    }
}

pub(crate) async fn parse_variable_file(
    path: &str,
) -> Result<Vec<Variable>, ArgError> {
    let variable_file = VariablesFile::load(path)?;
    let mut variables = Vec::new();
    if let Some(variable_entries) = variable_file.variables {
        for variable_entry in variable_entries {
            if let Some(var) = variable_entry.var {
                let variable = Variable::from_str(&var)?;
                variables.push(variable);
            }
        }
    }
    Ok(variables)
}
