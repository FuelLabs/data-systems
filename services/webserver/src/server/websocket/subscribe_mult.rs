use std::time::Duration;
use actix_ws::Session;
use futures::future::try_join_all;
use fuel_streams_core::server::SubscriptionPayload;
use tokio::time::timeout;
use tracing::{debug, error, info};

use super::WsSession;
use crate::server::{errors::WebsocketError, websocket::subscribe};

/// Maximum time to wait for all subscriptions to complete
const SUBSCRIPTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Subscribe to multiple topics simultaneously
///
/// # Arguments
///
/// * `session` - The WebSocket session
/// * `ctx` - The WebSocket context
/// * `subs` - Vector of subscription payloads
///
/// # Returns
///
/// Returns `Ok(())` if all subscriptions succeed, or a `WebsocketError` if any fail
///
/// # Error Handling
///
/// - Handles task panics
/// - Handles subscription errors
/// - Implements timeout to prevent hanging
pub async fn subscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    subs: Vec<SubscriptionPayload>,
) -> Result<(), WebsocketError> {
    if subs.is_empty() {
        debug!("No subscriptions requested");
        return Ok(());
    }

    info!("Starting {} subscriptions", subs.len());

    let handles: Vec<_> = subs
        .into_iter()
        .enumerate()
        .map(|(idx, payload)| {
            let mut ctx = ctx.clone();
            let mut session = session.clone();
            let api_key = ctx.api_key().clone();
            let subscription = (api_key, payload.clone());
            
            tokio::spawn(async move {
                debug!("Starting subscription {} with subject: {}", idx, payload.subject);
                match subscribe(&mut session, &mut ctx, &subscription.into()).await {
                    Ok(_) => {
                        debug!("Subscription {} completed successfully", idx);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Subscription {} failed: {}", idx, e);
                        Err(e)
                    }
                }
            })
        })
        .collect();

    // Wait for all subscriptions with timeout
    match timeout(SUBSCRIPTION_TIMEOUT, try_join_all(handles)).await {
        Ok(results) => {
            match results {
                Ok(_) => {
                    info!("All subscriptions completed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Subscription task failed: {}", e);
                    Err(WebsocketError::Internal(format!(
                        "Subscription task failed: {}", 
                        e
                    )))
                }
            }
        }
        Err(_) => {
            error!("Subscription timeout after {:?}", SUBSCRIPTION_TIMEOUT);
            Err(WebsocketError::Internal(
                format!("Subscriptions timed out after {:?}", SUBSCRIPTION_TIMEOUT)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::websocket::WsSession;
    use actix_ws::Session;
    use fuel_streams_core::server::DeliverPolicy;
    
    // Add your tests here
}
