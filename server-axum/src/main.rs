use axum::{
    extract::{self, Path},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

mod middleware;

#[derive(Debug, Default)]
struct AppState {
    count: Mutex<u64>,
}

async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

async fn count(Extension(state): Extension<Arc<AppState>>) -> String {
    let mut count = state.count.lock().unwrap();
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

async fn create_record(extract::Json(payload): extract::Json<CreateRecord>) -> Json<RecordCreated> {
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

async fn try_request(
    extract::Json(payload): extract::Json<TryBody>,
) -> Result<Json<TryBody>, (StatusCode, String)> {
    if payload.success {
        Ok(Json(payload))
    } else {
        Err((StatusCode::BAD_REQUEST, "request failed".to_string()))
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct QueryBody {
    require: String,
    length: u32,
    optional: Option<String>,
}

async fn query(
    Path(name): Path<String>,
    extract::Query(info): extract::Query<QueryBody>,
) -> String {
    format!("Query name {name}, body {:?}", info)
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState::default());
    let app = Router::new()
        .route("/hello/:name", get(greet))
        .route("/count", get(count))
        .route("/query/:name", get(query))
        .route("/record/create", post(create_record))
        .route("/try", post(try_request))
        .layer(Extension(shared_state))
        .layer(crate::middleware::SayHi {});

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        extract::{self, Path},
        http::{HeaderValue, Request, StatusCode},
        routing::get,
        Extension, Json, Router,
    };
    use tower::ServiceExt;

    use crate::{
        count, create_record, greet, query, try_request, AppState, CreateRecord, QueryBody, TryBody,
    };

    #[tokio::test]
    async fn test_hello() {
        // こちらは関数の単体テストが出来る
        // actixとは粒度が違う
        let result = greet(Path("test".to_string())).await;
        assert_eq!(result.as_str(), "Hello, test!");
    }

    #[tokio::test]
    async fn test_count() {
        let shared_state = Arc::new(AppState::default());
        for i in 1..3 {
            let result = count(Extension(shared_state.clone())).await;
            assert_eq!(result, format!("Hello, count: {i}"));
        }
    }

    #[tokio::test]
    async fn test_create_record() {
        let name = "test_record";
        let req = CreateRecord {
            name: name.to_string(),
        };
        let result = create_record(Json(req)).await;
        let Json(result) = result;
        assert_eq!(result.name.as_str(), name);
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_try_request() {
        let tt = vec![TryBody { success: true }, TryBody { success: false }];

        for t in tt {
            let result = try_request(Json(t)).await;
            match result {
                Ok(x) => {
                    assert!(x.success);
                }
                Err((status, body)) => {
                    assert_eq!(status, StatusCode::BAD_REQUEST);
                    assert_eq!(body.as_str(), "request failed");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_query() {
        let path = Path("queryname".to_string());
        let query_body = extract::Query(QueryBody {
            require: "query_body".to_string(),
            length: 1234,
            optional: None,
        });
        let result = query(path, query_body).await;
        assert_eq!(result.as_str(), "Query name queryname, body QueryBody { require: \"query_body\", length: 1234, optional: None }");
    }

    #[tokio::test]
    async fn test_middleware() {
        // Middleware単体ではserviceが埋められないのでテストが出来なかった。
        // towerでは[実際にやっている例](https://github.com/tower-rs/tower-http/blob/master/tower-http/src/set_header/response.rs)
        // があるがここではtrait boundを満たせなかった
        // axumのコード例を見るとAppを組み立ててAppをServiceExtをからoneshotを読んでいるので踏襲
        let app = Router::new()
            .route("/hello/:name", get(greet))
            .layer(crate::middleware::SayHi {});
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/hello/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().contains_key("middleware"));
        assert_eq!(
            res.headers().get("middleware"),
            Some(&HeaderValue::from_static("after"))
        );
        let body = hyper::body::to_bytes(res).await.unwrap();
        assert_eq!(&body[..], b"Hello, test!");
    }
}
