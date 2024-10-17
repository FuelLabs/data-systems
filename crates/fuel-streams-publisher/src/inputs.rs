use std::sync::Arc;

use fuel_core_types::fuel_tx::{
    input::{
        coin::{CoinPredicate, CoinSigned},
        message::{
            MessageCoinPredicate,
            MessageCoinSigned,
            MessageDataPredicate,
            MessageDataSigned,
        },
    },
    UniqueIdentifier,
};
use fuel_streams_core::{prelude::*, transactions::TransactionExt};

use crate::{
    maybe_include_predicate_and_script_subjects,
    metrics::PublisherMetrics,
    publish_all,
};

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    stream: &Stream<Input>,
    transaction: &Transaction,
    chain_id: &ChainId,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: Option<Bytes32>,
    script_tag: Option<Bytes32>,
) -> anyhow::Result<()> {
    let tx_id = transaction.id(chain_id);

    for (index, input) in transaction.inputs().iter().enumerate() {
        let mut subjects: Vec<(Box<dyn IntoSubject>, &'static str)> =
            match input {
                Input::Contract(contract) => {
                    let contract_id = contract.contract_id;

                    vec![
                        (
                            InputsContractSubject::new()
                                .with_tx_id(Some(tx_id.into()))
                                .with_index(Some(index))
                                .with_contract_id(Some(contract_id.into()))
                                .boxed(),
                            InputsContractSubject::WILDCARD,
                        ),
                        (
                            InputsByIdSubject::new()
                                .with_id_kind(Some(IdentifierKind::ContractID))
                                .with_id_value(Some(contract_id.into()))
                                .boxed(),
                            InputsByIdSubject::WILDCARD,
                        ),
                    ]
                }
                Input::CoinSigned(CoinSigned {
                    owner, asset_id, ..
                })
                | Input::CoinPredicate(CoinPredicate {
                    owner, asset_id, ..
                }) => {
                    vec![
                        (
                            InputsCoinSubject::new()
                                .with_tx_id(Some(tx_id.into()))
                                .with_index(Some(index))
                                .with_owner(Some(owner.into()))
                                .with_asset_id(Some(asset_id.into()))
                                .boxed(),
                            InputsCoinSubject::WILDCARD,
                        ),
                        (
                            InputsByIdSubject::new()
                                .with_id_kind(Some(IdentifierKind::AssetID))
                                .with_id_value(Some((*asset_id).into()))
                                .boxed(),
                            InputsByIdSubject::WILDCARD,
                        ),
                    ]
                }
                Input::MessageCoinSigned(MessageCoinSigned {
                    sender,
                    recipient,
                    ..
                })
                | Input::MessageCoinPredicate(MessageCoinPredicate {
                    sender,
                    recipient,
                    ..
                })
                | Input::MessageDataSigned(MessageDataSigned {
                    sender,
                    recipient,
                    ..
                })
                | Input::MessageDataPredicate(MessageDataPredicate {
                    sender,
                    recipient,
                    ..
                }) => {
                    vec![
                        (
                            InputsMessageSubject::new()
                                .with_tx_id(Some(tx_id.into()))
                                .with_index(Some(index))
                                .with_sender(Some(sender.into()))
                                .with_recipient(Some(recipient.into()))
                                .boxed(),
                            InputsMessageSubject::WILDCARD,
                        ),
                        (
                            InputsByIdSubject::new()
                                .with_id_kind(Some(IdentifierKind::Address))
                                .with_id_value(Some((*sender).into()))
                                .boxed(),
                            InputsByIdSubject::WILDCARD,
                        ),
                        (
                            InputsByIdSubject::new()
                                .with_id_kind(Some(IdentifierKind::Address))
                                .with_id_value(Some((*recipient).into()))
                                .boxed(),
                            InputsByIdSubject::WILDCARD,
                        ),
                    ]
                }
            };

        maybe_include_predicate_and_script_subjects(
            &mut subjects,
            &predicate_tag,
            &script_tag,
        );

        publish_all(stream, subjects, input, metrics, chain_id, block_producer)
            .await;
    }

    Ok(())
}
