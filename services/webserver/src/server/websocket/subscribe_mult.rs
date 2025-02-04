use actix_ws::Session;
use fuel_streams_core::server::SubscriptionPayload;

use super::WsSession;
use crate::server::{errors::WebsocketError, websocket::subscribe};

pub async fn subscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    subs: Vec<SubscriptionPayload>,
) -> Result<(), WebsocketError> {
    let handles: Vec<_> = subs
        .into_iter()
        .map(|payload| {
            let mut ctx = ctx.clone();
            let mut session = session.clone();
            let api_key = ctx.api_key().clone();
            let subscription = (api_key, payload);
            tokio::spawn(async move {
                subscribe(&mut session, &mut ctx, &subscription.into()).await?;
                Ok::<(), WebsocketError>(())
            })
        })
        .collect();
    let res = futures::future::join_all(handles).await;
}
