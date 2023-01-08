use axum::{extract::Path, routing::get, Extension, Router};

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
struct AppState {
    count: u64,
}

async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

async fn count(Extension(state): Extension<Arc<Mutex<AppState>>>) -> String {
    let mut ws = state.lock().unwrap();
    ws.count += 1;
    format!("Hello, count: {}", ws.count)
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(AppState::default()));
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
    use axum::extract::Path;

    use crate::greet;

    #[tokio::test]
    async fn test_hello() {
        // こちらは関数の単体テストが出来る
        // actixとは粒度が違う
        let result = greet(Path("test".to_string())).await;
        assert_eq!(result.as_str(), "Hello, test!");
    }
}
