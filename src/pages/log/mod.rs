mod types;

use actix_web::{error, post, web, Error, Result};
use backtrace::Backtrace;
use chrono::prelude::*;
use regex::Regex;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use types::{PathParams, QueryParams};
use web_hook::{AppData, AuthorizedUrl};

#[post("/log/{bucket}/{device_id}")]
pub async fn action(
  app_data: web::Data<AppData>,
  path: web::Path<PathParams>,
  query: web::Query<QueryParams>,
  bytes: web::Bytes,
  authed: Result<AuthorizedUrl>,
) -> Result<String, Error> {
  let bt = Backtrace::new();
  match authed {
    Ok(_) => match String::from_utf8(bytes.to_vec()) {
      Ok(text) => {
        if text.is_empty() {
          return Err(error::ErrorBadRequest("missing body"));
        }
        if let Err(e) = write_log_line(
          app_data.dir.as_str(),
          path.bucket.as_str(),
          path.device_id.as_str(),
          query
            .cat
            .clone()
            .unwrap_or(String::from("unknown"))
            .as_str(),
          query
            .from
            .clone()
            .unwrap_or(String::from("unknown"))
            .as_str(),
          text.as_str(),
        ) {
          return Err(error::ErrorInternalServerError(e));
        }
        Ok(String::from("ok"))
      }
      Err(e) => Err(error::ErrorInternalServerError(e)),
    },
    Err(e) => {
      println!("{:?}", bt);
      Err(e)
    }
  }
}

fn write_log_line(
  dir: &str,
  bucket: &str,
  device_id: &str,
  cat: &str,
  from: &str,
  body: &str,
) -> Result<(), io::Error> {
  let utc: DateTime<Utc> = Utc::now();
  let date_str: String = utc.format("%Y%m%d").to_string();
  let time_str: String = utc.format("%+").to_string();
  let log_path = format!("{}/{}/{}", dir, bucket, device_id);
  let log_file = format!("{}/{}.log", log_path, date_str);

  // mkdir
  fs::create_dir_all(log_path)?;

  // write log
  let mut file = OpenOptions::new()
    .create(true)
    .write(true)
    .append(true)
    .open(log_file)
    .unwrap();

  // escape data
  let re = Regex::new(r"[\r\n]").unwrap();
  let data = re.replace_all(body, "||");

  // log line fmt
  writeln!(
    file,
    "[{}]\t{}|{}\t####\t{}\t{}\t{}",
    time_str, device_id, bucket, cat, from, data
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use actix_web::{
    body::to_bytes,
    dev::Service,
    http::{self, header::USER_AGENT},
    test, App,
  };
  use web_hook::AppData;

  #[actix_web::test]
  async fn test_page_log_action_error() {
    // Start `action` service
    let app = test::init_service(
      App::new()
        .app_data(web::Data::new(AppData {
          dir: String::from("./logs/web_hook_test"),
          secret: String::from("12345"),
          ua: String::from("foobar"),
        }))
        .service(action),
    )
    .await;

    {
      // 404 - wrong method
      let req = test::TestRequest::get()
        .uri("/log/sms/100?ts=123&code=456")
        .to_request();

      let resp = app.call(req).await.unwrap();
      assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    {
      // 404 - mismatch path
      let req = test::TestRequest::post().uri("/log/sss").to_request();

      let resp = app.call(req).await.unwrap();
      assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    {
      // 400 - missing code
      let req = test::TestRequest::post()
        .uri("/log/sms/100?ts=123")
        .insert_header((USER_AGENT, "foobar"))
        .to_request();

      let resp = app.call(req).await.unwrap();
      assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    {
      // 400 - bad code
      let req = test::TestRequest::post()
        .uri("/log/sms/100?ts=123&code=123")
        .insert_header((USER_AGENT, "foobar"))
        .to_request();

      let resp = app.call(req).await.unwrap();
      assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    {
      // 400 - bad UA
      let req = test::TestRequest::post()
        .uri("/log/sms/100?ts=123&code=sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns")
        .insert_header((USER_AGENT, "ABC"))
        .to_request();

      let resp = app.call(req).await.unwrap();
      assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    {
      // 400 - no request body
      let req = test::TestRequest::post()
        .uri("/log/sms/100?ts=123&code=sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns")
        .insert_header((USER_AGENT, "foobar"))
        .to_request();

      let resp = app.call(req).await.unwrap();
      assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }
  }

  #[actix_web::test]
  async fn test_page_log_action_ok() {
    // Start `action` service
    let app = test::init_service(
      App::new()
        .app_data(web::Data::new(AppData {
          dir: String::from("./logs/web_hook_test"),
          secret: String::from("12345"),
          ua: String::from("foobar"),
        }))
        .service(action),
    )
    .await;

    {
      // 200 - with correct UA
      let req = test::TestRequest::post()
        .uri("/log/sms/100?ts=123&code=sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns&cat=text&from=13500009999")
        .insert_header((USER_AGENT, "foobar"))
        .set_payload(String::from("中文\n你好"))
        .to_request();

      let resp = app.call(req).await.unwrap();
      assert_eq!(resp.status(), http::StatusCode::OK);

      let body_bytes = to_bytes(resp.into_body()).await.unwrap();
      assert_eq!(body_bytes, r##"ok"##);
    }
  }
}
