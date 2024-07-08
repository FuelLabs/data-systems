use anyhow::Context;

use futures_util::stream::TryStreamExt;
use std::{
    ops::Deref,
    sync::Arc,
};
use tokio::sync::broadcast::Receiver;
use tracing::{
    info,
    warn,
};

use fuel_core::combined_database::CombinedDatabase;
use fuel_core_types::{
    blockchain::block::Block,
    fuel_tx::{
        field::Inputs,
        Receipt,
        Transaction,
        UniqueIdentifier,
    },
    fuel_types::{
        AssetId,
        ChainId,
    },
    services::{
        block_importer::ImportResult,
        executor::TransactionExecutionResult,
    },
};

const NUM_TOPICS: usize = 3;

struct NatsConnection {
    jetstream: async_nats::jetstream::Context,
    #[allow(dead_code)]
    /// Messages published to jetstream
    jetstream_messages: async_nats::jetstream::stream::Stream,
    /// Max publishing payload in connected NATS server
    max_payload_size: usize,
}

pub struct Publisher {
    chain_id: ChainId,
    base_asset_id: AssetId,
    fuel_core_database: CombinedDatabase,
    blocks_subscription: Receiver<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>,
    nats: NatsConnection,
}

impl Publisher {
    pub async fn new(
        nats_url: &str,
        nats_nkey: Option<String>,
        chain_id: ChainId,
        base_asset_id: AssetId,
        fuel_core_database: CombinedDatabase,
        blocks_subscription: Receiver<
            Arc<dyn Deref<Target = ImportResult> + Send + Sync>,
        >,
    ) -> anyhow::Result<Self> {
        Ok(Publisher {
            chain_id,
            base_asset_id,
            fuel_core_database,
            blocks_subscription,
            nats: Self::connect_to_nats(nats_url, nats_nkey).await?,
        })
    }

    async fn connect_to_nats(
        nats_url: &str,
        nats_nkey: Option<String>,
    ) -> anyhow::Result<NatsConnection> {
        let client = match nats_nkey {
            Some(nkey) => async_nats::connect_with_options(
                nats_url,
                async_nats::ConnectOptions::with_nkey(nkey),
            )
            .await
            .context(format!("Connecting to {nats_url}"))?,
            None => async_nats::connect(nats_url)
                .await
                .context(format!("Connecting to {nats_url}"))?,
        };

        let max_payload_size = client.server_info().max_payload;
        info!("NATS Publisher: max_payload_size={max_payload_size}");

        // Create a JetStream context
        let jetstream = async_nats::jetstream::new(client);

        let jetstream_messages = jetstream
            .get_or_create_stream(async_nats::jetstream::stream::Config {
                name: "fuel".to_string(),
                subjects: vec![
                    // blocks.{height}
                    "blocks.*".to_string(),
                    // receipts.{height}.{contract_id}.{kind}
                    // or
                    // receipts.{height}.{contract_id}.{topic_1}
                    "receipts.*.*.*".to_string(),
                    // receipts.{height}.{contract_id}.{topic_1}.{topic_2}
                    "receipts.*.*.*.*".to_string(),
                    // receipts.{height}.{contract_id}.{topic_1}.{topic_2}.{topic_3}
                    "receipts.*.*.*.*.*".to_string(),
                    // transactions.{height}.{index}.{kind}
                    "transactions.*.*.*".to_string(),
                    // owners.{height}.{owner_id}
                    "owners.*.*".to_string(),
                    // assets.{height}.{asset_id}
                    "assets.*.*".to_string(),
                ],
                storage: async_nats::jetstream::stream::StorageType::File,
                ..Default::default()
            })
            .await?;

        Ok(NatsConnection {
            jetstream,
            jetstream_messages,
            max_payload_size,
        })
    }

