use std::collections::HashMap;

use actix_web::{web, HttpRequest, Responder};
use actix_ws::Message;
use futures_util::StreamExt as _;
use tokio::sync::RwLock;

use crate::server::state::ServerState;

// static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub type WsClients = RwLock<HashMap<usize, WsClient>>;

pub type OutBoundWsChannel = tokio::sync::broadcast::Sender<
    std::result::Result<Message, actix_web::Error>,
>;

#[derive(Debug, Clone)]
pub struct WsClient {
    pub socket_id: usize,
    pub user_id: uuid::Uuid,
    pub sender: Option<OutBoundWsChannel>,
}

pub async fn get_ws(
    req: HttpRequest,
    body: web::Payload,
    _state: web::Data<ServerState>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(msg) => println!("Got text: {msg}"),
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}
