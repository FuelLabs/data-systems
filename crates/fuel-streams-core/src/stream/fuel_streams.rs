use std::sync::Arc;

use fuel_streams_store::db::Db;

use super::Stream;
use crate::{nats::*, types::*};

#[derive(Clone, Debug)]
pub struct FuelStreams {
    pub blocks: Stream<Block>,
    pub transactions: Stream<Transaction>,
    pub inputs: Stream<Input>,
    pub outputs: Stream<Output>,
    pub receipts: Stream<Receipt>,
    pub utxos: Stream<Utxo>,
    pub logs: Stream<Log>,
    pub nats_client: Arc<NatsClient>,
    pub db: Arc<Db>,
}

impl FuelStreams {
    pub async fn new(nats_client: &NatsClient, db: &Arc<Db>) -> Self {
        Self {
            blocks: Stream::<Block>::get_or_init(nats_client, db).await,
            transactions: Stream::<Transaction>::get_or_init(nats_client, db)
                .await,
            inputs: Stream::<Input>::get_or_init(nats_client, db).await,
            outputs: Stream::<Output>::get_or_init(nats_client, db).await,
            receipts: Stream::<Receipt>::get_or_init(nats_client, db).await,
            utxos: Stream::<Utxo>::get_or_init(nats_client, db).await,
            logs: Stream::<Log>::get_or_init(nats_client, db).await,
            nats_client: Arc::new(nats_client.clone()),
            db: Arc::clone(&db),
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

    pub fn nats_client(&self) -> Arc<NatsClient> {
        self.nats_client.clone()
    }
}
