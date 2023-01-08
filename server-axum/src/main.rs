use axum::{extract::Path, routing::get, Extension, Router};

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
struct AppState {
    count: u64,
}

async fn hello(Path(name): Path<String>) -> String {
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
        .route("/hello/:name", get(hello))
        .route("/count", get(count))
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
