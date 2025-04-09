use std::str::FromStr;

use aws_config::Region;

use super::{StorageConfig, StorageEnv, StorageRole};

#[derive(Debug, Clone, Default)]
pub struct S3StorageOpts {
    pub env: StorageEnv,
    pub role: StorageRole,
    pub namespace: Option<String>,
}

impl StorageConfig for S3StorageOpts {
    fn new(env: StorageEnv, role: StorageRole) -> Self {
        Self {
            env,
            role,
            namespace: None,
        }
    }

    fn from_env(role: Option<StorageRole>) -> Self {
        let env = std::env::var("AWS_STORAGE_ENV")
            .map(|s| StorageEnv::from_str(&s).unwrap_or_default())
            .unwrap_or_default();

        Self {
            env,
            role: role.unwrap_or_default(),
            namespace: None,
        }
    }

    fn endpoint_url(&self) -> String {
        match self.role {
            StorageRole::Admin => dotenvy::var("AWS_ENDPOINT_URL")
                .expect("AWS_ENDPOINT_URL must be set for admin role"),
            StorageRole::Public => {
                match self.env {
                    StorageEnv::Local => "http://localhost:4566".to_string(),
                    StorageEnv::Testnet | StorageEnv::Mainnet => {
                        let bucket = self.bucket();
                        let region = self.region();
                        format!("https://{bucket}.s3-website-{region}.amazonaws.com")
                    }
                }
            }
        }
    }

    fn environment(&self) -> &StorageEnv {
        &self.env
    }

    fn role(&self) -> &StorageRole {
        &self.role
    }
}

impl S3StorageOpts {
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn region(&self) -> Region {
        let region = match &self.role {
            StorageRole::Admin => dotenvy::var("AWS_REGION")
                .expect("AWS_REGION must be set for admin role"),
            StorageRole::Public => "us-east-1".to_string(),
        };
        Region::new(region)
    }

    pub fn bucket(&self) -> String {
        if matches!(self.role, StorageRole::Admin) {
            return dotenvy::var("AWS_S3_BUCKET_NAME")
                .expect("AWS_S3_BUCKET_NAME must be set for admin role");
        }

        let base_bucket = match self.env {
            StorageEnv::Local => "fuel-streams-local",
            StorageEnv::Testnet => "fuel-streams-testnet",
            StorageEnv::Mainnet => "fuel-streams",
        };

        match &self.namespace {
            Some(ns) => format!("{base_bucket}-{ns}"),
            None => base_bucket.to_string(),
        }
    }

    pub fn credentials(&self) -> Option<aws_sdk_s3::config::Credentials> {
        match self.role {
            StorageRole::Admin => Some(aws_sdk_s3::config::Credentials::new(
                dotenvy::var("AWS_ACCESS_KEY_ID")
                    .expect("AWS_ACCESS_KEY_ID must be set for admin role"),
                dotenvy::var("AWS_SECRET_ACCESS_KEY")
                    .expect("AWS_SECRET_ACCESS_KEY must be set for admin role"),
                None,
                None,
                "static",
            )),
            StorageRole::Public => None,
        }
    }

    pub fn with_random_namespace(mut self) -> Self {
        let random_namespace = {
            use rand::Rng;
            let random_int: u32 = rand::rng().random();
            format!("namespace-{}", random_int)
        };
        self.namespace = Some(random_namespace);
        self
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_bucket_names() {
        let opts = S3StorageOpts::new(StorageEnv::Local, StorageRole::Public);
        assert_eq!(opts.bucket(), "fuel-streams-local");

        let opts = opts.with_namespace("test");
        assert_eq!(opts.bucket(), "fuel-streams-local-test");

        let opts = S3StorageOpts::new(StorageEnv::Testnet, StorageRole::Public);
        assert_eq!(opts.bucket(), "fuel-streams-testnet");

        let opts = S3StorageOpts::new(StorageEnv::Mainnet, StorageRole::Public);
        assert_eq!(opts.bucket(), "fuel-streams");
    }

    #[test]
    fn test_public_endpoint_urls() {
        let opts = S3StorageOpts::new(StorageEnv::Local, StorageRole::Public);
        assert_eq!(opts.endpoint_url(), "http://localhost:4566");

        let opts = S3StorageOpts::new(StorageEnv::Testnet, StorageRole::Public);
        assert_eq!(
            opts.endpoint_url(),
            "https://fuel-streams-testnet.s3-website-us-east-1.amazonaws.com"
        );

        let opts = S3StorageOpts::new(StorageEnv::Mainnet, StorageRole::Public);
        assert_eq!(
            opts.endpoint_url(),
            "https://fuel-streams.s3-website-us-east-1.amazonaws.com"
        );
    }
}
