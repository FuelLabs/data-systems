use std::sync::Arc;

use actix_ws::Session;
use fuel_web_utils::telemetry::Telemetry;
use uuid::Uuid;

use crate::{
    metrics::Metrics,
    server::ws::{context::WsContext, errors::WsSubscriptionError},
};

pub async fn handle_unknown(
    user_id: Uuid,
    session: Session,
    telemetry: &Arc<Telemetry<Metrics>>,
) {
    tracing::error!("Received unknown message type");
    let ctx = WsContext::new(user_id, session, telemetry.clone());
    ctx.close_with_error(WsSubscriptionError::ClosedWithReason(
        "Unknown message type".to_string(),
    ))
    .await;
}
