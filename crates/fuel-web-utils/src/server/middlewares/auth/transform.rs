use std::{
    collections::HashMap,
    task::{Context, Poll},
};

use actix_service::Transform;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error,
    HttpMessage,
    Result,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};

use super::jwt::authorize_request;

pub struct JwtAuth {
    jwt_secret: String,
}

impl JwtAuth {
    pub fn new(jwt_secret: String) -> Self {
        JwtAuth { jwt_secret }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
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
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service,
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
    jwt_secret: String,
}

impl<S, B> actix_service::Service<ServiceRequest> for JwtAuthMiddleware<S>
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
        let jwt_secret = self.jwt_secret.clone();
        let query_map: HashMap<String, String> = req
            .query_string()
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    Some((
                        key.to_string(),
                        urlencoding::decode(value).unwrap().into_owned(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Validate the JWT
        let headers = req.headers().clone();
        match authorize_request((jwt_secret, headers, query_map)) {
            Ok((user_id, _jwt)) => {
                req.extensions_mut().insert(user_id);
                Box::pin(self.service.call(req))
            }
            Err(e) => {
                let err = e.to_string();
                // If JWT is invalid or missing, reject the request
                Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized(err))
                })
            }
        }
    }
}
