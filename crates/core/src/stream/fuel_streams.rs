use std::sync::Arc;

use fuel_message_broker::NatsMessageBroker;
use fuel_streams_domains::{
    infra::{
        db::Db,
        record::{RecordEntity, RecordPacket},
    },
    predicates::Predicate,
    Subjects,
};
use fuel_web_utils::api_key::ApiKeyRole;

use super::{BoxedStream, Stream, StreamError};
use crate::{subjects::*, types::*};

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

    pub async fn subscribe_by_subject(
        &self,
        api_key_role: &ApiKeyRole,
        subscription: &Subscription,
    ) -> Result<BoxedStream, StreamError> {
        let subject_payload = subscription.payload.clone();
        let deliver_policy = subscription.deliver_policy;
        let subject: Subjects = subject_payload.try_into()?;
        let stream = match subject {
            Subjects::Block(blocks_subject) => {
                let subject = Arc::new(blocks_subject);
                self.blocks
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::Inputs(inputs_subject) => {
                let subject = Arc::new(inputs_subject);
                self.inputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::InputsCoin(inputs_coin_subject) => {
                let subject = Arc::new(inputs_coin_subject);
                self.inputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::InputsContract(inputs_contract_subject) => {
                let subject = Arc::new(inputs_contract_subject);
                self.inputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::InputsMessage(inputs_message_subject) => {
                let subject = Arc::new(inputs_message_subject);
                self.inputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::Outputs(outputs_subject) => {
                let subject = Arc::new(outputs_subject);
                self.outputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::OutputsCoin(outputs_coin_subject) => {
                let subject = Arc::new(outputs_coin_subject);
                self.outputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::OutputsContract(outputs_contract_subject) => {
                let subject = Arc::new(outputs_contract_subject);
                self.outputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::OutputsChange(outputs_change_subject) => {
                let subject = Arc::new(outputs_change_subject);
                self.outputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::OutputsVariable(outputs_variable_subject) => {
                let subject = Arc::new(outputs_variable_subject);
                self.outputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::OutputsContractCreated(
                outputs_contract_created_subject,
            ) => {
                let subject = Arc::new(outputs_contract_created_subject);
                self.outputs
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::Predicates(predicates_subject) => {
                let subject = Arc::new(predicates_subject);
                self.predicates
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::Receipts(receipts_subject) => {
                let subject = Arc::new(receipts_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsCall(receipts_call_subject) => {
                let subject = Arc::new(receipts_call_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsReturn(receipts_return_subject) => {
                let subject = Arc::new(receipts_return_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsReturnData(receipts_return_data_subject) => {
                let subject = Arc::new(receipts_return_data_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsPanic(receipts_panic_subject) => {
                let subject = Arc::new(receipts_panic_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsRevert(receipts_revert_subject) => {
                let subject = Arc::new(receipts_revert_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsLog(receipts_log_subject) => {
                let subject = Arc::new(receipts_log_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsLogData(receipts_log_data_subject) => {
                let subject = Arc::new(receipts_log_data_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsTransfer(receipts_transfer_subject) => {
                let subject = Arc::new(receipts_transfer_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsTransferOut(receipts_transfer_out_subject) => {
                let subject = Arc::new(receipts_transfer_out_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsScriptResult(receipts_script_result_subject) => {
                let subject = Arc::new(receipts_script_result_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsMessageOut(receipts_message_out_subject) => {
                let subject = Arc::new(receipts_message_out_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsMint(receipts_mint_subject) => {
                let subject = Arc::new(receipts_mint_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::ReceiptsBurn(receipts_burn_subject) => {
                let subject = Arc::new(receipts_burn_subject);
                self.receipts
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::Transactions(transactions_subject) => {
                let subject = Arc::new(transactions_subject);
                self.transactions
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
            Subjects::Utxos(utxos_subject) => {
                let subject = Arc::new(utxos_subject);
                self.utxos
                    .subscribe_dynamic(subject, deliver_policy, api_key_role)
                    .await
            }
        };

        Ok(Box::new(stream))
    }
}
