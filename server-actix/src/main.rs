use std::sync::Mutex;

use actix_web::{get, web, App, HttpServer, Responder};

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

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState::default());
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(greet)
            .service(count)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
