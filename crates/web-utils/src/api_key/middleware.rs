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
pub struct ApiKeyAuth {
    db: Arc<Db>,
    manager: Arc<ApiKeysManager>,
}

impl ApiKeyAuth {
    pub fn new(manager: &Arc<ApiKeysManager>, db: &Arc<Db>) -> Self {
        ApiKeyAuth {
            db: db.to_owned(),
            manager: manager.to_owned(),
        }
    }

    pub async fn middleware(
        State(auth): State<ApiKeyAuth>,
        req: Request<Body>,
        next: Next,
    ) -> Result<Response, ApiKeyError> {
        let (mut parts, body) = req.into_parts();
        let api_key_str = ApiKeysManager::extract_api_key(&mut parts).await?;
        let api_key = auth
            .manager
            .validate_api_key(&api_key_str, &auth.db)
            .await?;

        auth.manager
            .check_subscriptions(api_key.id(), api_key.role())?;
        auth.manager
            .check_rate_limit(api_key.id(), api_key.role())?;

        api_key.validate_status()?;
        let mut req = Request::from_parts(parts, body);
        req.extensions_mut().insert(api_key.clone());
        tracing::debug!(%api_key, "Request authenticated successfully");
        let response = next.run(req).await;
        Ok(response)
    }
}
