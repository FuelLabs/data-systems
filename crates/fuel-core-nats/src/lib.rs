mod nats;

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

pub struct Publisher {
    chain_id: ChainId,
    base_asset_id: AssetId,
    fuel_core_database: CombinedDatabase,
    blocks_subscription: Receiver<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>,
    nats: nats::Nats,
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
        let nats = nats::Nats::connect(nats_url, nats_nkey, &None).await?;

        Ok(Publisher {
            chain_id,
            base_asset_id,
            fuel_core_database,
            blocks_subscription,
            nats,
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
    pub async fn run(mut self, sandbox_id: &Option<String>) -> anyhow::Result<Self> {
        info!(
            "NATS Publisher chain_id={} base_asset_id={} started",
            self.chain_id, self.base_asset_id
        );

        // Check the last block height in the stream
        let stream_height = {
            let config = async_nats::jetstream::consumer::pull::Config {
                deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::Last,
                filter_subject: nats::SubjectName::Blocks.to_subject_string(sandbox_id),
                ..Default::default()
            };
            let consumer = self
                .nats
                .jetstream
                .create_consumer_on_stream(config, self.nats.stream_name.clone())
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

                self.publish_block(&block, &receipts_, sandbox_id).await?;
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
            self.publish_block(&result.sealed_block.entity, &receipts_, sandbox_id)
                .await?;
        }

        Ok(self)
    }

    /// Publish the Block, its Transactions, and the given Receipts into NATS.
    pub async fn publish_block(
        &self,
        block: &Block<Transaction>,
        receipts: &[Receipt],
        sandbox_id: &Option<String>,
    ) -> anyhow::Result<()> {
        let height = u32::from(block.header().consensus().height);

        // Publish the block.
        info!("NATS Publisher: Block#{height}");
        let payload = serde_json::to_string_pretty(block)?;

        let block_subject = nats::Subject::Blocks { height };
        self.nats
            .publish(block_subject, payload.into(), sandbox_id)
            .await?;

        for (index, tx) in block.transactions().iter().enumerate() {
            if let Transaction::Script(s) = tx {
                for i in s.inputs() {
                    // Publish transaction to owners
                    if let Some(owner_id) = i.input_owner().cloned() {
                        let payload = serde_json::to_string_pretty(tx)?;
                        let owners_subject = nats::Subject::Owners { height, owner_id };
                        self.nats
                            .publish(owners_subject, payload.into(), sandbox_id)
                            .await?;
                    }
                    // Publish transaction to assets
                    if let Some(asset_id) = i.asset_id(&self.base_asset_id).cloned() {
                        let payload = serde_json::to_string_pretty(tx)?;
                        let assets_subject = nats::Subject::Assets { height, asset_id };
                        self.nats
                            .publish(assets_subject, payload.into(), sandbox_id)
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
            let payload = serde_json::to_string_pretty(tx)?;
            let transactions_subject = nats::Subject::Transactions {
                height,
                index,
                kind: tx_kind.to_string(),
            };
            self.nats
                .publish(transactions_subject, payload.into(), sandbox_id)
                .await?;
        }

        for receipt in receipts.iter() {
            let receipt_kind = match receipt {
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

            let contract_id = receipt.contract_id().map(|x| x.to_string()).unwrap_or(
                "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            );

            let payload = serde_json::to_string_pretty(receipt)?;
            let receipt1_subject = nats::Subject::Receipts1 {
                height,
                contract_id: contract_id.clone(),
                topic_or_kind: receipt_kind.to_string(),
            };
            self.nats
                .publish(receipt1_subject, payload.into(), sandbox_id)
                .await?;

            // Publish LogData topics, if any.
            if let Receipt::LogData {
                data: Some(data), ..
            } = receipt
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

                    let receipt1_subject = nats::Subject::Receipts1 {
                        height,
                        contract_id,
                        topic_or_kind: topics.to_string(),
                    };
                    self.nats
                        .publish(receipt1_subject, payload.into(), sandbox_id)
                        .await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_nats::jetstream::stream::LastRawMessageErrorKind;

    use fuel_core::{
        combined_database::CombinedDatabase,
        schema::tx::receipt::all_receipts,
    };
    use fuel_core_types::{
        blockchain::SealedBlock,
        fuel_tx::Bytes32,
        services::executor::TransactionExecutionStatus,
    };
    use nats::SubjectName;
    use strum::IntoEnumIterator;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn doesnt_publish_any_message_when_no_block_has_been_mined() {
        let (_, blocks_subscription) =
            broadcast::channel::<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>(1);

        let sandbox_id = nats::random_sandbox_id();
        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            nats: nats::tests::get_nats_connection(&sandbox_id).await,
        };

        let publisher = publisher.run(&Some(sandbox_id)).await.unwrap();

        assert!(publisher.nats.has_no_message().await);
    }

    #[tokio::test]
    async fn publishes_a_block_message_when_a_single_block_has_been_mined() {
        let (blocks_subscriber, blocks_subscription) =
            broadcast::channel::<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>(1);
        let block = Arc::new(ImportResult::default());
        let _ = blocks_subscriber.send(block);

        // manually drop blocks to ensure `blocks_subscription` completes
        let _ = blocks_subscriber.clone();
        drop(blocks_subscriber);

        let sandbox_id = nats::random_sandbox_id();
        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            nats: nats::tests::get_nats_connection(&sandbox_id).await,
        };

        let sandbox_id = &Some(sandbox_id);
        let publisher = publisher.run(sandbox_id).await.unwrap();

        assert!(publisher
            .nats
            .jetstream_messages
            .get_last_raw_message_by_subject(
                &SubjectName::Blocks.to_subject_string(sandbox_id)
            )
            .await
            .is_ok_and(|raw_message| raw_message.sequence == 1));
    }

    #[tokio::test]
    async fn doesnt_publish_any_other_message_for_blocks_with_no_transactions() {
        let (blocks_subscriber, blocks_subscription) =
            broadcast::channel::<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>(1);
        let block = Arc::new(ImportResult::default());
        let _ = blocks_subscriber.send(block);

        // manually drop blocks to ensure `blocks_subscription` completes
        let _ = blocks_subscriber.clone();
        drop(blocks_subscriber);

        let sandbox_id = nats::random_sandbox_id();
        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            nats: nats::tests::get_nats_connection(&sandbox_id).await,
        };
        let sandbox_id = &Some(sandbox_id);
        let publisher = publisher.run(sandbox_id).await.unwrap();

        let non_block_subjects_count = nats::SubjectName::iter().len() - 1;

        let raw_messages_by_all_subjects =
            publisher.nats.get_last_raw_messages_by_all_subjects().await;
        let last_non_block_subjects =
            raw_messages_by_all_subjects.iter().filter(|result| {
                result
                    .as_ref()
                    .is_err_and(|e| e.kind() == LastRawMessageErrorKind::NoMessageFound)
            });

        assert!(non_block_subjects_count == last_non_block_subjects.count());

        assert!(publisher
            .nats
            .jetstream_messages
            .get_last_raw_message_by_subject(
                &SubjectName::Blocks.to_subject_string(sandbox_id)
            )
            .await
            .is_ok_and(|raw_message| raw_message.sequence == 1));
    }

    #[tokio::test]
    async fn publishes_transactions_for_each_published_block() {
        let (blocks_subscriber, blocks_subscription) =
            broadcast::channel::<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>(1);

        let mut block_entity = Block::default();
        *block_entity.transactions_mut() = vec![Transaction::default_test_tx()];

        let block = Arc::new(ImportResult {
            sealed_block: SealedBlock {
                entity: block_entity,
                ..Default::default()
            },
            ..Default::default()
        });
        let _ = blocks_subscriber.send(block);

        // manually drop blocks to ensure `blocks_subscription` completes
        let _ = blocks_subscriber.clone();
        drop(blocks_subscriber);

        let sandbox_id = nats::random_sandbox_id();
        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            nats: nats::tests::get_nats_connection(&sandbox_id).await,
        };

        let sandbox_id = &Some(sandbox_id);
        let publisher = publisher.run(sandbox_id).await.unwrap();

        for subject in [
            SubjectName::Transactions,
            SubjectName::Owners,
            SubjectName::Assets,
        ] {
            assert!(publisher
                .nats
                .jetstream_messages
                .get_last_raw_message_by_subject(&subject.to_subject_string(sandbox_id))
                .await
                .is_ok());
        }
    }

    #[tokio::test]
    async fn publishes_receipt_for_successful_tx_statuses() {
        let (blocks_subscriber, blocks_subscription) =
            broadcast::channel::<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>(1);

        let successful_tx_status = TransactionExecutionStatus {
            id: Bytes32::default(),
            result: TransactionExecutionResult::Success {
                result: None,
                receipts: all_receipts(),
                total_gas: 0,
                total_fee: 0,
            },
        };

        let block = Arc::new(ImportResult {
            tx_status: vec![successful_tx_status],
            ..Default::default()
        });
        let _ = blocks_subscriber.send(block);

        // manually drop blocks to ensure `blocks_subscription` completes
        let _ = blocks_subscriber.clone();
        drop(blocks_subscriber);

        let sandbox_id = nats::random_sandbox_id();
        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            nats: nats::tests::get_nats_connection(&sandbox_id).await,
        };

        let sandbox_id = &Some(sandbox_id);
        let publisher = publisher.run(sandbox_id).await.unwrap();

        assert!(publisher
            .nats
            .jetstream_messages
            .get_last_raw_message_by_subject(
                &SubjectName::Receipts1.to_subject_string(sandbox_id)
            )
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn publishes_receipt_for_failed_tx_statuses() {
        let (blocks_subscriber, blocks_subscription) =
            broadcast::channel::<Arc<dyn Deref<Target = ImportResult> + Send + Sync>>(1);

        let successful_tx_status = TransactionExecutionStatus {
            id: Bytes32::default(),
            result: TransactionExecutionResult::Failed {
                result: None,
                receipts: all_receipts(),
                total_gas: 0,
                total_fee: 0,
            },
        };

        let block = Arc::new(ImportResult {
            tx_status: vec![successful_tx_status],
            ..Default::default()
        });
        let _ = blocks_subscriber.send(block);

        // manually drop blocks to ensure `blocks_subscription` completes
        let _ = blocks_subscriber.clone();
        drop(blocks_subscriber);

        let sandbox_id = nats::random_sandbox_id();
        let publisher = Publisher {
            base_asset_id: AssetId::default(),
            chain_id: ChainId::default(),
            fuel_core_database: CombinedDatabase::default(),
            blocks_subscription,
            nats: nats::tests::get_nats_connection(&sandbox_id).await,
        };

        let sandbox_id = &Some(sandbox_id);
        let publisher = publisher.run(sandbox_id).await.unwrap();

        assert!(publisher
            .nats
            .jetstream_messages
            .get_last_raw_message_by_subject(
                &SubjectName::Receipts1.to_subject_string(sandbox_id)
            )
            .await
            .is_ok());
    }
}
