use fuel_core_types::fuel_tx::{
    field::Inputs,
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
    types::{Bytes32, ChainId, IdentifierKind, Input, Transaction},
    Stream,
};

macro_rules! get_inputs {
    ($transaction:expr, $($variant:ident),+) => {
        match $transaction {
            Transaction::Mint(_) => vec![],
            $(Transaction::$variant(tx) => tx.inputs().to_vec(),)+
        }
    };
}

fn inputs_from_transaction(transaction: &Transaction) -> Vec<Input> {
    get_inputs!(transaction, Script, Blob, Create, Upload, Upgrade)
}

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
    chain_id: &ChainId,
    transactions: &[Transaction],
) -> anyhow::Result<()> {
    let subjects: Vec<InputSubject> = transactions
        .iter()
        .flat_map(|transaction| {
            let tx_id = transaction.id(chain_id);
            let inputs = inputs_from_transaction(transaction);
            inputs
                .iter()
                .enumerate()
                .map(|(index, input)| match input {
                    Input::Contract(contract) => {
                        let (by_id, subject) = contract_subject(
                            contract.contract_id,
                            tx_id.into(),
                            index,
                        );
                        InputSubject::Contract(by_id, subject, input.to_owned())
                    }
                    Input::CoinSigned(coin) => {
                        let (by_id, subject) =
                            coin_subject(coin, tx_id.into(), index);
                        InputSubject::Coin(by_id, subject, input.to_owned())
                    }
                    Input::CoinPredicate(coin) => {
                        let (by_id, subject) =
                            coin_subject(coin, tx_id.into(), index);
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
                .collect::<Vec<InputSubject>>()
        })
        .collect();

    for subject_item in subjects {
        match subject_item {
            InputSubject::Contract(by_id, subject, payload) => {
                stream.publish(&subject, &payload).await?;
                stream.publish(&by_id, &payload).await?;
            }
            InputSubject::Coin(by_id, subject, payload) => {
                stream.publish(&subject, &payload).await?;
                stream.publish(&by_id, &payload).await?;
            }
            InputSubject::Message(
                by_id_sender,
                by_id_recipient,
                subject,
                payload,
            ) => {
                stream.publish(&subject, &payload).await?;
                stream.publish(&by_id_sender, &payload).await?;
                stream.publish(&by_id_recipient, &payload).await?;
            }
        };
    }

    Ok(())
}
