use std::time::Duration;

use async_nats::ConnectOptions;

use super::NatsNamespace;

#[derive(Debug, Clone, Default)]
pub enum NatsUserRole {
    Admin,
    #[default]
    Public,
}

/// Represents options for configuring a NATS client.
///
/// # Examples
///
/// Creating a new `NatsClientOpts` instance:
///
/// ```
/// use fuel_streams_core::nats::NatsClientOpts;
///
/// let opts = NatsClientOpts::new("nats://localhost:4222");
/// ```
///
/// Creating a public `NatsClientOpts`:
///
/// ```
/// use fuel_streams_core::nats::NatsClientOpts;
///
/// let opts = NatsClientOpts::public_opts("nats://localhost:4222");
/// ```
///
/// Modifying `NatsClientOpts`:
///
/// ```
/// use fuel_streams_core::nats::{NatsClientOpts, NatsUserRole};
///
/// let opts = NatsClientOpts::new("nats://localhost:4222")
///     .with_role(NatsUserRole::Admin)
///     .with_timeout(10);
/// ```
#[derive(Debug, Clone)]
pub struct NatsClientOpts {
    /// The URL of the NATS server to connect to.
    pub(crate) url: String,
    /// The role of the user connecting to the NATS server (Admin or Public).
    pub(crate) role: NatsUserRole,
    /// The namespace used as a prefix for NATS streams, consumers, and subject names.
    pub(crate) namespace: NatsNamespace,
    /// The timeout in seconds for NATS operations.
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
            NatsUserRole::Public => {
                ("public", env!("NATS_PUBLIC_PASS").to_string())
            }
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
