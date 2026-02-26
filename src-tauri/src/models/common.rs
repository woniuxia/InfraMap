use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub search: Option<String>,
    pub filters: Option<HashMap<String, String>>,
}

impl QueryParams {
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1)
    }
    pub fn page_size(&self) -> u32 {
        self.page_size.unwrap_or(20)
    }
}

#[derive(Debug, Serialize)]
pub struct PagedResult<T: Serialize> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}
