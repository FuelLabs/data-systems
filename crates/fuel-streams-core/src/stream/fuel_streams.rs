use std::sync::Arc;

use async_nats::{
    jetstream::{context::CreateStreamErrorKind, stream::State as StreamState},
    RequestErrorKind,
};
use futures::stream::BoxStream;

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

pub struct FuelStreamsUtils;
impl FuelStreamsUtils {
    pub fn is_within_subject_names(subject_name: &str) -> bool {
        let subject_names = Self::subjects_names();
        subject_names.contains(&subject_name)
    }

    pub fn subjects_names() -> &'static [&'static str] {
        &[
            Transaction::NAME,
            Block::NAME,
            Input::NAME,
            Receipt::NAME,
            Utxo::NAME,
            Log::NAME,
        ]
    }

    pub fn wildcards() -> Vec<&'static str> {
        let nested_wildcards = [
            Transaction::WILDCARD_LIST,
            Block::WILDCARD_LIST,
            Input::WILDCARD_LIST,
            Receipt::WILDCARD_LIST,
            Utxo::WILDCARD_LIST,
            Log::WILDCARD_LIST,
        ];
        nested_wildcards
            .into_iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>()
    }
}

impl FuelStreams {
    pub async fn new(
        nats_client: &NatsClient,
        storage: &Arc<S3Storage>,
    ) -> Self {
        Self {
            transactions: Stream::<Transaction>::new(nats_client, storage)
                .await,
            blocks: Stream::<Block>::new(nats_client, storage).await,
            inputs: Stream::<Input>::new(nats_client, storage).await,
            outputs: Stream::<Output>::new(nats_client, storage).await,
            receipts: Stream::<Receipt>::new(nats_client, storage).await,
            utxos: Stream::<Utxo>::new(nats_client, storage).await,
            logs: Stream::<Log>::new(nats_client, storage).await,
        }
    }

    pub async fn setup_all(
        core_client: &NatsClient,
        publisher_client: &NatsClient,
        storage: &Arc<S3Storage>,
    ) -> (Self, Self) {
        let core_stream = Self::new(core_client, storage).await;
        let publisher_stream = Self::new(publisher_client, storage).await;
        (core_stream, publisher_stream)
    }

    pub async fn subscribe(
        &self,
        sub_subject: &str,
        subscription_config: Option<SubscriptionConfig>,
    ) -> Result<BoxStream<'_, Vec<u8>>, StreamError> {
        match sub_subject {
            Transaction::NAME => {
                self.transactions.subscribe_raw(subscription_config).await
            }
            Block::NAME => self.blocks.subscribe_raw(subscription_config).await,
            Input::NAME => self.inputs.subscribe_raw(subscription_config).await,
            Output::NAME => {
                self.outputs.subscribe_raw(subscription_config).await
            }
            Receipt::NAME => {
                self.receipts.subscribe_raw(subscription_config).await
            }
            Utxo::NAME => self.utxos.subscribe_raw(subscription_config).await,
            Log::NAME => self.logs.subscribe_raw(subscription_config).await,
            _ => Err(StreamError::StreamCreation(
                CreateStreamErrorKind::InvalidStreamName.into(),
            )),
        }
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

    async fn get_last_published_block(&self) -> anyhow::Result<Option<Block>>;
    async fn get_consumers_and_state(
        &self,
    ) -> Result<Vec<(String, Vec<String>, StreamState)>, RequestErrorKind>;

    #[cfg(any(test, feature = "test-helpers"))]
    async fn is_empty(&self) -> bool;
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

    async fn get_last_published_block(&self) -> anyhow::Result<Option<Block>> {
        self.blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await
            .map_err(|e| e.into())
    }

    async fn get_consumers_and_state(
        &self,
    ) -> Result<Vec<(String, Vec<String>, StreamState)>, RequestErrorKind> {
        Ok(vec![
            self.transactions.get_consumers_and_state().await?,
            self.blocks.get_consumers_and_state().await?,
            self.inputs.get_consumers_and_state().await?,
            self.outputs.get_consumers_and_state().await?,
            self.receipts.get_consumers_and_state().await?,
            self.utxos.get_consumers_and_state().await?,
            self.logs.get_consumers_and_state().await?,
        ])
    }

    #[cfg(any(test, feature = "test-helpers"))]
    async fn is_empty(&self) -> bool {
        self.blocks.is_empty(BlocksSubject::WILDCARD).await
            && self
                .transactions
                .is_empty(TransactionsSubject::WILDCARD)
                .await
    }
}
