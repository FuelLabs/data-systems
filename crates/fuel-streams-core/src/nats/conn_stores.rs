use super::store::{Storable, Store};
use crate::{
    nats::{NatsClient, NatsError},
    types::{Block, Transaction},
};

#[derive(Clone)]
pub struct ConnStores {
    pub blocks: Store<Block>,
    pub transactions: Store<Transaction>,
}

impl ConnStores {
    pub async fn new(client: &NatsClient) -> Result<Self, NatsError> {
        let transactions = Transaction::create_store(client).await?;
        let blocks = Block::create_store(client).await?;

        Ok(Self {
            transactions,
            blocks,
        })
    }
}

#[cfg(feature = "test-helpers")]
impl ConnStores {
    pub fn get_store_list(&self) -> Vec<&super::types::NatsStore> {
        vec![&self.blocks.store, &self.transactions.store]
    }
}
