use fuel_networks::{FuelNetwork, FuelNetworkUserRole};

// Introduced for consistency.
// TODO: make it more ergonomic by probably using FuelNetwork in S3Client directly
#[derive(Debug, Clone, Default)]
pub struct S3ClientOpts {
    pub fuel_network: FuelNetwork,
    pub role: FuelNetworkUserRole,
    pub namespace: Option<String>,
}

impl S3ClientOpts {
    pub fn new(fuel_network: FuelNetwork) -> Self {
        Self {
            fuel_network,
            role: FuelNetworkUserRole::default(),
            namespace: None,
        }
    }

    pub fn admin_opts() -> Self {
        Self::new(FuelNetwork::load_from_env())
            .with_role(FuelNetworkUserRole::Admin)
    }

    pub fn with_role(self, role: FuelNetworkUserRole) -> Self {
        Self { role, ..self }
    }

    pub fn endpoint_url(&self) -> Option<String> {
        match self.role {
            FuelNetworkUserRole::Admin => dotenvy::var("AWS_ENDPOINT_URL").ok(),
            FuelNetworkUserRole::Default => Some(self.fuel_network.to_s3_url()),
        }
    }

    pub fn region(&self) -> Option<String> {
        match self.role {
            FuelNetworkUserRole::Admin => dotenvy::var("AWS_S3_REGION").ok(),
            FuelNetworkUserRole::Default => {
                Some(self.fuel_network.to_s3_region())
            }
        }
    }

    // TODO: Consider revamping and reusing NATs' Namespace here
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_random_namespace(mut self) -> Self {
        let random_namespace = {
            use rand::Rng;
            let random_int: u32 = rand::thread_rng().gen();
            format!("namespace-{}", random_int)
        };
        self.namespace = Some(random_namespace);
        self
    }

    pub fn bucket(&self) -> String {
        let bucket = match self.role {
            FuelNetworkUserRole::Admin => dotenvy::var("AWS_S3_BUCKET_NAME")
                .expect("AWS_S3_BUCKET_NAME must be set for admin role"),
            FuelNetworkUserRole::Default => self.fuel_network.to_s3_bucket(),
        };

        format!(
            "{}-{}",
            bucket,
            self.namespace.to_owned().unwrap_or_default()
        )
    }
}
