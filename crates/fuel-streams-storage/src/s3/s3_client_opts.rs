use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub enum S3Role {
    Admin,
    #[default]
    Public,
}

#[derive(Debug, Clone, Default)]
pub enum S3Env {
    #[default]
    Local,
    Testnet,
    Mainnet,
}

impl FromStr for S3Env {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(S3Env::Local),
            "testnet" => Ok(S3Env::Testnet),
            "mainnet" => Ok(S3Env::Mainnet),
            _ => Err(format!("unknown S3 type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct S3ClientOpts {
    pub s3_env: S3Env,
    pub role: S3Role,
    pub namespace: Option<String>,
}

impl S3ClientOpts {
    pub fn new(s3_env: S3Env, role: S3Role) -> Self {
        Self {
            s3_env,
            role,
            namespace: None,
        }
    }

    pub fn from_env(role: Option<S3Role>) -> Self {
        let s3_env = std::env::var("NETWORK")
            .map(|s| S3Env::from_str(&s).unwrap_or_default())
            .unwrap_or_default();

        Self {
            s3_env,
            role: role.unwrap_or_default(),
            namespace: None,
        }
    }

    pub fn admin_opts() -> Self {
        Self::from_env(Some(S3Role::Admin))
    }

    pub fn public_opts() -> Self {
        Self::from_env(Some(S3Role::Public))
    }

    pub fn endpoint_url(&self) -> String {
        match self.role {
            S3Role::Admin => dotenvy::var("AWS_ENDPOINT_URL")
                .expect("AWS_ENDPOINT_URL must be set for admin role"),
            S3Role::Public => {
                match self.s3_env {
                    S3Env::Local => "http://localhost:4566".to_string(),
                    S3Env::Testnet | S3Env::Mainnet => {
                        let bucket = self.bucket();
                        let region = self.region();
                        format!("https://{bucket}.s3-website-{region}.amazonaws.com")
                    }
                }
            }
        }
    }

    pub fn region(&self) -> String {
        match &self.role {
            S3Role::Admin => dotenvy::var("AWS_REGION")
                .expect("AWS_REGION must be set for admin role"),
            S3Role::Public => "us-east-1".to_string(),
        }
    }

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
        if matches!(self.role, S3Role::Admin) {
            return dotenvy::var("AWS_S3_BUCKET_NAME")
                .expect("AWS_S3_BUCKET_NAME must be set for admin role");
        }

        let base_bucket = match self.s3_env {
            S3Env::Local => "fuel-streams-local",
            S3Env::Testnet => "fuel-streams-testnet",
            S3Env::Mainnet => "fuel-streams",
        };

        self.namespace
            .as_ref()
            .map(|ns| format!("{base_bucket}-{ns}"))
            .unwrap_or(base_bucket.to_string())
    }
}
