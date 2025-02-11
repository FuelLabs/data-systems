use actix_ws::Session;
use fuel_streams_core::{
    server::ServerResponse,
    types::{ServerRequest, Subscription},
};
use futures::future::try_join_all;

use crate::server::{errors::WebsocketError, websocket::WsSession};

pub async fn unsubscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    server_request: &ServerRequest,
) -> Result<(), WebsocketError> {
    let subscriptions = server_request.subscriptions(&ctx.api_key());
    let handles: Vec<_> = subscriptions
        .into_iter()
        .map(|subscription| {
            let ctx = ctx.clone();
            let mut session = session.clone();
            actix_web::rt::spawn(async move {
                unsubscribe(&mut session, &ctx, &subscription).await
            })
        })
        .collect();

    match try_join_all(handles).await {
        Ok(_) => {
            tracing::info!("Unsubscribed from all subscriptions");
            Ok(())
        }
        Err(err) => {
            tracing::error!("Unsubscribe task failed: {}", err);
            Err(WebsocketError::Unsubscribe(format!(
                "Unsubscription task failed: {err}"
            )))
        }
    }
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
