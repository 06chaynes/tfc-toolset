mod about;

pub(crate) mod apply;
pub(crate) mod remove;

pub use apply::apply;
pub use remove::remove;

use crate::cli::command::common::WorkspaceArgs;

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct Commands {
    #[command(subcommand)]
    pub command: VariableSetCmds,
}

#[derive(Subcommand, Debug)]
pub(crate) enum VariableSetCmds {
    #[clap(about = about::APPLY)]
    Apply(ManageWorkspaceArgs),
    #[clap(about = about::REMOVE)]
    Remove(ManageWorkspaceArgs),
}

#[derive(Args, Debug)]
pub struct ManageWorkspaceArgs {
    #[arg(short, long, help = about::VARIABLE_SET_ID)]
    pub var_set_id: Vec<String>,
    #[clap(flatten)]
    default: WorkspaceArgs,
}
