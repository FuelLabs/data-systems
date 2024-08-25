use std::time::Duration;

use async_nats::ConnectOptions;

use super::NatsNamespace;

#[derive(Debug, Clone, Default)]
pub enum NatsUserRole {
    Admin,
    #[default]
    Public,
}

#[derive(Debug, Clone)]
pub struct NatsClientOpts {
    pub(crate) url: String,
    pub(crate) role: NatsUserRole,
    // This is being used as prefix for nats streams, consumers and subjects names
    pub(crate) namespace: NatsNamespace,
    pub(crate) timeout_secs: u64,
}

impl NatsClientOpts {
    pub fn new(url: impl ToString) -> Self {
        Self {
            url: url.to_string(),
            role: NatsUserRole::default(),
            namespace: NatsNamespace::default(),
            timeout_secs: 5,
        }
    }

    pub fn public_opts(url: impl ToString) -> Self {
        Self::new(url).with_role(NatsUserRole::Public)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn admin_opts(url: impl ToString) -> Self {
        Self::new(url).with_role(NatsUserRole::Admin)
    }

    pub fn with_role(self, role: NatsUserRole) -> Self {
        Self { role, ..self }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_rdn_namespace(self) -> Self {
        let namespace = format!(r"namespace-{}", Self::random_int());
        self.with_namespace(&namespace)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_namespace(self, namespace: &str) -> Self {
        let namespace = NatsNamespace::Custom(namespace.to_string());
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
            .name(Self::conn_id())
    }

    // This will be useful for debugging and monitoring connections
    fn conn_id() -> String {
        format!(r"connection-{}", Self::random_int())
    }

    fn random_int() -> u32 {
        use rand::Rng;
        rand::thread_rng().gen()
    }
}
