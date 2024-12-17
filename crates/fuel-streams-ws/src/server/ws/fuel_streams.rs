use std::sync::Arc;

use async_nats::{
    jetstream::{context::CreateStreamErrorKind, stream::State as StreamState},
    RequestErrorKind,
};
use fuel_streams::types::Log;
use fuel_streams_core::{prelude::*, SubscriptionConfig};
use futures::stream::BoxStream;

#[derive(Clone)]
/// Streams we currently support publishing to.
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
    pub async fn new(
        nats_client: &NatsClient,
        s3_client: &Arc<S3Client>,
    ) -> Self {
        Self {
            transactions: Stream::<Transaction>::new(nats_client, s3_client)
                .await,
            blocks: Stream::<Block>::new(nats_client, s3_client).await,
            inputs: Stream::<Input>::new(nats_client, s3_client).await,
            outputs: Stream::<Output>::new(nats_client, s3_client).await,
            receipts: Stream::<Receipt>::new(nats_client, s3_client).await,
            utxos: Stream::<Utxo>::new(nats_client, s3_client).await,
            logs: Stream::<Log>::new(nats_client, s3_client).await,
        }
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
}

#[async_trait::async_trait]
pub trait FuelStreamsExt: Sync + Send + 'static {
    fn blocks(&self) -> &Stream<Block>;
    fn transactions(&self) -> &Stream<Transaction>;
    fn inputs(&self) -> &Stream<Input>;
    fn outputs(&self) -> &Stream<Output>;
    fn receipts(&self) -> &Stream<Receipt>;
    fn utxos(&self) -> &Stream<Utxo>;
    fn logs(&self) -> &Stream<Log>;

    async fn get_last_published_block(&self) -> anyhow::Result<Option<Block>>;

    fn subjects_names() -> &'static [&'static str] {
        &[
            Transaction::NAME,
            Block::NAME,
            Input::NAME,
            Receipt::NAME,
            Utxo::NAME,
            Log::NAME,
        ]
    }

    fn is_within_subject_names(subject_name: &str) -> bool {
        let subject_names = Self::subjects_names();
        subject_names.contains(&subject_name)
    }

    fn subjects_wildcards(&self) -> &[&'static str] {
        &[
            TransactionsSubject::WILDCARD,
            BlocksSubject::WILDCARD,
            InputsByIdSubject::WILDCARD,
            InputsCoinSubject::WILDCARD,
            InputsMessageSubject::WILDCARD,
            InputsContractSubject::WILDCARD,
            ReceiptsLogSubject::WILDCARD,
            ReceiptsBurnSubject::WILDCARD,
            ReceiptsByIdSubject::WILDCARD,
            ReceiptsCallSubject::WILDCARD,
            ReceiptsMintSubject::WILDCARD,
            ReceiptsPanicSubject::WILDCARD,
            ReceiptsReturnSubject::WILDCARD,
            ReceiptsRevertSubject::WILDCARD,
            ReceiptsLogDataSubject::WILDCARD,
            ReceiptsTransferSubject::WILDCARD,
            ReceiptsMessageOutSubject::WILDCARD,
            ReceiptsReturnDataSubject::WILDCARD,
            ReceiptsTransferOutSubject::WILDCARD,
            ReceiptsScriptResultSubject::WILDCARD,
            UtxosSubject::WILDCARD,
            LogsSubject::WILDCARD,
        ]
    }

    fn wildcards() -> Vec<&'static str> {
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

    async fn get_consumers_and_state(
        &self,
    ) -> Result<Vec<(String, Vec<String>, StreamState)>, RequestErrorKind>;

    #[cfg(feature = "test-helpers")]
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
        Ok(self
            .blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await?)
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

    #[cfg(feature = "test-helpers")]
    async fn is_empty(&self) -> bool {
        self.blocks.is_empty(BlocksSubject::WILDCARD).await
            && self
                .transactions
                .is_empty(TransactionsSubject::WILDCARD)
                .await
    }
}
