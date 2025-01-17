use std::time::Duration;

use async_nats::ConnectOptions;

use crate::Namespace;

#[derive(Debug, Clone)]
pub struct NatsOpts {
    pub(crate) url: String,
    pub(crate) namespace: Namespace,
    pub(crate) timeout_secs: u64,
    pub(crate) ack_wait_secs: Option<u64>,
}

impl NatsOpts {
    pub fn new(url: String) -> Self {
        Self {
            url,
            timeout_secs: 5,
            ack_wait_secs: None,
            namespace: Namespace::None,
        }
    }

    pub fn from_env() -> Self {
        let url = dotenvy::var("NATS_URL").expect("NATS_URL must be set");
        Self::new(url)
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn with_url<S: Into<String>>(self, url: S) -> Self {
        Self {
            url: url.into(),
            ..self
        }
    }

    pub fn with_ack_wait(self, secs: u64) -> Self {
        Self {
            ack_wait_secs: Some(secs),
            ..self
        }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_rdn_namespace(self) -> Self {
        let namespace = format!(r"namespace-{}", Self::random_int());
        self.with_namespace(&namespace)
    }

    pub fn with_namespace(self, namespace: &str) -> Self {
        use crate::Namespace;
        let namespace = Namespace::Custom(namespace.to_string());
        Self { namespace, ..self }
    }

    pub fn with_timeout(self, secs: u64) -> Self {
        Self {
            timeout_secs: secs,
            ..self
        }
    }

    pub(super) fn connect_opts(&self) -> ConnectOptions {
        let user = dotenvy::var("NATS_ADMIN_USER")
            .expect("NATS_ADMIN_USER must be set");
        let pass = dotenvy::var("NATS_ADMIN_PASS")
            .expect("NATS_ADMIN_PASS must be set");
        let opts = ConnectOptions::with_user_and_password(user, pass);
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
        rand::thread_rng().gen_range(0..1000000)
    }
}
