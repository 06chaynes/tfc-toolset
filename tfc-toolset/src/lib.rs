pub mod error;
pub mod filter;
pub mod run;
pub mod settings;
pub mod variable;
pub mod workspace;

use serde::{Deserialize, Serialize};

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
