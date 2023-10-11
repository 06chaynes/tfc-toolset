use crate::{error::ToolError, settings::Core, BASE_URL};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use surf::{http::Method, Client, RequestBuilder};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vars {
    #[serde(rename = "type")]
    pub relationship_type: String,
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
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
    pub description: Option<String>,
    pub category: Option<Category>,
    pub hcl: Option<bool>,
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