    /// Publish messages from node(`fuel-core`) to NATS stream
    ///   receipts.{height}.{contract_id}.{kind}                         e.g. receipts.9000.*.return
    ///   receipts.{height}.{contract_id}.{topic_1}                      e.g. receipts.*.my_custom_topic
    ///   receipts.{height}.{contract_id}.{topic_1}.{topic_2}            e.g. receipts.*.counter.inrc
    ///   receipts.{height}.{contract_id}.{topic_1}.{topic_2}.{topic_3}
    ///   transactions.{height}.{index}.{kind}                           e.g. transactions.1.1.mint
    ///   blocks.{height}                                                e.g. blocks.1
    ///   owners.{height}.{owner_id}                                     e.g. owners.*.0xab..cd
    ///   assets.{height}.{asset_id}                                     e.g. assets.*.0xab..cd
    pub async fn run(mut self) -> anyhow::Result<()> {
        info!(
            "NATS Publisher chain_id={} base_asset_id={} started",
            self.chain_id, self.base_asset_id
        );

        // Check the last block height in the stream
        let stream_height = {
            let config = async_nats::jetstream::consumer::pull::Config {
                deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::Last,
                filter_subject: "blocks.*".to_string(),
                ..Default::default()
            };
            let consumer = self
                .nats
                .jetstream
                .create_consumer_on_stream(config, "fuel")
                .await?;
            let mut batch = consumer.fetch().max_messages(1).messages().await?;

            if let Ok(Some(message)) = batch.try_next().await {
                let block_height: u32 =
                    message.subject.strip_prefix("blocks.").unwrap().parse()?;
                block_height
            } else {
                0
            }
        };

        // Fast-forward the stream using the local Fuel node database
        if let Some(chain_height) = self.fuel_core_database.on_chain().latest_height()? {
            let chain_height: u32 = chain_height.into();
            if chain_height > stream_height + 1 {
                warn!("NATS Publisher: missing blocks: stream block height={stream_height}, chain block height={chain_height}");
            }

            for height in stream_height + 1..=chain_height {
                let block: Block = self
                    .fuel_core_database
                    .on_chain()
                    .get_sealed_block_by_height(&height.into())?
                    .unwrap_or_else(|| {
                        panic!("NATS Publisher: no block at height {height}")
                    })
                    .entity;

                use fuel_core_types::services::txpool::TransactionStatus;
                let mut receipts_: Vec<Receipt> = vec![];
                let chain_id = self.chain_id;
                for t in block.transactions().iter() {
                    let status: Option<TransactionStatus> = self
                        .fuel_core_database
                        .off_chain()
                        .get_tx_status(&t.id(&chain_id))?;
                    match status {
                        Some(TransactionStatus::Failed { mut receipts, .. }) => {
                            receipts_.append(&mut receipts);
                        }
                        Some(TransactionStatus::Success { mut receipts, .. }) => {
                            receipts_.append(&mut receipts);
                        }
                        Some(TransactionStatus::Submitted { .. }) => (),
                        Some(TransactionStatus::SqueezedOut { .. }) => (),
                        // TODO: check that we'd get the same result from the block importer subscription
                        None => (),
                    }
                }

                let height: u32 = **block.header().height();

                info!(
                "NATS Publisher: publishing block {height} / {chain_height} with {} receipts",
                receipts_.len()
            );

                self.publish_block(&block, &receipts_).await?;
            }
        }

        // Continue publishing blocks from the block importer subscription
        while let Ok(result) = self.blocks_subscription.recv().await {
            let mut receipts_: Vec<Receipt> = vec![];
            for t in result.tx_status.iter() {
                let mut receipts = match &t.result {
                    TransactionExecutionResult::Success { receipts, .. } => {
                        receipts.clone()
                    }
                    TransactionExecutionResult::Failed { receipts, .. } => {
                        receipts.clone()
                    }
                };
                receipts_.append(&mut receipts);
            }
            let result = &**result;
            self.publish_block(&result.sealed_block.entity, &receipts_)
                .await?;
        }

        Ok(())
    }

