use std::sync::Arc;

use anyhow::Result;
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
use futures::stream::{StreamExt, TryStreamExt};
use rayon::prelude::*;

use crate::{
    identifiers::{Identifier, IdsExtractable, SubjectPayloadBuilder},
    metrics::PublisherMetrics,
    PublishError,
    PublishPayload,
    SubjectPayload,
    CONCURRENCY_LIMIT,
};

pub async fn publish_tasks(
    stream: &Stream<Input>,
    transactions: &[Transaction],
    chain_id: &ChainId,
    block_producer: &Address,
    metrics: &Arc<PublisherMetrics>,
) -> Result<(), PublishError> {
    futures::stream::iter(
        transactions
            .iter()
            .flat_map(|tx| create_publish_payloads(tx, chain_id)),
    )
    .map(Ok)
    .try_for_each_concurrent(*CONCURRENCY_LIMIT, |payload| {
        let metrics = metrics.clone();
        let chain_id = chain_id.to_owned();
        let block_producer = block_producer.clone();
        async move {
            payload
                .publish(stream, &metrics, &chain_id, &block_producer)
                .await
        }
    })
    .await
}

fn create_publish_payloads(
    tx: &Transaction,
    chain_id: &ChainId,
) -> Vec<PublishPayload<Input>> {
    let tx_id = tx.id(chain_id);

    tx.inputs()
        .par_iter()
        .enumerate()
        .flat_map_iter(|(index, input)| {
            build_input_payloads(tx, tx_id.into(), input, index)
        })
        .collect()
}

fn build_input_payloads(
    tx: &Transaction,
    tx_id: Bytes32,
    input: &Input,
    index: usize,
) -> Vec<PublishPayload<Input>> {
    main_subjects(input, tx_id, index)
        .into_par_iter()
        .chain(InputsByIdSubject::build_subjects_payload(tx, &[input]))
        .map(|subject| PublishPayload {
            subject,
            payload: input.to_owned(),
        })
        .collect()
}

fn main_subjects(
    input: &Input,
    tx_id: Bytes32,
    index: usize,
) -> Vec<SubjectPayload> {
    match input {
        Input::Contract(contract) => {
            let contract_id = contract.contract_id;
            vec![(
                InputsContractSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_contract_id(Some(contract_id.into()))
                    .boxed(),
                InputsContractSubject::WILDCARD,
            )]
        }
        Input::CoinSigned(CoinSigned {
            owner, asset_id, ..
        })
        | Input::CoinPredicate(CoinPredicate {
            owner, asset_id, ..
        }) => {
            vec![(
                InputsCoinSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_owner(Some(owner.into()))
                    .with_asset_id(Some(asset_id.into()))
                    .boxed(),
                InputsCoinSubject::WILDCARD,
            )]
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
            vec![(
                InputsMessageSubject::new()
                    .with_tx_id(Some(tx_id))
                    .with_index(Some(index))
                    .with_sender(Some(sender.into()))
                    .with_recipient(Some(recipient.into()))
                    .boxed(),
                InputsMessageSubject::WILDCARD,
            )]
        }
    }
}

impl IdsExtractable for Input {
    fn extract_identifiers(&self, _tx: &Transaction) -> Vec<Identifier> {
        let mut ids = match self {
            Input::CoinSigned(CoinSigned {
                owner, asset_id, ..
            }) => {
                vec![
                    Identifier::Address(owner.into()),
                    Identifier::AssetId(asset_id.into()),
                ]
            }
            Input::CoinPredicate(CoinPredicate {
                owner, asset_id, ..
            }) => {
                vec![
                    Identifier::Address(owner.into()),
                    Identifier::AssetId(asset_id.into()),
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
                    Identifier::Address(sender.into()),
                    Identifier::Address(recipient.into()),
                ]
            }
            Input::Contract(contract) => {
                vec![Identifier::ContractId(contract.contract_id.into())]
            }
        };

        if let Some((predicate_bytecode, _, _)) = self.predicate() {
            let predicate_tag = crate::sha256(predicate_bytecode);
            ids.push(Identifier::PredicateId(predicate_tag))
        }

        ids
    }
}
