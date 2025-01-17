use std::sync::Arc;

use fuel_message_broker::MessageBroker;
use fuel_streams_store::db::Db;

use super::Stream;
use crate::types::*;

#[derive(Clone, Debug)]
pub struct FuelStreams {
    pub blocks: Stream<Block>,
    pub transactions: Stream<Transaction>,
    pub inputs: Stream<Input>,
    pub outputs: Stream<Output>,
    pub receipts: Stream<Receipt>,
    pub utxos: Stream<Utxo>,
    pub msg_broker: Arc<dyn MessageBroker>,
    pub db: Arc<Db>,
}

impl FuelStreams {
    pub async fn new(broker: &Arc<dyn MessageBroker>, db: &Arc<Db>) -> Self {
        Self {
            blocks: Stream::<Block>::get_or_init(broker, db).await,
            transactions: Stream::<Transaction>::get_or_init(broker, db).await,
            inputs: Stream::<Input>::get_or_init(broker, db).await,
            outputs: Stream::<Output>::get_or_init(broker, db).await,
            receipts: Stream::<Receipt>::get_or_init(broker, db).await,
            utxos: Stream::<Utxo>::get_or_init(broker, db).await,
            msg_broker: Arc::clone(broker),
            db: Arc::clone(db),
        }
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    pub fn broker(&self) -> Arc<dyn MessageBroker> {
        self.msg_broker.clone()
    }
}
