pub mod error;
pub mod filter;
pub mod run;
pub mod settings;

pub mod tag;
pub mod variable;
pub mod variable_set;
pub mod workspace;

use crate::settings::Core;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surf::http::Method;
use surf::{Request, RequestBuilder};
use url::Url;

pub const BASE_URL: &str = "https://app.terraform.io/api/v2";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pagination {
    #[serde(rename = "current-page")]
    pub current_page: u32,
    #[serde(rename = "page-size")]
    pub page_size: u32,
    #[serde(rename = "prev-page")]
    pub prev_page: Option<u32>,
    #[serde(rename = "next-page")]
    pub next_page: Option<u32>,
    #[serde(rename = "total-pages")]
    pub total_pages: u32,
    #[serde(rename = "total-count")]
    pub total_count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    pub pagination: Pagination,
}

pub(crate) fn set_page_number(page_number: u32, u: Url) -> Option<Url> {
    let u = match Url::parse_with_params(
        u.clone().as_str(),
        &[("page[number]", &page_number.to_string())],
    ) {
        Ok(u) => u,
        Err(e) => {
            error!("{:#?}", e);
            return None;
        }
    };
    Some(u)
}

pub(crate) fn build_request(
    method: Method,
    url: Url,
    config: &Core,
    body: Option<Value>,
) -> Request {
    let mut req = RequestBuilder::new(method, url.clone())
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Content-Type", "application/vnd.api+json")
        .build();
    if let Some(body) = body {
        req.set_body(body);
    }
    req
}
