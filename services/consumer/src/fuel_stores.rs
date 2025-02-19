use std::sync::Arc;

use fuel_streams_core::types::{
    Block,
    Input,
    Output,
    Receipt,
    Transaction,
    Utxo,
};
use fuel_streams_domains::{
    blocks::BlockDbItem,
    inputs::InputDbItem,
    outputs::OutputDbItem,
    receipts::ReceiptDbItem,
    transactions::TransactionDbItem,
    utxos::UtxoDbItem,
};
use fuel_streams_store::{
    db::Db,
    record::{DbTransaction, RecordEntity, RecordPacket},
    store::Store,
};

use crate::errors::ConsumerError;

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

    pub async fn insert_by_entity(
        &self,
        db_tx: &mut DbTransaction,
        packet: &RecordPacket,
    ) -> Result<(), ConsumerError> {
        let subject_id = packet.subject_id();
        let entity = RecordEntity::from_subject_id(&subject_id)?;
        match entity {
            RecordEntity::Block => {
                let db_item: BlockDbItem = packet.try_into()?;
                self.blocks
                    .insert_record_with_transaction(db_tx, &db_item)
                    .await?;
            }
            RecordEntity::Transaction => {
                let db_item: TransactionDbItem = packet.try_into()?;
                self.transactions
                    .insert_record_with_transaction(db_tx, &db_item)
                    .await?;
            }
            RecordEntity::Input => {
                let db_item: InputDbItem = packet.try_into()?;
                self.inputs
                    .insert_record_with_transaction(db_tx, &db_item)
                    .await?;
            }
            RecordEntity::Output => {
                let db_item: OutputDbItem = packet.try_into()?;
                self.outputs
                    .insert_record_with_transaction(db_tx, &db_item)
                    .await?;
            }
            RecordEntity::Receipt => {
                let db_item: ReceiptDbItem = packet.try_into()?;
                self.receipts
                    .insert_record_with_transaction(db_tx, &db_item)
                    .await?;
            }
            RecordEntity::Utxo => {
                let db_item: UtxoDbItem = packet.try_into()?;
                self.utxos
                    .insert_record_with_transaction(db_tx, &db_item)
                    .await?;
            }
        };
        Ok(())
    }
}
