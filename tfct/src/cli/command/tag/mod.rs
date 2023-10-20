mod about;

pub(crate) mod add;
pub(crate) mod list;
pub(crate) mod remove;

pub use add::add;
pub use list::list;
pub use remove::remove;

use super::common::WorkspaceArgs;

use crate::error::ArgError;
use clap::{Args, Subcommand};
use tfc_toolset::tag::Tag;
use tfc_toolset_extras::file::input::tag::TagsFile;

#[derive(Args, Debug)]
pub(crate) struct Commands {
    #[command(subcommand)]
    pub command: TagCmds,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TagCmds {
    #[clap(about = about::ADD)]
    Add(ManageArgs),
    #[clap(about = about::LIST)]
    List(WorkspaceArgs),
    #[clap(about = about::REMOVE)]
    Remove(ManageArgs),
}

#[derive(Args, Debug)]
pub struct ManageArgs {
    #[arg(short, long, help = about::NAME, required = true)]
    pub name: Vec<String>,
    #[arg(long, help = about::TAG_FILE)]
    pub tag_file: Option<String>,
    #[clap(flatten)]
    default: WorkspaceArgs,
}

pub(crate) fn check_tag_identifier(args: &ManageArgs) -> Result<(), ArgError> {
    if args.name.is_empty() && args.tag_file.is_none() {
        Err(ArgError::MissingTagIdentifier)
    } else {
        Ok(())
    }
}

pub(crate) async fn parse_tag_file(path: &str) -> Result<Vec<Tag>, ArgError> {
    let tag_file = TagsFile::load(path)?;
    let mut tags = Vec::new();
    if let Some(tag_entries) = tag_file.tags {
        for tag_entry in tag_entries {
            if let Some(tag) = tag_entry.attributes {
                tags.push(Tag::new(tag.name));
            }
        }
    }
    Ok(tags)
}
