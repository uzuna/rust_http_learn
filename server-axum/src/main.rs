use axum::{
    extract::Path,
    routing::get,
    Router,
};

use std::net::SocketAddr;

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/hello/:name", get(hello));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
