use std::time::Duration;

use async_nats::ConnectOptions;
use fuel_networks::{FuelNetwork, FuelNetworkUserRole};

use super::NatsNamespace;

/// Represents options for configuring a NATS client.
///
/// # Examples
///
/// Creating a new `NatsClientOpts` instance:
///
/// ```
/// use fuel_streams_storage::nats::NatsClientOpts;
/// use fuel_networks::FuelNetwork;
///
/// let opts = NatsClientOpts::new(FuelNetwork::Local);
/// ```
///
/// Creating a public `NatsClientOpts`:
///
/// ```
/// use fuel_streams_storage::nats::NatsClientOpts;
/// use fuel_networks::FuelNetwork;
///
/// let opts = NatsClientOpts::new(FuelNetwork::Local);
/// ```
///
/// Modifying `NatsClientOpts`:
///
/// ```
/// use fuel_streams_storage::nats::NatsClientOpts;
/// use fuel_networks::{FuelNetwork, FuelNetworkUserRole};
///
/// let opts = NatsClientOpts::new(FuelNetwork::Local)
///     .with_role(FuelNetworkUserRole::Admin)
///     .with_timeout(10);
/// ```
#[derive(Debug, Clone)]
pub struct NatsClientOpts {
    pub network: FuelNetwork,
    /// The role of the user connecting to the NATS server (Admin or Public).
    pub(crate) role: FuelNetworkUserRole,
    /// The namespace used as a prefix for NATS streams, consumers, and subject names.
    pub(crate) namespace: NatsNamespace,
    /// The timeout in seconds for NATS operations.
    pub(crate) timeout_secs: u64,
}

impl NatsClientOpts {
    pub fn new(network: FuelNetwork) -> Self {
        Self {
            network,
            role: FuelNetworkUserRole::default(),
            namespace: NatsNamespace::default(),
            timeout_secs: 5,
        }
    }

    pub fn admin_opts() -> Self {
        Self::new(FuelNetwork::load_from_env())
            .with_role(FuelNetworkUserRole::Admin)
    }

    pub fn with_role(self, role: FuelNetworkUserRole) -> Self {
        Self { role, ..self }
    }

    pub fn get_url(&self) -> String {
        match self.role {
            FuelNetworkUserRole::Admin => dotenvy::var("NATS_URL")
                .expect("NATS_URL must be set for admin role"),
            FuelNetworkUserRole::Default => self.network.to_nats_url(),
        }
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
            FuelNetworkUserRole::Admin => (
                Some("admin".to_string()),
                Some(
                    dotenvy::var("NATS_ADMIN_PASS")
                        .expect("`NATS_ADMIN_PASS` env must be set"),
                ),
            ),
            FuelNetworkUserRole::Default => {
                (Some("default_user".to_string()), Some("".to_string()))
            }
        };

        match (user, pass) {
            (Some(user), Some(pass)) => {
                ConnectOptions::with_user_and_password(user, pass)
                    .connection_timeout(Duration::from_secs(self.timeout_secs))
                    .max_reconnects(1)
                    .name(Self::conn_id())
            }
            _ => ConnectOptions::new()
                .connection_timeout(Duration::from_secs(self.timeout_secs))
                .max_reconnects(1)
                .name(Self::conn_id()),
        }
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
