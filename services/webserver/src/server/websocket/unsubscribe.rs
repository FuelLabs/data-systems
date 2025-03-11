use actix_ws::Session;
use fuel_streams_core::{
    server::ServerResponse,
    types::{ServerRequest, Subscription},
};

use crate::server::{errors::WebsocketError, websocket::WsSession};

pub async fn unsubscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    server_request: &ServerRequest,
) -> Result<(), WebsocketError> {
    let subscriptions = server_request.subscriptions(ctx.api_key());
    for subscription in subscriptions {
        unsubscribe(session, ctx, &subscription).await?;
    }

    tracing::info!("Unsubscribed from all subscriptions");
    Ok(())
}

pub async fn unsubscribe(
    session: &mut Session,
    ctx: &WsSession,
    subscription: &Subscription,
) -> Result<(), WebsocketError> {
    tracing::info!("Unsubscribing from {}", subscription);
    let msg = ServerResponse::Unsubscribed(subscription.clone());
    ctx.send_message(session, msg).await?;
    ctx.remove_subscription(subscription).await;
    Ok(())
}
