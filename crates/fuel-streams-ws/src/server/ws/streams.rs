use async_nats::{jetstream::stream::State as StreamState, RequestErrorKind};
use fuel_streams::{client::Client, types::Log, StreamConfig};
use fuel_streams_core::prelude::*;
use futures_util::StreamExt;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum StreamableType {
    Transaction,
    Block,
    Input,
    Output,
    Receipt,
    Utxo,
    Log,
}

pub fn get_streamable_type(name: &str) -> Option<StreamableType> {
    match name {
        Transaction::NAME => Some(StreamableType::Transaction),
        Block::NAME => Some(StreamableType::Block),
        Input::NAME => Some(StreamableType::Input),
        Output::NAME => Some(StreamableType::Output),
        Receipt::NAME => Some(StreamableType::Receipt),
        Utxo::NAME => Some(StreamableType::Utxo),
        Log::NAME => Some(StreamableType::Log),
        _ => None,
    }
}

#[derive(Clone, Debug)]
/// Streams we currently support publishing to.
pub struct Streams {
    pub transactions: Stream<Transaction>,
    pub blocks: Stream<Block>,
    pub inputs: Stream<Input>,
    pub outputs: Stream<Output>,
    pub receipts: Stream<Receipt>,
    pub utxos: Stream<Utxo>,
    pub logs: Stream<Log>,
    pub nats_client: NatsClient,
}

impl Streams {
    pub async fn new(nats_client: &NatsClient) -> Self {
        Self {
            transactions: Stream::<Transaction>::new(nats_client).await,
            blocks: Stream::<Block>::new(nats_client).await,
            inputs: Stream::<Input>::new(nats_client).await,
            outputs: Stream::<Output>::new(nats_client).await,
            receipts: Stream::<Receipt>::new(nats_client).await,
            utxos: Stream::<Utxo>::new(nats_client).await,
            logs: Stream::<Log>::new(nats_client).await,
            nats_client: nats_client.clone(),
        }
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

    pub fn is_within_subject_names(subject_name: &str) -> bool {
        let subject_names = Self::subjects_names();
        subject_names.contains(&subject_name)
    }

    pub fn subjects_wildcards() -> &'static [&'static str] {
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

    pub async fn get_last_published_block(
        &self,
    ) -> anyhow::Result<Option<Block>> {
        Ok(self
            .blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await?)
    }

    pub async fn get_consumers_and_state(
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

    // pub async fn run_dynamic_consumer<S: Streamable>(
    //     name: &str,
    //     client: Client,
    // ) -> anyhow::Result<mpsc::UnboundedReceiver<StreamData<S>>> {
    //     match get_streamable_type(name) {
    //         Some(StreamableType::Transaction) => {
    //             let rx = Streams::run_streamable_consumer::<Transaction>(client).await
    //         }
    //         Some(StreamableType::Block) => {
    //             Streams::run_streamable_consumer::<Block>(client).await
    //         }
    //         Some(StreamableType::Input) => {
    //             Streams::run_streamable_consumer::<Input>(client).await
    //         }
    //         Some(StreamableType::Output) => {
    //             Streams::run_streamable_consumer::<Output>(client).await
    //         }
    //         Some(StreamableType::Receipt) => {
    //             Streams::run_streamable_consumer::<Receipt>(client).await
    //         }
    //         Some(StreamableType::Utxo) => {
    //             Streams::run_streamable_consumer::<Utxo>(client).await
    //         }
    //         Some(StreamableType::Log) => {
    //             Streams::run_streamable_consumer::<Log>(client).await
    //         }
    //         None => Err(anyhow::anyhow!("Unknown streamable type: {}", name)),
    //     }
    // }

    pub async fn run_streamable_consumer<S: Streamable + Send + 'static>(
        client: Client,
    ) -> anyhow::Result<mpsc::UnboundedReceiver<StreamData<S>>> {
        // Create a new stream for blocks
        let stream = fuel_streams::Stream::<S>::new(&client).await;

        // Configure the stream to start from the last published block
        let config = StreamConfig {
            deliver_policy: DeliverPolicy::Last,
        };

        // Subscribe to the block stream with the specified configuration
        let mut sub = stream.subscribe_with_config(config).await?;

        let (tx, rx) = mpsc::unbounded_channel::<StreamData<S>>();

        // Process incoming blocks
        actix_web::rt::spawn(async move {
            while let Some(bytes) = sub.next().await {
                match bytes.as_ref() {
                    Ok(message) => {
                        tracing::info!("Received message: {:?}", message);
                        let decoded_msg =
                            S::decode_raw(message.payload.to_vec()).await;
                        if let Err(e) = tx.send(decoded_msg) {
                            tracing::error!(
                                "Error sending decoded message: {:?}",
                                e
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error receiving message from stream: {:?}",
                            e
                        );
                    }
                }
            }
        });

        Ok(rx)
    }
}
