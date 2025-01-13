use actix_ws::Session;

pub async fn handle_ping(session: &mut Session, bytes: &[u8]) {
    tracing::info!("Received ping, {:?}", bytes);
    if session.pong(bytes).await.is_err() {
        tracing::error!("Error sending pong, {:?}", bytes);
    }
}

pub fn handle_pong(bytes: &[u8]) {
    tracing::info!("Received pong, {:?}", bytes);
}
