use std::fmt;

use async_trait::async_trait;
use streams_core::nats::{ConnId, NatsConn};

use super::{ClientError, ConnectionResult};

pub(crate) static CONN_ID: &str = "fuel";

#[async_trait]
pub trait ClientConn: Clone + fmt::Debug + Send {
    fn new(url: impl ToString + Send) -> Self;
    async fn connect(self) -> ConnectionResult<Self>;

    #[cfg(feature = "test_helpers")]
    fn with_conn_id(self, conn_id: ConnId) -> Self;
}

#[derive(Debug, Clone)]
pub struct Client {
    url: String,
    conn_id: Option<ConnId>,
    #[allow(unused)]
    conn: Option<NatsConn>,
}

#[async_trait]
impl ClientConn for Client {
    fn new(url: impl ToString + Send) -> Self {
        Self {
            url: url.to_string(),
            conn_id: None,
            conn: None,
        }
    }

    #[cfg(feature = "test_helpers")]
    fn with_conn_id(self, conn_id: ConnId) -> Self {
        Self {
            conn_id: Some(conn_id),
            ..self
        }
    }

    async fn connect(self) -> ConnectionResult<Self> {
        let url = self.url.clone();
        let conn_id = match self.conn_id.clone() {
            Some(value) => value,
            None => ConnId::new(CONN_ID),
        };

        let conn = NatsConn::as_public(&url, conn_id.to_owned())
            .await
            .map_err(|s| ClientError::ConnectionError { url, source: s })?;

        Ok(Self {
            conn: Some(conn),
            ..self
        })
    }
}
