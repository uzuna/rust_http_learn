use axum::{
    extract::{self, Path},
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

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

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState::default());
    let app = Router::new()
        .route("/hello/:name", get(greet))
        .route("/count", get(count))
        .route("/record/create", post(create_record))
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{extract::Path, Extension, Json};

    use crate::{count, create_record, greet, AppState, CreateRecord};

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
}
