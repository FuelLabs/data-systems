use std::time::Duration;

use async_nats::ConnectOptions;

use super::NatsNamespace;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum NatsAuth {
    Admin,
    System,
    #[default]
    Public,
    Custom(String, String),
}

impl NatsAuth {
    fn credentials_from_env(&self) -> (String, String) {
        match self {
            NatsAuth::Admin => (
                dotenvy::var("NATS_ADMIN_USER")
                    .expect("NATS_ADMIN_USER must be set"),
                dotenvy::var("NATS_ADMIN_PASS")
                    .expect("NATS_ADMIN_PASS must be set"),
            ),
            NatsAuth::System => (
                dotenvy::var("NATS_SYS_USER")
                    .expect("NATS_SYS_USER must be set"),
                dotenvy::var("NATS_SYS_PASS")
                    .expect("NATS_SYS_PASS must be set"),
            ),
            NatsAuth::Public => ("default_user".to_string(), "".to_string()),
            NatsAuth::Custom(user, pass) => {
                (user.to_string(), pass.to_string())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct NatsClientOpts {
    /// The URL of the NATS server.
    pub(crate) url: String,
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
    pub fn new(url: String) -> Self {
        Self {
            url,
            namespace: NatsNamespace::default(),
            timeout_secs: 5,
            domain: None,
            user: None,
            password: None,
        }
    }

    pub fn admin_opts() -> Self {
        Self::from_env(Some(NatsAuth::Admin))
    }
    pub fn system_opts() -> Self {
        Self::from_env(Some(NatsAuth::System))
    }
    pub fn public_opts() -> Self {
        Self::from_env(Some(NatsAuth::Public))
    }

    pub fn from_env(auth: Option<NatsAuth>) -> Self {
        let url = dotenvy::var("NATS_URL").expect("NATS_URL must be set");
        let (user, pass) = auth.unwrap_or_default().credentials_from_env();
        Self::new(url).with_user(user).with_password(pass)
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn with_url(self, url: String) -> Self {
        Self { url, ..self }
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
        let opts = match (self.user.clone(), self.password.clone()) {
            (Some(user), Some(pass)) => {
                ConnectOptions::with_user_and_password(user, pass)
            }
            _ => ConnectOptions::new(),
        };

        opts.connection_timeout(Duration::from_secs(self.timeout_secs))
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

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_role_credentials() {
        // Setup
        env::set_var("NATS_ADMIN_USER", "admin");
        env::set_var("NATS_ADMIN_PASS", "admin_pass");

        // Test Admin role credentials
        let (user, pass) = NatsAuth::Admin.credentials_from_env();
        assert_eq!(user, "admin");
        assert_eq!(pass, "admin_pass");

        // Cleanup
        env::remove_var("NATS_ADMIN_USER");
        env::remove_var("NATS_ADMIN_PASS");
    }

    #[test]
    fn test_from_env_with_role() {
        // Setup
        env::set_var("NATS_URL", "nats://localhost:4222");
        env::set_var("NATS_ADMIN_USER", "admin");
        env::set_var("NATS_ADMIN_PASS", "admin_pass");

        // Test Admin role
        let opts = NatsClientOpts::from_env(Some(NatsAuth::Admin));
        assert_eq!(opts.user, Some("admin".to_string()));
        assert_eq!(opts.password, Some("admin_pass".to_string()));

        // Cleanup
        env::remove_var("NATS_URL");
        env::remove_var("NATS_ADMIN_USER");
        env::remove_var("NATS_ADMIN_PASS");
    }
}
