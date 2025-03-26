use std::sync::Arc;

use fuel_message_broker::NatsMessageBroker;
use fuel_streams_domains::{predicates::Predicate, Subjects};
use fuel_streams_store::{
    db::Db,
    record::{RecordEntity, RecordPacket},
};
use fuel_streams_subject::subject::IntoSubject;
use fuel_web_utils::api_key::ApiKeyRole;

use super::{BoxedStream, Stream, StreamError};
use crate::types::*;

#[derive(Clone, Debug)]
pub struct FuelStreams {
    pub blocks: Stream<Block>,
    pub transactions: Stream<Transaction>,
    pub inputs: Stream<Input>,
    pub outputs: Stream<Output>,
    pub receipts: Stream<Receipt>,
    pub utxos: Stream<Utxo>,
    pub predicates: Stream<Predicate>,
    pub msg_broker: Arc<NatsMessageBroker>,
    pub db: Arc<Db>,
}

impl FuelStreams {
    pub async fn new(broker: &Arc<NatsMessageBroker>, db: &Arc<Db>) -> Self {
        Self {
            blocks: Stream::<Block>::get_or_init(broker, db).await,
            transactions: Stream::<Transaction>::get_or_init(broker, db).await,
            inputs: Stream::<Input>::get_or_init(broker, db).await,
            outputs: Stream::<Output>::get_or_init(broker, db).await,
            receipts: Stream::<Receipt>::get_or_init(broker, db).await,
            utxos: Stream::<Utxo>::get_or_init(broker, db).await,
            predicates: Stream::<Predicate>::get_or_init(broker, db).await,
            msg_broker: Arc::clone(broker),
            db: Arc::clone(db),
        }
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    pub fn broker(&self) -> Arc<NatsMessageBroker> {
        self.msg_broker.clone()
    }

    pub async fn publish_by_entity(
        &self,
        packet: Arc<RecordPacket>,
    ) -> Result<(), StreamError> {
        let subject = (*packet).subject_str();
        let subject_id = (*packet).subject_id();
        let entity = RecordEntity::from_subject_id(&subject_id)?;
        let response = StreamResponse::try_from(&*packet)?;
        let response = Arc::new(response);
        match entity {
            RecordEntity::Block => {
                self.blocks.publish(&subject, &response).await
            }
            RecordEntity::Transaction => {
                self.transactions.publish(&subject, &response).await
            }
            RecordEntity::Input => {
                self.inputs.publish(&subject, &response).await
            }
            RecordEntity::Receipt => {
                self.receipts.publish(&subject, &response).await
            }
            RecordEntity::Output => {
                self.outputs.publish(&subject, &response).await
            }
            RecordEntity::Utxo => self.utxos.publish(&subject, &response).await,
            RecordEntity::Predicate => {
                self.predicates.publish(&subject, &response).await
            }
        }
    }

    pub async fn subscribe_by_entity(
        &self,
        api_key_role: &ApiKeyRole,
        subscription: &Subscription,
    ) -> Result<BoxedStream, StreamError> {
        let subject_payload = subscription.payload.clone();
        let deliver_policy = subscription.deliver_policy;
        let subject: Subjects = subject_payload.clone().try_into()?;
        let subject: Arc<dyn IntoSubject> = subject.into();
        let subject_id = subject_payload.subject.as_str();
        let record_entity = RecordEntity::try_from(subject_id)?;
        let stream = match record_entity {
            RecordEntity::Block => {
                self.blocks
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            RecordEntity::Transaction => {
                self.transactions
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            RecordEntity::Input => {
                self.inputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            RecordEntity::Output => {
                self.outputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            RecordEntity::Receipt => {
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            RecordEntity::Utxo => {
                self.utxos
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            RecordEntity::Predicate => {
                self.predicates
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
        };
        Ok(Box::new(stream))
    }
}
