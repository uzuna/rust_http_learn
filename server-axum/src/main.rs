use axum::{extract::Path, routing::get, Extension, Router};

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

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState::default());
    let app = Router::new()
        .route("/hello/:name", get(greet))
        .route("/count", get(count))
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

    use axum::{extract::Path, Extension};

    use crate::{count, greet, AppState};

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
}
