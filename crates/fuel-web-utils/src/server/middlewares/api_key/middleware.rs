use std::{
    borrow::Cow,
    collections::HashMap,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use actix_service::Transform;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error,
    HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};

use super::{rate_limiter::RateLimitsController, ApiKeyError, ApiKeysManager};

pub struct ApiKeyAuth {
    manager: Arc<ApiKeysManager>,
    rate_limiter_controller: Option<Arc<RateLimitsController>>,
}

impl ApiKeyAuth {
    pub fn new(
        manager: &Arc<ApiKeysManager>,
        rate_limit_duration: Option<Duration>,
    ) -> Self {
        ApiKeyAuth {
            manager: manager.to_owned(),
            rate_limiter_controller: rate_limit_duration
                .map(|duration| RateLimitsController::new(duration).arc()),
        }
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
            service: Arc::new(service),
            manager: self.manager.clone(),
            rate_limiter_controller: self.rate_limiter_controller.clone(),
        }))
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: Arc<S>,
    manager: Arc<ApiKeysManager>,
    rate_limiter_controller: Option<Arc<RateLimitsController>>,
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
        let manager = self.manager.clone();
        let rate_limiter_controller = self.rate_limiter_controller.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let api_key_str = manager.key_from_headers((headers, query_map))?;
            let api_key = manager
                .validate_api_key(&api_key_str)
                .await?
                .ok_or_else(|| Error::from(ApiKeyError::Invalid))?;

            if let Some(rate_limiter_controller) = rate_limiter_controller {
                if !rate_limiter_controller
                    .check_rate_limit(api_key.user_id().into())
                    .await
                {
                    return Err(actix_web::error::ErrorTooManyRequests(
                        "Rate limit per user exceeded",
                    ))
                }
            }

            match api_key.validate_status() {
                Ok(()) => {
                    tracing::debug!(
                        %api_key,
                        "Request authenticated successfully"
                    );
                    req.extensions_mut().insert(api_key);
                    service.call(req).await
                }
                Err(err) => {
                    tracing::debug!(
                        %api_key,
                        "Request authentication failed"
                    );
                    Err(Error::from(err))
                }
            }
        })
    }
}
