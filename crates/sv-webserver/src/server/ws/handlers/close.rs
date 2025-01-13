use std::sync::Arc;

use actix_ws::Session;
use fuel_web_utils::telemetry::Telemetry;
use uuid::Uuid;

use crate::{
    metrics::Metrics,
    server::ws::{context::WsContext, errors::WsSubscriptionError},
};

pub async fn handle_close(
    reason: Option<actix_ws::CloseReason>,
    user_id: Uuid,
    session: Session,
    telemetry: &Arc<Telemetry<Metrics>>,
) {
    tracing::info!(
        "Got close event, terminating session with reason {:?}",
        reason
    );
    let reason_str = reason.and_then(|r| r.description).unwrap_or_default();
    let ctx = WsContext::new(user_id, session, telemetry.clone());
    ctx.close_with_error(WsSubscriptionError::ClosedWithReason(
        reason_str.to_string(),
    ))
    .await;
}
