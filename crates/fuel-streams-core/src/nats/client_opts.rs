use std::time::Duration;

use async_nats::ConnectOptions;

use super::{ConnId, NatsNamespace};

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
    pub(crate) conn_id: ConnId,
    pub(crate) namespace: NatsNamespace,
    pub(crate) timeout_secs: u64,
}

impl ClientOpts {
    pub fn new(url: impl ToString) -> Self {
        Self {
            url: url.to_string(),
            role: NatsUserRole::default(),
            conn_id: ConnId::default(),
            namespace: NatsNamespace::default(),
            timeout_secs: 5,
        }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn public_opts(url: impl ToString) -> Self {
        Self::new(url).with_role(NatsUserRole::Public)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn admin_opts(url: impl ToString) -> Self {
        Self::new(url).with_role(NatsUserRole::Admin)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_role(self, role: NatsUserRole) -> Self {
        Self { role, ..self }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_conn_id(self, conn_id: ConnId) -> Self {
        Self { conn_id, ..self }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_namespace(self, namespace: &str) -> Self {
        let namespace = namespace.into();
        Self { namespace, ..self }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_rdn_namespace(self) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_int: u32 = rng.gen();
        let namespace = format!(r"namespace-{random_int}").into();
        Self { namespace, ..self }
    }

    pub fn with_timeout(self, secs: u64) -> Self {
        Self {
            timeout_secs: secs,
            ..self
        }
    }

    pub(super) fn connect_opts(&self) -> ConnectOptions {
        let (user, pass) = match self.role {
            NatsUserRole::Admin => (
                "admin",
                dotenvy::var("NATS_ADMIN_PASS")
                    .expect("`NATS_ADMIN_PASS` env must be set"),
            ),
            NatsUserRole::Public => (
                "public",
                dotenvy::var("NATS_PUBLIC_PASS")
                    .expect("`NATS_PUBLIC_PASS` env must be set"),
            ),
        };

        ConnectOptions::with_user_and_password(user.into(), pass)
            .connection_timeout(Duration::from_secs(self.timeout_secs))
            .max_reconnects(1)
            .name(&self.conn_id)
    }
}