    /// Publish the Block, its Transactions, and the given Receipts into NATS.
    pub async fn publish_block(
        &self,
        block: &Block<Transaction>,
        receipts: &[Receipt],
    ) -> anyhow::Result<()> {
        let height = u32::from(block.header().consensus().height);

        // Publish the block.
        info!("NATS Publisher: Block#{height}");
        let payload = serde_json::to_string_pretty(block)?;
        self.publish(format!("blocks.{height}"), payload.into())
            .await?;

        for (index, tx) in block.transactions().iter().enumerate() {
            if let Transaction::Script(s) = tx {
                for i in s.inputs() {
                    // Publish transaction to owners
                    if let Some(owner_id) = i.input_owner() {
                        let payload = serde_json::to_string_pretty(tx)?;
                        self.publish(
                            format!("owners.{height}.{owner_id}"),
                            payload.into(),
                        )
                        .await?;
                    }
                    // Publish transaction to assets
                    if let Some(asset_id) = i.asset_id(&self.base_asset_id) {
                        let payload = serde_json::to_string_pretty(tx)?;
                        self.publish(
                            format!("assets.{height}.{asset_id}"),
                            payload.into(),
                        )
                        .await?;
                    }
                }
            }

            let tx_kind = match tx {
                Transaction::Create(_) => "create",
                Transaction::Mint(_) => "mint",
                Transaction::Script(_) => "script",
                Transaction::Upload(_) => "upload",
                Transaction::Upgrade(_) => "upgrade",
            };

            // Publish the transaction.
            info!("NATS Publisher: Transaction#{height}.{index}.{tx_kind}");
            let payload = serde_json::to_string_pretty(tx)?;
            self.publish(
                format!("transactions.{height}.{index}.{tx_kind}"),
                payload.into(),
            )
            .await?;
        }

        for r in receipts.iter() {
            let receipt_kind = match r {
                Receipt::Call { .. } => "call",
                Receipt::Return { .. } => "return",
                Receipt::ReturnData { .. } => "return_data",
                Receipt::Panic { .. } => "panic",
                Receipt::Revert { .. } => "revert",
                Receipt::Log { .. } => "log",
                Receipt::LogData { .. } => "log_data",
                Receipt::Transfer { .. } => "transfer",
                Receipt::TransferOut { .. } => "transfer_out",
                Receipt::ScriptResult { .. } => "script_result",
                Receipt::MessageOut { .. } => "message_out",
                Receipt::Mint { .. } => "mint",
                Receipt::Burn { .. } => "burn",
            };

            let contract_id = r.contract_id().map(|x| x.to_string()).unwrap_or(
                "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            );

            // Publish the receipt.
            info!("NATS Publisher: Receipt#{height}.{contract_id}.{receipt_kind}");
            let payload = serde_json::to_string_pretty(r)?;
            let subject = format!("receipts.{height}.{contract_id}.{receipt_kind}");
            self.publish(subject, payload.into()).await?;

            // Publish LogData topics, if any.
            if let Receipt::LogData {
                data: Some(data), ..
            } = r
            {
                info!("NATS Publisher: Log Data Length: {}", data.len());
                // 0x0000000012345678
                let header = vec![0, 0, 0, 0, 18, 52, 86, 120];
                if data.starts_with(&header) {
                    let data = &data[header.len()..];
                    let mut topics = vec![];
                    for i in 0..NUM_TOPICS {
                        let topic_bytes: Vec<u8> = data[32 * i..32 * (i + 1)]
                            .iter()
                            .cloned()
                            .take_while(|x| *x > 0)
                            .collect();
                        let topic = String::from_utf8_lossy(&topic_bytes).into_owned();
                        if !topic.is_empty() {
                            topics.push(topic);
                        }
                    }
                    let topics = topics.join(".");
                    // TODO: JSON payload to match other topics? {"data": payload}
                    let payload = data[NUM_TOPICS * 32..].to_owned();

                    // Publish receipt topics
                    info!("NATS Publisher: Receipt#{height}.{contract_id}.{topics}");
                    self.publish(
                        format!("receipts.{height}.{contract_id}.{topics}"),
                        payload.into(),
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }

    /// A wrapper around JetStream::publish() that also checks that the payload size does not exceed NATS server's max_payload_size.
    async fn publish(
        &self,
        subject: String,
        payload: bytes::Bytes,
    ) -> anyhow::Result<()> {
        // Check message size
        let payload_size = payload.len();
        if payload_size > self.nats.max_payload_size {
            anyhow::bail!(
                "{subject} payload size={payload_size} exceeds max_payload_size={}",
                self.nats.max_payload_size
            )
        }
        // Publish
        let ack_future = self.nats.jetstream.publish(subject, payload).await?;
        // Wait for an ACK
        ack_future.await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_nats::jetstream::stream::LastRawMessageErrorKind;
    use fuel_core::combined_database::CombinedDatabase;
    use tokio::sync::broadcast;

    const NATS_URL: &'static str = "nats://localhost:4222";

    #[tokio::test]
    async fn connects_to_nats() {
        let nats_nkey = None;
        let chain_id = ChainId::default();
        let base_asset_id = AssetId::default();
        let fuel_core_database = CombinedDatabase::default();
        let (_, blocks_subscription) =
            broadcast::channel::<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>(1);

        let publisher = Publisher::new(
            NATS_URL,
            nats_nkey,
            chain_id,
            base_asset_id,
            fuel_core_database,
            blocks_subscription,
        )
        .await
        .expect(&format!("Ensure NATS server is running at {NATS_URL}"));

        assert!(publisher
            .nats
            .jetstream_messages
            .get_last_raw_message_by_subject("*")
            .await
            .is_err_and(|err| err.kind() == LastRawMessageErrorKind::NoMessageFound));

        assert_eq!(publisher.chain_id, chain_id);
        assert_eq!(publisher.base_asset_id, base_asset_id);
    }
}
