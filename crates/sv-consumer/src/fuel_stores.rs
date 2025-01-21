use std::sync::Arc;

use fuel_streams_core::types::{
    Block,
    Input,
    Output,
    Receipt,
    Transaction,
    Utxo,
};
use fuel_streams_store::{db::Db, store::Store};

#[derive(Debug, Clone)]
pub struct FuelStores {
    pub blocks: Store<Block>,
    pub transactions: Store<Transaction>,
    pub inputs: Store<Input>,
    pub outputs: Store<Output>,
    pub receipts: Store<Receipt>,
    pub utxos: Store<Utxo>,
}

impl FuelStores {
    pub fn new(db: &Arc<Db>) -> Self {
        Self {
            blocks: Store::new(db),
            transactions: Store::new(db),
            inputs: Store::new(db),
            outputs: Store::new(db),
            receipts: Store::new(db),
            utxos: Store::new(db),
        }
    }

    #[allow(unused)]
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_namespace(&mut self, namespace: &str) -> &mut Self {
        self.blocks.with_namespace(namespace);
        self.transactions.with_namespace(namespace);
        self.inputs.with_namespace(namespace);
        self.outputs.with_namespace(namespace);
        self.receipts.with_namespace(namespace);
        self.utxos.with_namespace(namespace);
        self
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }
}
