use std::sync::Arc;

use fuel_core_types::fuel_tx::{
    input::{
        coin::{Coin, CoinSpecification},
        message::{Message, MessageSpecification},
    },
    UniqueIdentifier,
};
use fuel_streams_core::{
    inputs::{
        InputsByIdSubject,
        InputsCoinSubject,
        InputsContractSubject,
        InputsMessageSubject,
    },
    prelude::*,
    transactions::WithTxInputs,
    types::{Bytes32, IdentifierKind, Input, Transaction},
    Stream,
};

use crate::{
    build_subject_name,
    metrics::PublisherMetrics,
    publish_with_metrics,
};

fn coin_subject<T: CoinSpecification>(
    coin: &Coin<T>,
    tx_id: Bytes32,
    index: usize,
) -> (InputsByIdSubject, InputsCoinSubject) {
    let owner = coin.owner;
    let asset_id = coin.asset_id;
    let subject = InputsCoinSubject::new()
        .with_tx_id(Some(tx_id))
        .with_index(Some(index))
        .with_owner(Some(owner.into()))
        .with_asset_id(Some(asset_id.into()));

    let by_id = InputsByIdSubject::new()
        .with_id_kind(Some(IdentifierKind::AssetID))
        .with_id_value(Some(asset_id.into()));

    (by_id, subject)
}

fn message_subject<T: MessageSpecification>(
    message: &Message<T>,
    tx_id: Bytes32,
    index: usize,
) -> (InputsByIdSubject, InputsByIdSubject, InputsMessageSubject) {
    let sender = message.sender;
    let recipient = message.recipient;
    let subject = InputsMessageSubject::new()
        .with_tx_id(Some(tx_id))
        .with_index(Some(index))
        .with_sender(Some(sender.into()))
        .with_recipient(Some(recipient.into()));

    let by_id_sender = InputsByIdSubject::new()
        .with_id_kind(Some(IdentifierKind::Address))
        .with_id_value(Some(message.sender.into()));

    let by_id_recipient = InputsByIdSubject::new()
        .with_id_kind(Some(IdentifierKind::Address))
        .with_id_value(Some(message.recipient.into()));

    (by_id_sender, by_id_recipient, subject)
}

fn contract_subject(
    contract_id: fuel_core_types::fuel_tx::ContractId,
    tx_id: Bytes32,
    index: usize,
) -> (InputsByIdSubject, InputsContractSubject) {
    let subject = InputsContractSubject::new()
        .with_tx_id(Some(tx_id))
        .with_index(Some(index))
        .with_contract_id(Some(contract_id.into()));

    let by_id = InputsByIdSubject::new()
        .with_id_kind(Some(IdentifierKind::ContractID))
        .with_id_value(Some(contract_id.into()));

    (by_id, subject)
}

#[derive(Debug, Clone)]
enum InputSubject {
    Contract(InputsByIdSubject, InputsContractSubject, Input),
    Coin(InputsByIdSubject, InputsCoinSubject, Input),
    Message(
        InputsByIdSubject, // by sender
        InputsByIdSubject, // by recipient
        InputsMessageSubject,
        Input,
    ),
}

pub async fn publish(
    stream: &Stream<Input>,
    transaction: &Transaction,
    chain_id: &ChainId,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: Option<Bytes32>,
) -> anyhow::Result<()> {
    let tx_id = transaction.id(chain_id);

    let subjects = transaction
        .inputs()
        .iter()
        .enumerate()
        .map(|(index, input)| match input {
            Input::Contract(contract) => {
                let (by_id, subject) =
                    contract_subject(contract.contract_id, tx_id.into(), index);
                InputSubject::Contract(by_id, subject, input.to_owned())
            }
            Input::CoinSigned(coin) => {
                let (by_id, subject) = coin_subject(coin, tx_id.into(), index);
                InputSubject::Coin(by_id, subject, input.to_owned())
            }
            Input::CoinPredicate(coin) => {
                let (by_id, subject) = coin_subject(coin, tx_id.into(), index);
                InputSubject::Coin(by_id, subject, input.to_owned())
            }
            Input::MessageCoinSigned(message) => {
                let (by_id_sender, by_id_recipient, subject) =
                    message_subject(message, tx_id.into(), index);
                InputSubject::Message(
                    by_id_sender,
                    by_id_recipient,
                    subject,
                    input.to_owned(),
                )
            }
            Input::MessageCoinPredicate(message) => {
                let (by_id_sender, by_id_recipient, subject) =
                    message_subject(message, tx_id.into(), index);
                InputSubject::Message(
                    by_id_sender,
                    by_id_recipient,
                    subject,
                    input.to_owned(),
                )
            }
            Input::MessageDataSigned(message) => {
                let (by_id_sender, by_id_recipient, subject) =
                    message_subject(message, tx_id.into(), index);
                InputSubject::Message(
                    by_id_sender,
                    by_id_recipient,
                    subject,
                    input.to_owned(),
                )
            }
            Input::MessageDataPredicate(message) => {
                let (by_id_sender, by_id_recipient, subject) =
                    message_subject(message, tx_id.into(), index);
                InputSubject::Message(
                    by_id_sender,
                    by_id_recipient,
                    subject,
                    input.to_owned(),
                )
            }
        })
        .collect::<Vec<InputSubject>>();

    for subject_item in subjects {
        match subject_item {
            InputSubject::Contract(by_id_subject, subject, payload) => {
                publish_with_metrics!(
                    stream.publish_raw(
                        &build_subject_name(&predicate_tag, &subject),
                        &payload
                    ),
                    metrics,
                    chain_id,
                    block_producer,
                    InputsContractSubject::WILDCARD
                );
                publish_with_metrics!(
                    stream.publish_raw(
                        &build_subject_name(&predicate_tag, &by_id_subject),
                        &payload
                    ),
                    metrics,
                    chain_id,
                    block_producer,
                    InputsByIdSubject::WILDCARD
                );
            }
            InputSubject::Coin(by_id_subject, subject, payload) => {
                publish_with_metrics!(
                    stream.publish_raw(
                        &build_subject_name(&predicate_tag, &subject),
                        &payload
                    ),
                    metrics,
                    chain_id,
                    block_producer,
                    InputsCoinSubject::WILDCARD
                );
                publish_with_metrics!(
                    stream.publish_raw(
                        &build_subject_name(&predicate_tag, &by_id_subject),
                        &payload
                    ),
                    metrics,
                    chain_id,
                    block_producer,
                    InputsByIdSubject::WILDCARD
                );
            }
            InputSubject::Message(
                by_id_sender_subject,
                by_id_recipient_subject,
                subject,
                payload,
            ) => {
                publish_with_metrics!(
                    stream.publish_raw(
                        &build_subject_name(&predicate_tag, &subject),
                        &payload
                    ),
                    metrics,
                    chain_id,
                    block_producer,
                    InputsMessageSubject::WILDCARD
                );
                publish_with_metrics!(
                    stream.publish_raw(
                        &build_subject_name(
                            &predicate_tag,
                            &by_id_sender_subject
                        ),
                        &payload
                    ),
                    metrics,
                    chain_id,
                    block_producer,
                    InputsByIdSubject::WILDCARD
                );
                publish_with_metrics!(
                    stream.publish_raw(
                        &build_subject_name(
                            &predicate_tag,
                            &by_id_recipient_subject
                        ),
                        &payload
                    ),
                    metrics,
                    chain_id,
                    block_producer,
                    InputsByIdSubject::WILDCARD
                );
            }
        };
    }

    Ok(())
}
