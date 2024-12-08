use async_nats::{jetstream::stream::State as StreamState, RequestErrorKind};
use fuel_streams::types::Log;
use fuel_streams_core::prelude::*;

#[derive(Clone, Debug)]
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

#[cfg_attr(test, mockall::automock)]
impl FuelStreams {
    pub async fn new(nats_client: &NatsClient) -> Self {
        Self {
            transactions: Stream::<Transaction>::new(nats_client).await,
            blocks: Stream::<Block>::new(nats_client).await,
            inputs: Stream::<Input>::new(nats_client).await,
            outputs: Stream::<Output>::new(nats_client).await,
            receipts: Stream::<Receipt>::new(nats_client).await,
            utxos: Stream::<Utxo>::new(nats_client).await,
            logs: Stream::<Log>::new(nats_client).await,
        }
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
