use crate::{error::ToolError, settings::Core, BASE_URL};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use surf::{http::Method, Client, RequestBuilder};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vars {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub attributes: Attributes,
}

// the vars are in the format of key=value:description:category:hcl:sensitive
// we need to parse each one into a variable::Vars
// description, category, hcl, sensitive are all optional and will be None if not provided
// to skip a field just use a colon e.g. key=value::::true would only set key, value, and sensitive
// accepting the default for the rest
impl FromStr for Vars {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(':') {
            let var_split: Vec<&str> = s.split(':').collect();
            let key_val = var_split[0].to_string();
            let key_val_split: Vec<&str> = key_val.split('=').collect();
            let key = key_val_split[0].to_string();
            let value = key_val_split[1].to_string();
            let description = if var_split[1].is_empty() {
                None
            } else {
                Some(var_split[1].to_string())
            };
            let category = if var_split[2].is_empty() {
                Category::default()
            } else {
                Category::from(var_split[2].to_string())
            };
            let hcl = if var_split[3].is_empty() {
                None
            } else {
                Some(var_split[3].parse::<bool>()?)
            };
            let sensitive = if var_split[4].is_empty() {
                None
            } else {
                Some(var_split[4].parse::<bool>()?)
            };
            Ok(Vars {
                relationship_type: "vars".to_string(),
                attributes: Attributes {
                    key,
                    value: Some(value),
                    description,
                    category,
                    hcl,
                    sensitive,
                },
            })
        } else {
            let key_val_split = s.split('=').collect::<Vec<&str>>();
            let key = key_val_split[0].to_string();
            let value = key_val_split[1].to_string();
            Ok(Vars {
                relationship_type: "vars".to_string(),
                attributes: Attributes {
                    key,
                    value: Some(value),
                    description: None,
                    category: Category::default(),
                    hcl: None,
                    sensitive: None,
                },
            })
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    #[default]
    Terraform,
    Env,
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Terraform => write!(f, "terraform"),
            Category::Env => write!(f, "env"),
        }
    }
}

impl From<String> for Category {
    fn from(s: String) -> Self {
        match s.as_str() {
            "terraform" => Category::Terraform,
            "env" => Category::Env,
            _ => Category::Terraform,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Attributes {
    pub key: String,
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hcl: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Variable {
    pub id: String,
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Deserialize)]
struct VariablesResponseOuter {
    pub data: Vec<Variable>,
}

pub async fn list(
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> Result<Vec<Variable>, ToolError> {
    let url =
        Url::parse(&format!("{}/workspaces/{}/vars/", BASE_URL, workspace_id))?;
    let req = RequestBuilder::new(Method::Get, url)
        .header("Authorization", format!("Bearer {}", config.token))
        .build();
    match client.recv_string(req).await {
        Ok(s) => Ok(serde_json::from_str::<VariablesResponseOuter>(&s)?.data),
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}

pub async fn delete(
    variable_id: &str,
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> Result<(), ToolError> {
    let url = Url::parse(&format!(
        "{}/workspaces/{}/vars/{}",
        BASE_URL, workspace_id, variable_id
    ))?;
    let req = RequestBuilder::new(Method::Delete, url)
        .header("Authorization", format!("Bearer {}", config.token))
        .build();
    client
        .recv_string(req)
        .await
        .map_err(|e| ToolError::General(e.into_inner()))?;
    Ok(())
}
