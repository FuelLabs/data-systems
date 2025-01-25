use std::{str::FromStr, sync::Arc};

use fuel_core_types::blockchain::SealedBlock;
use fuel_message_broker::MessageBroker;
use fuel_streams_core::{
    prelude::SubjectBuildable,
    subjects::TransactionsSubject,
    types::{BlockHeight, FuelCoreLike, Transaction, TransactionStatus},
};
use fuel_streams_domains::{Metadata, MsgPayload};
use fuel_streams_store::{
    db::Db,
    record::{DataEncoder, QueryOptions},
    store::Store,
};
use fuel_web_utils::telemetry::Telemetry;
use futures::StreamExt;

use crate::{error::PublishError, metrics::Metrics};

pub async fn publish_block(
    message_broker: &Arc<dyn MessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    sealed_block: &Arc<SealedBlock>,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let metadata = Metadata::new(fuel_core, sealed_block);
    let fuel_core = Arc::clone(fuel_core);
    let payload = MsgPayload::new(fuel_core, sealed_block, &metadata).await?;
    let encoded = payload.encode().await?;

    message_broker
        .publish_block(payload.message_id(), encoded.clone())
        .await?;

    if let Some(metrics) = telemetry.base_metrics() {
        metrics.update_publisher_success_metrics(
            &payload.subject(),
            encoded.len(),
        );
    }

    tracing::info!("New block submitted: {}", payload.block_height());
    Ok(())
}

pub async fn process_transactions_status_none(
    db: &Db,
    message_broker: &Arc<dyn MessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let block_height = BlockHeight::from(0);
    let db = db.to_owned().arc();
    let store = Store::<Transaction>::new(&db);
    let subject = TransactionsSubject::new()
        .with_tx_status(Some(TransactionStatus::None))
        .dyn_arc();

    let query_opts = QueryOptions::default().with_limit(500);
    let mut historical = store
        .historical_streaming(
            subject.to_owned(),
            Some(block_height),
            Some(query_opts),
        )
        .await;

    while let Some(result) = historical.next().await {
        let (subject, _) = result?;
        let block_height = subject.split(".").nth(1);
        let sealed_block = match block_height {
            Some(height) => Ok(fuel_core
                .get_sealed_block_by_height(BlockHeight::from_str(height)?)),
            None => Err(PublishError::BlockNotFound),
        }?;

        let entity = sealed_block.clone().entity;
        let txs_len = &entity.transactions().len();
        let block_height = entity.header().height();
        let sealed_block = Arc::new(sealed_block);
        tracing::info!(
            "Recovering block #{} with {} transactions not processed",
            block_height,
            txs_len
        );
        publish_block(message_broker, fuel_core, &sealed_block, telemetry)
            .await?;
    }
    todo!()
}
