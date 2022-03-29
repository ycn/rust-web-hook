mod types;

use actix_web::{get, post, web, Error, HttpResponse, Result};
use types::{PathParams1, PathParams2, QueryParams, RequestBody, Response};
use web_hook::AuthorizedUrl;

/**
 * To learn the meta of Rust: https://doc.rust-lang.org/1.6.0/book/ownership.html
 * To learn the extractors of actix-web: https://actix.rs/docs/extractors/
 * Custom Extractor see: lib::AuthorizedUrl
 */

#[get("/hello/{name}.html")]
pub async fn get(
    path: web::Path<PathParams1>,
    query: web::Query<QueryParams>,
    authed: Result<AuthorizedUrl>,
) -> Result<HttpResponse, Error> {
    match authed {
        Ok(_) => {
            let res = Response {
                path: Some(path.into_inner()),
                query: query.into_inner(),
                body: Some("hello"),
            };
            Ok(HttpResponse::Ok().json(res))
        }
        Err(e) => Err(e),
    }
}

#[post("/hello/{id}/set/{tail:.*}")]
pub async fn post(
    path: web::Path<PathParams2>,
    query: web::Query<QueryParams>,
    data: web::Json<RequestBody>,
    authed: Result<AuthorizedUrl>,
) -> Result<HttpResponse, Error> {
    match authed {
        Ok(_) => {
            let res = Response {
                path: Some(path.into_inner()),
                query: query.into_inner(),
                body: Some(data.into_inner()),
            };
            Ok(HttpResponse::Ok().json(res))
        }
        Err(e) => Err(e),
    }
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
    async fn test_page_hello_get_error() {
        // Start `get` service
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    dir: String::from("./logs/web_hook_test"),
                    secret: String::from("12345"),
                    ua: String::from("foobar"),
                }))
                .service(get),
        )
        .await;

        {
            // 404 - wrong method
            let req = test::TestRequest::post()
                .uri("/hello/andy.html?ts=123&code=456")
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
        }

        {
            // 404 - mismatch path
            let req = test::TestRequest::get().uri("/hello/andy").to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
        }

        {
            // 400 - missing code
            let req = test::TestRequest::get()
                .uri("/hello/andy.html?ts=123")
                .insert_header((USER_AGENT, "foobar"))
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        }

        {
            // 400 - bad code
            let req = test::TestRequest::get()
                .uri("/hello/andy.html?ts=123&code=123")
                .insert_header((USER_AGENT, "foobar"))
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        }

        {
            // 400 - bad UA
            let req = test::TestRequest::get()
                .uri("/hello/andy.html?ts=123&code=sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns")
                .insert_header((USER_AGENT, "ABC"))
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        }
    }

    #[actix_web::test]
    async fn test_page_hello_get_ok() {
        // Start `get` service
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    dir: String::from("./logs/web_hook_test"),
                    secret: String::from("12345"),
                    ua: String::from("foobar"),
                }))
                .service(get),
        )
        .await;

        {
            // 200 - with correct UA
            let req = test::TestRequest::get()
                .uri("/hello/andy.html?ts=123&code=sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns")
                .insert_header((USER_AGENT, "foobar"))
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::OK);

            let body_bytes = to_bytes(resp.into_body()).await.unwrap();
            assert_eq!(
                body_bytes,
                r##"{"path":{"name":"andy"},"query":{"ts":123,"code":"sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns"},"body":"hello"}"##
            );
        }
    }

    #[actix_web::test]
    async fn test_page_hello_post_error() {
        // Start `post` service
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    dir: String::from("./logs/web_hook_test"),
                    secret: String::from("12345"),
                    ua: String::from("foobar"),
                }))
                .service(post),
        )
        .await;

        {
            // 404 - wrong method
            let req = test::TestRequest::get()
                .uri("/hello/100/set/a/b/c?ts=123&code=456")
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
        }

        {
            // 404 - mismatch path
            let req = test::TestRequest::post().uri("/hello/100/set").to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
        }

        {
            // 400 - missing code
            let req = test::TestRequest::post()
                .uri("/hello/100/set/a/b/c?ts=123")
                .insert_header((USER_AGENT, "foobar"))
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        }

        {
            // 400 - bad code
            let req = test::TestRequest::post()
                .uri("/hello/100/set/a/b/c?ts=123&code=123")
                .insert_header((USER_AGENT, "foobar"))
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        }

        {
            // 400 - bad UA
            let req = test::TestRequest::post()
                .uri("/hello/100/set/a/b/c?ts=123&code=sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns")
                .insert_header((USER_AGENT, "ABC"))
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        }

        {
            // 400 - no request body
            let req = test::TestRequest::post()
                .uri("/hello/100/set/a/b/c?ts=123&code=sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns")
                .insert_header((USER_AGENT, "foobar"))
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
        }
    }

    #[actix_web::test]
    async fn test_page_hello_post_ok() {
        // Start `post` service
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    dir: String::from("./logs/web_hook_test"),
                    secret: String::from("12345"),
                    ua: String::from("foobar"),
                }))
                .service(post),
        )
        .await;

        {
            // 200 - with correct UA
            let req = test::TestRequest::post()
                .uri("/hello/100/set/a/b/c?ts=123&code=sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns")
                .insert_header((USER_AGENT, "foobar"))
                .set_json(&RequestBody {
                    r#type: Some(String::from("sms")),
                    from: Some(String::from("12345")),
                    data: Some(String::from("中文")),
                })
                .to_request();

            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.status(), http::StatusCode::OK);

            let body_bytes = to_bytes(resp.into_body()).await.unwrap();
            assert_eq!(
                body_bytes,
                r##"{"path":{"id":100,"tail":"a/b/c"},"query":{"ts":123,"code":"sGUTG_BJFh9DRUcxsnMb0DyOq6iO09uCHonwLyvWGns"},"body":{"type":"sms","from":"12345","data":"中文"}}"##
            );
        }
    }
}
