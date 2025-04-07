use std::fmt::Display;

use fuel_streams_types::BlockHeight;

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
}

impl Display for S3TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S3TableName::Blocks => write!(f, "blocks"),
            S3TableName::Transactions => write!(f, "transactions"),
            S3TableName::Receipts => write!(f, "receipts"),
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

    pub fn build_key(
        &self,
        start_block: BlockHeight,
        end_block: BlockHeight,
    ) -> String {
        let filename = format!("{:010}-{:010}.avro", start_block, end_block);
        format!("{}/{}/{}", self.chain, self.table, filename)
    }
}
