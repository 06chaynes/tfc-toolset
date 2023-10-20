use crate::ExtrasError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tfc_toolset::{
    error::ToolError,
    tag::Attributes,
    workspace::{Workspace, WorkspaceTags},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Attributes>,
    pub workspace: Option<Workspace>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TagsFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

impl From<Vec<WorkspaceTags>> for TagsFile {
    fn from(vec: Vec<WorkspaceTags>) -> Self {
        let mut tags_file = TagsFile { tags: None };
        let mut tags_vec = Vec::new();
        for workspace_tags in vec {
            for tag_vec in workspace_tags.tags {
                for tag in tag_vec.data {
                    let entry = Tag {
                        id: tag.id,
                        attributes: Some(tag.attributes),
                        workspace: Some(workspace_tags.workspace.clone()),
                    };
                    tags_vec.push(entry);
                }
            }
        }
        tags_file.tags = Some(tags_vec);
        tags_file
    }
}

impl TagsFile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ExtrasError> {
        let file = File::open(path).map_err(ToolError::Io)?;
        let reader = BufReader::new(file);
        let tags_file: Self =
            serde_json::from_reader(reader).map_err(ToolError::Json)?;
        Ok(tags_file)
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
