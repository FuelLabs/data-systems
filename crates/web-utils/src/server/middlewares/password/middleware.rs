use std::{
    borrow::Cow,
    collections::HashMap,
    sync::Arc,
    task::{Context, Poll},
};

use actix_service::Transform;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};

use super::PasswordManager;

pub struct PasswordAuth {
    manager: Arc<PasswordManager>,
}

impl PasswordAuth {
    pub fn new(manager: &Arc<PasswordManager>) -> Self {
        PasswordAuth {
            manager: manager.to_owned(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for PasswordAuth
where
    S: actix_service::Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = Error,
        > + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PasswordAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PasswordAuthMiddleware {
            service: Arc::new(service),
            manager: self.manager.clone(),
        }))
    }
}

pub struct PasswordAuthMiddleware<S> {
    service: Arc<S>,
    manager: Arc<PasswordManager>,
}

impl<S, B> actix_service::Service<ServiceRequest> for PasswordAuthMiddleware<S>
where
    S: actix_service::Service<
            ServiceRequest,
            Response = ServiceResponse<B>,
            Error = Error,
        > + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let query_map: HashMap<String, String> = req
            .query_string()
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
        let password_in_headers =
            match self.manager.password_from_headers((headers, query_map)) {
                Ok(password) => password,
                Err(e) => {
                    let err = e.to_string();
                    tracing::debug!(
                        "Error extracting password from headers: {}",
                        err
                    );
                    return Box::pin(async { Err(Error::from(e)) });
                }
            };

        match self.manager.validate_password(&password_in_headers) {
            Ok(()) => {
                tracing::debug!("Request authenticated successfully");
                Box::pin(self.service.call(req))
            }
            Err(e) => {
                let err = e.to_string();
                tracing::debug!(
                    "Request with password verification error: {}",
                    err
                );
                // If password is invalid or missing, reject the request
                Box::pin(async { Err(Error::from(e)) })
            }
        }
    }
}
