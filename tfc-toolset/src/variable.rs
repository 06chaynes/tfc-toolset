use crate::{error::ToolError, settings::Core, BASE_URL};
use serde::{Deserialize, Serialize};
use surf::{http::Method, Client, RequestBuilder};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Attributes {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Variable {
    pub id: String,
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Deserialize)]
struct VariablesResponseOuter {
    pub data: Vec<Variable>,
}

pub async fn get_variables(
    workspace_id: &str,
    config: &Core,
    client: Client,
) -> Result<Vec<Variable>, ToolError> {
    let url =
        Url::parse(&format!("{}/workspaces/{}/vars/", BASE_URL, workspace_id))?;
    let req = RequestBuilder::new(Method::Get, url)
        .header("Authorization", &format!("Bearer {}", config.token))
        .build();
    match client.recv_string(req).await {
        Ok(s) => Ok(serde_json::from_str::<VariablesResponseOuter>(&s)?.data),
        Err(e) => Err(ToolError::General(e.into_inner())),
    }
}
