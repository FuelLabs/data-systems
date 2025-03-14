use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use fuel_streams_store::db::Db;

use super::{ApiKeyError, ApiKeysManager};

#[derive(Clone)]
pub struct ApiKeyMiddleware;
impl ApiKeyMiddleware {
    pub async fn handler(
        State((manager, db)): State<(Arc<ApiKeysManager>, Arc<Db>)>,
        req: Request<Body>,
        next: Next,
    ) -> Result<Response, ApiKeyError> {
        let (mut parts, body) = req.into_parts();
        let api_key_str = ApiKeysManager::extract_api_key(&mut parts).await?;
        let api_key = manager.validate_api_key(&api_key_str, &db).await?;

        manager.check_subscriptions(api_key.id(), api_key.role())?;
        manager.check_rate_limit(api_key.id(), api_key.role())?;
        api_key.validate_status()?;

        let mut req = Request::from_parts(parts, body);
        req.extensions_mut().insert(api_key.clone());
        tracing::debug!(%api_key, "Request authenticated successfully");
        let response = next.run(req).await;
        Ok(response)
    }
}
