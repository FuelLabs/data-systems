use std::sync::Arc;

use actix_ws::Session;
use bytestring::ByteString;
use futures_util::{stream::FuturesUnordered, StreamExt as _};
use tokio::sync::Mutex;

#[allow(dead_code)]
#[derive(Clone)]
struct WsClient {
    inner: Arc<Mutex<WsClientInner>>,
}

#[allow(dead_code)]
struct WsClientInner {
    sessions: Vec<Session>,
}

#[allow(dead_code)]
impl WsClient {
    fn new() -> Self {
        WsClient {
            inner: Arc::new(Mutex::new(WsClientInner {
                sessions: Vec::new(),
            })),
        }
    }

    async fn insert(&self, session: Session) {
        self.inner.lock().await.sessions.push(session);
    }

    async fn broadcast(&self, msg: impl Into<ByteString>) {
        let msg = msg.into();

        let mut inner = self.inner.lock().await;
        let mut unordered = FuturesUnordered::new();

        for mut session in inner.sessions.drain(..) {
            let msg = msg.clone();

            unordered.push(async move {
                let res = session.text(msg).await;
                res.map(|_| session)
                    .map_err(|_| tracing::debug!("Dropping session"))
            });
        }

        while let Some(res) = unordered.next().await {
            if let Ok(session) = res {
                inner.sessions.push(session);
            }
        }
    }
}
