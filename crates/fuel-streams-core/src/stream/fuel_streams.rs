use std::sync::Arc;

use fuel_streams_store::db::Db;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct FuelStreams {
    pub transactions: Stream<Transaction>,
    pub blocks: Stream<Block>,
    pub inputs: Stream<Input>,
    pub outputs: Stream<Output>,
    pub receipts: Stream<Receipt>,
    pub utxos: Stream<Utxo>,
    pub logs: Stream<Log>,
}

impl FuelStreams {
    pub async fn new(nats_client: &NatsClient, db: &Arc<Db>) -> Self {
        Self {
            transactions: Stream::<Transaction>::get_or_init(nats_client, db)
                .await,
            blocks: Stream::<Block>::get_or_init(nats_client, db).await,
            inputs: Stream::<Input>::get_or_init(nats_client, db).await,
            outputs: Stream::<Output>::get_or_init(nats_client, db).await,
            receipts: Stream::<Receipt>::get_or_init(nats_client, db).await,
            utxos: Stream::<Utxo>::get_or_init(nats_client, db).await,
            logs: Stream::<Log>::get_or_init(nats_client, db).await,
        }
    }

    pub async fn setup_all(
        core_client: &NatsClient,
        publisher_client: &NatsClient,
        db: &Arc<Db>,
    ) -> (Self, Self) {
        let core_stream = Self::new(core_client, db).await;
        let publisher_stream = Self::new(publisher_client, db).await;
        (core_stream, publisher_stream)
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[async_trait::async_trait]
pub trait FuelStreamsExt: Sync + Send {
    fn blocks(&self) -> &Stream<Block>;
    fn transactions(&self) -> &Stream<Transaction>;
    fn inputs(&self) -> &Stream<Input>;
    fn outputs(&self) -> &Stream<Output>;
    fn receipts(&self) -> &Stream<Receipt>;
    fn utxos(&self) -> &Stream<Utxo>;
    fn logs(&self) -> &Stream<Log>;

    // async fn get_last_published_block(&self) -> anyhow::Result<Option<Block>>;
    // async fn get_consumers_and_state(
    //     &self,
    // ) -> Result<Vec<(String, Vec<String>, StreamState)>, RequestErrorKind>;

    // #[cfg(any(test, feature = "test-helpers"))]
    // async fn is_empty(&self) -> bool;
}

#[async_trait::async_trait]
impl FuelStreamsExt for FuelStreams {
    fn blocks(&self) -> &Stream<Block> {
        &self.blocks
    }
    fn transactions(&self) -> &Stream<Transaction> {
        &self.transactions
    }
    fn inputs(&self) -> &Stream<Input> {
        &self.inputs
    }
    fn outputs(&self) -> &Stream<Output> {
        &self.outputs
    }
    fn receipts(&self) -> &Stream<Receipt> {
        &self.receipts
    }
    fn utxos(&self) -> &Stream<Utxo> {
        &self.utxos
    }
    fn logs(&self) -> &Stream<Log> {
        &self.logs
    }

    // async fn get_last_published_block(&self) -> anyhow::Result<Option<Block>> {
    //     self.blocks
    //         .get_last_published(BlocksSubject::WILDCARD)
    //         .await
    //         .map_err(|e| e.into())
    // }

    // async fn get_consumers_and_state(
    //     &self,
    // ) -> Result<Vec<(String, Vec<String>, StreamState)>, RequestErrorKind> {
    //     Ok(vec![
    //         self.transactions.get_consumers_and_state().await?,
    //         self.blocks.get_consumers_and_state().await?,
    //         self.inputs.get_consumers_and_state().await?,
    //         self.outputs.get_consumers_and_state().await?,
    //         self.receipts.get_consumers_and_state().await?,
    //         self.utxos.get_consumers_and_state().await?,
    //         self.logs.get_consumers_and_state().await?,
    //     ])
    // }

    // #[cfg(any(test, feature = "test-helpers"))]
    // async fn is_empty(&self) -> bool {
    //     self.blocks.is_empty(BlocksSubject::WILDCARD).await
    //         && self
    //             .transactions
    //             .is_empty(TransactionsSubject::WILDCARD)
    //             .await
    // }
}
