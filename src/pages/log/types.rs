use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PathParams {
  pub bucket: String,
  pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
  pub cat: String,
  pub from: String,
  pub data: String,
}