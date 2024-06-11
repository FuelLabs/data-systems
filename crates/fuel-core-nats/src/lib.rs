use anyhow::{
    bail,
    Context,
};
use futures_util::stream::TryStreamExt;
use tracing::info;

use fuel_core_types::{
    fuel_tx::field::Inputs,
    services::{
        block_importer::ImportResult,
        executor::TransactionExecutionResult,
    },
};

const NUM_TOPICS: usize = 3;

/// Connect to a NATS server and publish messages
///   receipts.{height}.{contract_id}.{kind}                         e.g. receipts.9000.*.return
///   receipts.{height}.{contract_id}.{topic_1}                      e.g. receipts.*.my_custom_topic
///   receipts.{height}.{contract_id}.{topic_1}.{topic_2}            e.g. receipts.*.counter.inrc
///   receipts.{height}.{contract_id}.{topic_1}.{topic_2}.{topic_3}
///   transactions.{height}.{index}.{kind}                           e.g. transactions.1.1.mint
///   blocks.{height}                                                e.g. blocks.1
///   owners.{height}.{owner_id}                                     e.g. owners.*.0xab..cd
///   assets.{height}.{asset_id}                                     e.g. assets.*.0xab..cd
pub async fn nats_publisher(
    mut subscription: tokio::sync::broadcast::Receiver<
        std::sync::Arc<dyn std::ops::Deref<Target = ImportResult> + Send + Sync>,
    >,
    nats_url: String,
) -> anyhow::Result<()> {
    // Connect to the NATS server
    let client = async_nats::connect(&nats_url)
        .await
        .context(format!("Connecting to {nats_url}"))?;
    // Create a JetStream context
    let jetstream = async_nats::jetstream::new(client);
    // Create a JetStream stream (if it doesn't exist)
    let _stream = jetstream
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
                "assets.*.*.".to_string(),
            ],
            storage: async_nats::jetstream::stream::StorageType::File,
            ..Default::default()
        })
        .await?;

    // Check the last block height in the stream
    let last_block_height = {
        let config = async_nats::jetstream::consumer::pull::Config {
            deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::Last,
            filter_subject: "blocks.*".to_string(),
            ..Default::default()
        };
        let consumer = jetstream.create_consumer_on_stream(config, "fuel").await?;
        let mut batch = consumer.fetch().max_messages(1).messages().await?;

        if let Ok(Some(message)) = batch.try_next().await {
            let block_height: u32 =
                message.subject.strip_prefix("blocks.").unwrap().parse()?;
            block_height
        } else {
            0
        }
    };

    info!("NATS Publisher started");

    while let Ok(result) = subscription.recv().await {
        let result = &**result;
        let height = u32::from(result.sealed_block.entity.header().consensus().height);
        if height != last_block_height + 1 {
            bail!("NATS Publisher: missing blocks: stream block height={last_block_height}, chain block height={height}");
        }
        let block = &result.sealed_block.entity;

        // Publish the block.
        info!("NATS Publisher: Block#{height}");
        let payload = serde_json::to_string_pretty(block)?;
        jetstream
            .publish(format!("blocks.{height}"), payload.into())
            .await?;

        use fuel_core_types::fuel_tx::Transaction;
        for (index, tx) in block.transactions().iter().enumerate() {
            if let Transaction::Script(s) = tx {
                for i in s.inputs() {
                    if let Some(owner_id) = i.input_owner() {
                        let payload = serde_json::to_string_pretty(tx)?;
                        jetstream
                            .publish(
                                format!("owners.{height}.{owner_id}"),
                                payload.into(),
                            )
                            .await?;
                    }
                    use fuel_core_types::fuel_tx::AssetId;
                    // TODO: from chain config?
                    let base_asset_id = AssetId::zeroed();
                    if let Some(asset_id) = i.asset_id(&base_asset_id) {
                        let payload = serde_json::to_string_pretty(tx)?;
                        jetstream
                            .publish(
                                format!("assets.{height}.{asset_id}"),
                                payload.into(),
                            )
                            .await?;
                    }
                }
            };

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
            jetstream
                .publish(
                    format!("transactions.{height}.{index}.{tx_kind}"),
                    payload.into(),
                )
                .await?;
        }

        for t in result.tx_status.iter() {
            let receipts = match &t.result {
                TransactionExecutionResult::Success { receipts, .. } => receipts,
                TransactionExecutionResult::Failed { receipts, .. } => receipts,
            };

            use fuel_core_types::fuel_tx::Receipt;
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
                jetstream.publish(subject, payload.into()).await?;

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
                            let topic =
                                String::from_utf8_lossy(&topic_bytes).into_owned();
                            if !topic.is_empty() {
                                topics.push(topic);
                            }
                        }
                        let topics = topics.join(".");
                        // TODO: JSON payload to match other topics? {"data": payload}
                        let payload = data[NUM_TOPICS * 32..].to_owned();

                        // Publish
                        info!("NATS Publisher: Receipt#{height}.{contract_id}.{topics}");
                        jetstream
                            .publish(
                                format!("receipts.{height}.{contract_id}.{topics}"),
                                payload.into(),
                            )
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}
