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

#[cfg(test)]
mod tests {
    use std::future;

    use crate::{count, greet, AppState};
    use actix_web::{body::MessageBody, rt::pin, test, web, App};

    #[actix_web::test]
    async fn test_hello() {
        // serviceをテストするのでinit_service, call_serviceを使う
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
}
