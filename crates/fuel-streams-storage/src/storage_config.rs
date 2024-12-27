use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub enum StorageRole {
    Admin,
    #[default]
    Public,
}

#[derive(Debug, Clone, Default)]
pub enum StorageEnv {
    #[default]
    Local,
    Testnet,
    Mainnet,
}

impl FromStr for StorageEnv {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(StorageEnv::Local),
            "testnet" => Ok(StorageEnv::Testnet),
            "mainnet" => Ok(StorageEnv::Mainnet),
            _ => Err(format!("unknown environment type: {}", s)),
        }
    }
}

pub trait StorageConfig: Send + Sync + std::fmt::Debug + Sized {
    fn new(env: StorageEnv, role: StorageRole) -> Self;
    fn from_env(role: Option<StorageRole>) -> Self;

    fn admin_opts() -> Self {
        Self::from_env(Some(StorageRole::Admin))
    }

    fn public_opts() -> Self {
        Self::from_env(Some(StorageRole::Public))
    }

    fn endpoint_url(&self) -> String;
    fn environment(&self) -> &StorageEnv;
    fn role(&self) -> &StorageRole;
}
