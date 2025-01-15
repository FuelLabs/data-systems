use std::{
    collections::HashMap,
    sync::Arc,
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

use super::{validator::authorize_request, InMemoryApiKeyStorage};

pub struct ApiKeyAuth {
    api_key_register: Arc<InMemoryApiKeyStorage>,
}

impl ApiKeyAuth {
    pub fn new(api_key_register: Arc<InMemoryApiKeyStorage>) -> Self {
        ApiKeyAuth { api_key_register }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
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
    type Transform = ApiKeyAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyAuthMiddleware {
            service,
            api_key_register: self.api_key_register.clone(),
        }))
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: S,
    api_key_register: Arc<InMemoryApiKeyStorage>,
}

impl<S, B> actix_service::Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
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
                    Some((
                        key.to_string(),
                        urlencoding::decode(value).unwrap().into_owned(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Validate the api key
        let headers = req.headers().clone();
        let api_key_register = self.api_key_register.clone();
        match authorize_request((api_key_register, headers, query_map)) {
            Ok(api_key_info) => {
                req.extensions_mut().insert(api_key_info.user_id);
                req.extensions_mut().insert(api_key_info);
                Box::pin(self.service.call(req))
            }
            Err(e) => {
                let err = e.to_string();
                // If api key is invalid or missing, reject the request
                Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized(err))
                })
            }
        }
    }
}
