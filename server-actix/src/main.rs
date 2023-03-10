use std::{path::PathBuf, sync::Mutex};

use actix_files::NamedFile;
use actix_web::{
    get, post,
    web::{self, Json},
    App, HttpRequest, HttpServer, Responder,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

mod middleware;

#[derive(Debug, Default)]
struct AppState {
    count: Mutex<u32>,
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[get("/count")]
async fn count(data: web::Data<AppState>) -> impl Responder {
    let mut count = data.count.lock().unwrap();
    *count += 1;
    format!("Hello, count: {count}")
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
struct CreateRecord {
    name: String,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
struct RecordCreated {
    id: u64,
    name: String,
    ts: DateTime<Utc>,
}

#[post("/record/create")]
async fn create_record(Json(payload): Json<CreateRecord>) -> Json<RecordCreated> {
    let CreateRecord { name, .. } = payload;
    let record = RecordCreated {
        id: 1234,
        name,
        ts: Utc::now(),
    };
    Json(record)
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
struct TryBody {
    success: bool,
}

#[post("/try")]
async fn try_request(Json(payload): Json<TryBody>) -> actix_web::Result<Json<TryBody>> {
    if payload.success {
        Ok(Json(payload))
    } else {
        Err(actix_web::error::ErrorBadRequest("request failed"))
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct QueryBody {
    require: String,
    length: u32,
    optional: Option<String>,
}

#[get("/query/{name}")]
async fn query(name: web::Path<String>, web::Query(info): web::Query<QueryBody>) -> impl Responder {
    format!("Query name {name}, body {:?}", info)
}

#[get("/files/{filename:.*}")]
async fn index(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState::default());
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(crate::middleware::SayHi)
            .service(greet)
            .service(count)
            .service(create_record)
            .service(try_request)
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use std::future;

    use crate::{
        count, create_record, greet, index, query, try_request, AppState, CreateRecord,
        RecordCreated, TryBody,
    };
    use actix_web::{
        body::MessageBody,
        http::header::{self, HeaderValue},
        rt::pin,
        test::{self, read_body, read_body_json},
        web, App,
    };

    #[actix_web::test]
    async fn test_hello() {
        // service????????????????????????init_service, call_service?????????
        let app = test::init_service(App::new().service(greet)).await;
        let req = test::TestRequest::get().uri("/hello/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = resp.into_body();
        pin!(body);

        // first chunk
        let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"Hello test!")
        );
    }

    #[actix_web::test]
    async fn test_count() {
        let state = web::Data::new(AppState::default());
        let app = test::init_service(App::new().app_data(state.clone()).service(count)).await;

        for i in 1..3 {
            let req = test::TestRequest::get().uri("/count").to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let body = resp.into_body();
            pin!(body);

            // first chunk
            let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
            assert_eq!(
                bytes.unwrap().unwrap(),
                web::Bytes::from(format!("Hello, count: {i}"))
            );
        }
    }

    #[actix_web::test]
    async fn test_create_record() {
        let name = "test_record";
        let req = CreateRecord {
            name: name.to_string(),
        };
        let payload = serde_json::to_vec(&req).unwrap();
        let app = test::init_service(App::new().service(create_record)).await;
        let req = test::TestRequest::post()
            .uri("/record/create")
            .append_header(header::ContentType(mime::APPLICATION_JSON))
            .set_payload(payload)
            .to_request();
        let resp: RecordCreated = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.name.as_str(), name);
        println!("{:?}", resp);
    }

    #[actix_web::test]
    async fn test_try_request() {
        let tt = vec![TryBody { success: true }, TryBody { success: false }];

        for t in tt {
            let payload = serde_json::to_vec(&t).unwrap();
            let app = test::init_service(App::new().service(try_request)).await;
            let req = test::TestRequest::post()
                .uri("/try")
                .append_header(header::ContentType(mime::APPLICATION_JSON))
                .set_payload(payload)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status().is_success(), t.success);
            if resp.status().is_success() {
                let resp: TryBody = read_body_json(resp).await;
                assert!(resp.success);
            } else {
                let body = read_body(resp).await;
                assert_eq!(body, web::Bytes::from_static(b"request failed"));
            }
        }
    }

    #[actix_web::test]
    async fn test_query() {
        let app = test::init_service(App::new().service(query)).await;
        let req = test::TestRequest::get()
            .uri("/query/queryname?length=1234&require=query_string")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body = read_body(resp).await;
        assert_eq!(body, web::Bytes::from_static(b"Query name queryname, body QueryBody { require: \"query_string\", length: 1234, optional: None }"));
    }

    #[actix_web::test]
    async fn test_middleware() {
        let app =
            test::init_service(App::new().wrap(crate::middleware::SayHi).service(greet)).await;
        let req = test::TestRequest::get().uri("/hello/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.headers().contains_key("middleware"));
        assert_eq!(
            resp.headers().get("middleware"),
            Some(&HeaderValue::from_str("after").unwrap())
        );
    }

    #[actix_web::test]
    async fn test_staticfile() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get()
            .uri("/files/Cargo.toml")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body = read_body(resp).await;
        assert!(body.starts_with(b"[package]"));
    }
}
