use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PathParams1 {
  pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathParams2 {
  pub id: u32,
  pub tail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
  pub ts: Option<i64>,
  pub code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
  pub r#type: Option<String>,
  pub from: Option<String>,
  pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<P, T> {
  pub path: Option<P>,
  pub query: QueryParams,
  pub body: Option<T>,
}
