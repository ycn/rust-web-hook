use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PathParams {
  pub bucket: String,
  pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
  pub cat: Option<String>,
  pub from: Option<String>,
}