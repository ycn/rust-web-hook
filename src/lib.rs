use actix_web::dev::Payload;
use actix_web::http::header::USER_AGENT;
use actix_web::{error, web, Error, FromRequest, HttpRequest, Result};
use base64;
use futures_util::future::{err, ok, Ready};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// constants
// pub static SECRET: &'static str = "12345";
// pub static UA: &'static str = "foobar";

// structs
#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
  pub dir: String,
  pub ua: String,
  pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenParams {
  pub ts: Option<String>,
  pub code: Option<String>,
}

// utils
pub fn get_user_agent<'a>(req: &'a HttpRequest) -> Option<&'a str> {
  req.headers().get(USER_AGENT)?.to_str().ok()
}

pub fn is_authorized(req: &HttpRequest) -> bool {
  let ua = get_user_agent(&req);
  let query = web::Query::<TokenParams>::from_query(req.query_string()).unwrap();
  let app_data = req.app_data::<web::Data<AppData>>().unwrap();

  if query.ts.is_none() || query.code.is_none() {
    return false;
  }

  let token = get_token(Some(query.ts.as_ref().unwrap().as_str()), app_data);

  if ua.is_none() || app_data.ua.ne(ua.unwrap()) {
    return false;
  }

  if token.is_some() && token.unwrap().eq(query.code.as_ref().unwrap().as_str()) {
    return true;
  }

  false
}

pub fn get_token<'a>(ts: Option<&'a str>, app_data: &'a web::Data<AppData>) -> Option<String> {
  if ts.is_some() {
    let mut hasher = Sha256::new();
    hasher.update(format!(
      "{}!!{}##{}",
      app_data.secret,
      ts.unwrap(),
      app_data.secret
    ));
    let hash = hasher.finalize();
    Some(base64::encode_config(&hash, base64::URL_SAFE_NO_PAD))
  } else {
    None
  }
}

// [Middleware::Extractor] AuthorizedUrl
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizedUrl;

impl FromRequest for AuthorizedUrl {
  type Error = Error;
  type Future = Ready<Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
    if is_authorized(req) {
      ok(AuthorizedUrl)
    } else {
      err(error::ErrorBadRequest("not authorized"))
    }
  }
}
