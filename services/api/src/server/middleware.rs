use axum::{body::Body, http::Request, middleware::Next, response::Response};
use fuel_web_utils::api_key::{ApiKey, ApiKeyError};

pub async fn validate_scope_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiKeyError> {
    let api_key = ApiKey::from_req(&req)?;
    if !api_key.scopes().iter().any(|scope| scope.is_rest_api()) {
        tracing::warn!(
            id = %api_key.id(),
            user = %api_key.user(),
            "API key missing REST_API scope"
        );
        return Err(ApiKeyError::ScopePermission("REST_API".to_string()));
    }

    Ok(next.run(req).await)
}
