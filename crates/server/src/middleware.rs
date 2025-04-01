use actix_web::{
    App, Error, HttpResponse, HttpServer,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc;
use std::task::{Context, Poll};

/// Define the middleware struct.
pub struct AuthMiddleware;

/// Implement Transform to create a new middleware instance.
impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        })
    }
}

/// The middleware that wraps the inner service.
pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Check for the "Authorization" header.
        let auth_header = req.headers().get("Authorization");
        let valid_token = if let Some(header_value) = auth_header {
            // Here you would implement your logic to verify the token.
            if let Ok(header_str) = header_value.to_str() {
                // For this example, we simply check if it starts with "Bearer "
                header_str.starts_with("Bearer ")
            } else {
                false
            }
        } else {
            false
        };

        // If token is invalid or missing, redirect to the login screen.
        if !valid_token {
            let response = HttpResponse::Found()
                .append_header(("LOCATION", "http://localhost:8080"))
                .finish();
            return Box::pin(async move { Ok(req.into_response(response)) });
        }

        // If the token is valid, forward the request.
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
