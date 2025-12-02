use std::{
    fmt::Display,
    sync::LazyLock,
};

use fuel_streams_types::BlockHeight;

pub static BUCKET_PREFIX: LazyLock<String> = LazyLock::new(|| {
    dotenvy::var("BUCKET_PREFIX")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or("v1".to_string())
});

#[derive(Debug, Clone, Copy, Default, derive_more::Display)]
pub enum FuelNetwork {
    #[display("mainnet")]
    Mainnet,
    #[display("testnet")]
    Testnet,
    #[display("devnet")]
    Devnet,
    #[default]
    #[display("local")]
    Local,
}

impl FuelNetwork {
    pub fn load_from_env() -> Self {
        let network = dotenvy::var("NETWORK").expect("NETWORK must be set");
        match network.as_str() {
            "testnet" => Self::Testnet,
            "mainnet" => Self::Mainnet,
            "staging" => Self::Devnet,
            _ => Self::Local,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum S3TableName {
    #[default]
    Blocks,
    Transactions,
    Receipts,
    Metadata,
}

impl Display for S3TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S3TableName::Blocks => write!(f, "blocks"),
            S3TableName::Transactions => write!(f, "transactions"),
            S3TableName::Receipts => write!(f, "receipts"),
            S3TableName::Metadata => {
                write!(f, "metadata")
            }
        }
    }
}

pub struct S3KeyBuilder {
    chain: FuelNetwork,
    table: S3TableName,
}

impl S3KeyBuilder {
    pub fn new(chain: FuelNetwork) -> Self {
        Self {
            chain,
            table: S3TableName::default(),
        }
    }

    pub fn with_table(mut self, table: S3TableName) -> Self {
        self.table = table;
        self
    }

    pub fn with_chain(mut self, chain: FuelNetwork) -> Self {
        self.chain = chain;
        self
    }

    pub fn build_key(&self, filename: &str) -> String {
        format!(
            "{}/{}/{}/{}",
            *BUCKET_PREFIX, self.chain, self.table, filename
        )
    }

    pub fn build_key_from_heights(
        &self,
        start_block: BlockHeight,
        end_block: BlockHeight,
    ) -> String {
        let filename = format!("{:010}-{:010}.avro", start_block, end_block);
        format!(
            "{}/{}/{}/{}",
            *BUCKET_PREFIX, self.chain, self.table, filename
        )
    }
}
