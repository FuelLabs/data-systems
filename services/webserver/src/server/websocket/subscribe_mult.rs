use actix_ws::Session;
use fuel_streams_core::server::SubscriptionPayload;
use futures::future::try_join_all;

use super::WsSession;
use crate::server::{errors::WebsocketError, websocket::subscribe};

pub async fn subscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    subs: Vec<SubscriptionPayload>,
) -> Result<(), WebsocketError> {
    if subs.is_empty() {
        tracing::debug!("No subscriptions requested");
        return Ok(());
    }

    let handles: Vec<_> = subs
        .into_iter()
        .map(|payload| {
            let mut ctx = ctx.clone();
            let mut session = session.clone();
            let api_key = ctx.api_key().clone();
            let subscription = (api_key, payload.clone());
            actix_web::rt::spawn(async move {
                subscribe(&mut session, &mut ctx, &subscription.into()).await
            })
        })
        .collect();

    match try_join_all(handles).await {
        Ok(_) => {
            tracing::info!("All subscriptions completed successfully");
            Ok(())
        }
        Err(err) => {
            tracing::error!("Subscription task failed: {}", err);
            Err(WebsocketError::MultSubscription(format!(
                "Subscription task failed: {err}"
            )))
        }
    }
}
