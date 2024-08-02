use std::time::Duration;

use async_nats::ConnectOptions;

use super::{types::AsyncNatsClient, ConnId, NatsError};

#[derive(Debug, Clone, Default)]
pub enum NatsUserRole {
    Admin,
    #[default]
    Public,
}

#[derive(Debug, Clone)]
pub struct ClientOpts {
    pub(crate) url: String,
    pub(crate) role: NatsUserRole,
    // This ID is being used as name within the ConnectOptions
    pub(crate) conn_id: ConnId,
    // This is being used as prefix for nats streams, consumers and subjects names
    pub(crate) nats_prefix: String,
    pub(crate) timeout_secs: u64,
}

impl ClientOpts {
    pub fn new(url: impl ToString, conn_id: ConnId) -> Self {
        Self {
            url: url.to_string(),
            nats_prefix: conn_id.clone().into(),
            conn_id,
            role: NatsUserRole::default(),
            timeout_secs: 5,
        }
    }

    pub fn public_opts(url: impl ToString, conn_id: ConnId) -> Self {
        Self::new(url, conn_id).with_role(NatsUserRole::Public)
    }

    pub fn admin_opts(url: impl ToString, conn_id: ConnId) -> Self {
        Self::new(url, conn_id).with_role(NatsUserRole::Admin)
    }

    pub fn with_role(self, role: NatsUserRole) -> Self {
        Self { role, ..self }
    }

    pub fn with_conn_id(self, conn_id: ConnId) -> Self {
        Self { conn_id, ..self }
    }

    pub fn with_rnd_conn_id(self) -> Self {
        let conn_id = ConnId::rnd();
        Self { conn_id, ..self }
    }

    pub fn with_prefix(self, prefix: &str) -> Self {
        let prefix = prefix.into();
        Self {
            nats_prefix: prefix,
            ..self
        }
    }

    pub fn with_timeout(self, secs: u64) -> Self {
        Self {
            timeout_secs: secs,
            ..self
        }
    }

    pub(super) async fn connect(self) -> Result<AsyncNatsClient, NatsError> {
        let (user, pass) = match self.role {
            NatsUserRole::Admin => {
                ("admin", dotenvy::var("NATS_ADMIN_PASS").unwrap())
            }
            NatsUserRole::Public => {
                ("public", dotenvy::var("NATS_PUBLIC_PASS").unwrap())
            }
        };

        ConnectOptions::with_user_and_password(user.into(), pass)
            .connection_timeout(Duration::from_secs(self.timeout_secs))
            .max_reconnects(1)
            .name(&self.conn_id)
            .connect(self.url.to_string())
            .await
            .map_err(|e| NatsError::ConnectionError {
                url: self.url.to_owned(),
                source: e,
            })
    }
}
