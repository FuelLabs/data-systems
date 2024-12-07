use fuel_networks::{FuelNetwork, FuelNetworkUserRole};

// Introduced for consistency.
// TODO: make it more ergonomic by probably using FuelNetwork in S3Client directly
#[derive(Debug, Clone, Default)]
pub struct S3ClientOpts {
    pub bucket: String,
    region: String,
    pub fuel_network: FuelNetwork,
    pub role: FuelNetworkUserRole,
}

impl S3ClientOpts {
    pub fn new(fuel_network: FuelNetwork) -> Self {
        Self {
            bucket: "fuel_streams".to_string(),
            region: "us-east-1".to_string(),
            fuel_network,
            role: FuelNetworkUserRole::default(),
        }
    }

    pub fn admin_opts() -> Self {
        Self::new(FuelNetwork::load_from_env())
            .with_role(FuelNetworkUserRole::Admin)
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_random_bucket(self) -> Self {
        let random_bucket = {
            use rand::Rng;
            let random_int: u32 = rand::thread_rng().gen();
            format!("fuel-streams-test-{}", random_int)
        };

        self.with_bucket(&random_bucket)
    }

    pub fn with_bucket(mut self, bucket: &str) -> Self {
        self.bucket = bucket.to_string();
        self
    }

    pub fn with_region(mut self, region: &str) -> Self {
        self.region = region.to_string();
        self
    }

    pub fn with_role(self, role: FuelNetworkUserRole) -> Self {
        Self { role, ..self }
    }

    pub fn region(&self) -> String {
        self.region.to_string()
    }

    pub fn endpoint_url(&self) -> String {
        match self.role {
            FuelNetworkUserRole::Admin => dotenvy::var("AWS_ENDPOINT_URL")
                .expect("AWS_ENDPOINT_URL must be set for admin role"),
            FuelNetworkUserRole::Default => self.fuel_network.to_s3_url(),
        }
    }
}
