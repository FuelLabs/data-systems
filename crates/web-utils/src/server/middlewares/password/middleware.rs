use std::{borrow::Cow, collections::HashMap, sync::Arc};

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use tower::{Layer, Service};

use super::PasswordManager;

#[derive(Clone)]
pub struct PasswordAuthLayer {
    manager: Arc<PasswordManager>,
}

impl PasswordAuthLayer {
    pub fn new(manager: Arc<PasswordManager>) -> Self {
        Self { manager }
    }
}

impl<S> Layer<S> for PasswordAuthLayer {
    type Service = PasswordAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PasswordAuthMiddleware {
            inner,
            manager: self.manager.clone(),
        }
    }
}

#[derive(Clone)]
pub struct PasswordAuthMiddleware<S> {
    inner: S,
    manager: Arc<PasswordManager>,
}

impl<S> Service<Request> for PasswordAuthMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let manager = self.manager.clone();
        let query_map: HashMap<String, String> = req
            .uri()
            .query()
            .unwrap_or("")
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    let decoded =
                        urlencoding::decode(value).unwrap_or_else(|e| {
                            tracing::error!("Error decoding value: {}", e);
                            Cow::Borrowed(value)
                        });
                    Some((key.to_string(), decoded.into_owned()))
                } else {
                    None
                }
            })
            .collect();

        let headers = req.headers().clone();
        let call = self.inner.call(req);
        Box::pin(async move {
            let password_in_headers =
                match manager.password_from_headers(headers, query_map) {
                    Ok(password) => password,
                    Err(e) => {
                        tracing::debug!(
                            "Error extracting password from headers: {}",
                            e
                        );
                        return Ok(e.into_response());
                    }
                };

            match manager.validate_password(&password_in_headers) {
                Ok(()) => {
                    tracing::debug!("Request authenticated successfully");
                    call.await
                }
                Err(e) => {
                    tracing::debug!(
                        "Request with password verification error: {}",
                        e
                    );
                    Ok(e.into_response())
                }
            }
        })
    }
}

pub fn add_password_auth_layer(
    router: axum::Router,
    manager: Arc<PasswordManager>,
) -> axum::Router {
    router.layer(PasswordAuthLayer::new(manager))
}
