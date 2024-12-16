use std::time::Duration;

use async_nats::ConnectOptions;

use super::NatsNamespace;

#[derive(Debug, Clone, Default)]
pub enum NatsUserRole {
    Admin,
    #[default]
    Default,
}

#[derive(Debug, Copy, Clone, Default, clap::ValueEnum)]
pub enum FuelNetwork {
    Local,
    #[default]
    Testnet,
    Mainnet,
}

impl FuelNetwork {
    pub fn to_url(&self) -> String {
        match self {
            FuelNetwork::Local => "nats://localhost:4222".to_string(),
            FuelNetwork::Testnet => {
                "nats://stream-testnet.fuel.network:4222".to_string()
            }
            FuelNetwork::Mainnet => {
                "nats://stream.fuel.network:4222".to_string()
            }
        }
    }
}

/// Represents options for configuring a NATS client.
///
/// # Examples
///
/// Creating a new `NatsClientOpts` instance:
///
/// ```
/// use fuel_streams_core::nats::{NatsClientOpts, FuelNetwork};
///
/// let opts = NatsClientOpts::new(Some(FuelNetwork::Local));
/// ```
///
/// Creating a public `NatsClientOpts`:
///
/// ```
/// use fuel_streams_core::nats::{NatsClientOpts, FuelNetwork};
///
/// let opts = NatsClientOpts::default_opts(Some(FuelNetwork::Local));
/// ```
///
/// Modifying `NatsClientOpts`:
///
/// ```
/// use fuel_streams_core::nats::{NatsClientOpts, NatsUserRole, FuelNetwork};
///
/// let opts = NatsClientOpts::new(Some(FuelNetwork::Local))
///     .with_role(NatsUserRole::Admin)
///     .with_timeout(10);
/// ```
#[derive(Debug, Clone)]
pub struct NatsClientOpts {
    /// The URL of the NATS server to connect to.
    url: String,
    /// The role of the user connecting to the NATS server (Admin or Public).
    pub(crate) role: NatsUserRole,
    /// The namespace used as a prefix for NATS streams, consumers, and subject names.
    pub(crate) namespace: NatsNamespace,
    /// The timeout in seconds for NATS operations.
    pub(crate) timeout_secs: u64,
    /// The domain to use for the NATS client.
    pub(crate) domain: Option<String>,
    /// The user to use for the NATS client.
    pub(crate) user: Option<String>,
    /// The password to use for the NATS client.
    pub(crate) password: Option<String>,
}

impl NatsClientOpts {
    pub fn new(network: Option<FuelNetwork>) -> Self {
        Self {
            url: network.unwrap_or_default().to_url(),
            role: NatsUserRole::default(),
            namespace: NatsNamespace::default(),
            timeout_secs: 5,
            domain: None,
            user: None,
            password: None,
        }
    }

    pub fn default_opts(network: Option<FuelNetwork>) -> Self {
        Self::new(network).with_role(NatsUserRole::Default)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn admin_opts(network: Option<FuelNetwork>) -> Self {
        Self::new(network).with_role(NatsUserRole::Admin)
    }

    pub fn with_role(self, role: NatsUserRole) -> Self {
        Self { role, ..self }
    }
    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn with_fuel_network(self, network: FuelNetwork) -> Self {
        Self {
            url: network.to_url(),
            ..self
        }
    }

    pub fn with_domain(self, domain: String) -> Self {
        Self {
            domain: Some(domain),
            ..self
        }
    }

    pub fn with_user(self, user: String) -> Self {
        Self {
            user: Some(user),
            ..self
        }
    }

    pub fn with_password(self, password: String) -> Self {
        Self {
            password: Some(password),
            ..self
        }
    }

    pub fn with_custom_url(self, url: String) -> Self {
        Self { url, ..self }
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
                Some("admin".to_string()),
                Some(
                    dotenvy::var("NATS_ADMIN_PASS")
                        .expect("`NATS_ADMIN_PASS` env must be set"),
                ),
            ),
            NatsUserRole::Default => {
                (Some("default_user".to_string()), Some("".to_string()))
            }
        };

        let (user, pass) = match (self.user.clone(), self.password.clone()) {
            (Some(user), Some(pass)) => (Some(user), Some(pass)),
            _ => (user, pass),
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
