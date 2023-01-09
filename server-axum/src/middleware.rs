use axum::{
    body::Body,
    http::{HeaderValue, Request},
    response::Response,
};
use futures::future::BoxFuture;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct SayHi;

impl<S> Layer<S> for SayHi {
    type Service = SayHiMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SayHiMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct SayHiMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for SayHiMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        req.headers_mut()
            .insert("middleware", HeaderValue::from_static("before"));
        let future = self.inner.call(req);
        Box::pin(async move {
            let mut res: Response = future.await?;
            res.headers_mut()
                .insert("middleware", HeaderValue::from_static("after"));
            Ok(res)
        })
    }
}
