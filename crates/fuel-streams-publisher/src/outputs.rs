use std::sync::Arc;

use fuel_core_types::fuel_tx::{Output, UniqueIdentifier};
use fuel_streams_core::{
    outputs::{
        OutputsByIdSubject,
        OutputsChangeSubject,
        OutputsCoinSubject,
        OutputsContractCreatedSubject,
        OutputsContractSubject,
        OutputsVariableSubject,
    },
    prelude::*,
    transactions::TransactionExt,
    types::{ChainId, IdentifierKind, Transaction},
    Stream,
};

use crate::{
    maybe_include_predicate_and_script_subjects,
    metrics::PublisherMetrics,
    publish_all,
};

#[allow(clippy::too_many_arguments)]
pub async fn publish(
    stream: &Stream<fuel_core_types::fuel_tx::Output>,
    chain_id: &ChainId,
    transaction: &Transaction,
    metrics: &Arc<PublisherMetrics>,
    block_producer: &Address,
    predicate_tag: Option<Bytes32>,
    script_tag: Option<Bytes32>,
) -> anyhow::Result<()> {
    let tx_id = transaction.id(chain_id);
    let outputs = transaction.outputs();
    for (index, output) in outputs.iter().enumerate() {
        let mut subjects: Vec<(Box<dyn IntoSubject>, &'static str)> =
            match output {
                Output::Coin { to, asset_id, .. } => vec![
                    (
                        OutputsCoinSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index as u16))
                            .with_to(Some((*to).into()))
                            .with_asset_id(Some((*asset_id).into()))
                            .boxed(),
                        OutputsCoinSubject::WILDCARD,
                    ),
                    (
                        OutputsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::Address))
                            .with_id_value(Some(Bytes32::from(*to)))
                            .boxed(),
                        OutputsByIdSubject::WILDCARD,
                    ),
                ],
                Output::Contract(contract) => {
                    let input_index = contract.input_index as usize;
                    let contract_id = if let Input::Contract(input_contract) =
                        &transaction.inputs()[input_index]
                    {
                        input_contract.contract_id
                    } else {
                        anyhow::bail!("Contract input not found");
                    };
                    vec![
                        (
                            OutputsContractSubject::new()
                                .with_tx_id(Some(tx_id.into()))
                                .with_index(Some(index as u16))
                                .with_contract_id(Some(contract_id.into()))
                                .boxed(),
                            OutputsContractSubject::WILDCARD,
                        ),
                        (
                            OutputsByIdSubject::new()
                                .with_id_kind(Some(IdentifierKind::ContractID))
                                .with_id_value(Some(Bytes32::from(
                                    *contract_id,
                                )))
                                .boxed(),
                            OutputsByIdSubject::WILDCARD,
                        ),
                    ]
                }
                Output::Change { to, asset_id, .. } => vec![
                    (
                        OutputsChangeSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index as u16))
                            .with_to(Some((*to).into()))
                            .with_asset_id(Some((*asset_id).into()))
                            .boxed(),
                        OutputsChangeSubject::WILDCARD,
                    ),
                    (
                        OutputsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::Address))
                            .with_id_value(Some(Bytes32::from(*to)))
                            .boxed(),
                        OutputsByIdSubject::WILDCARD,
                    ),
                ],
                Output::Variable { to, asset_id, .. } => vec![
                    (
                        OutputsVariableSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index as u16))
                            .with_to(Some((*to).into()))
                            .with_asset_id(Some((*asset_id).into()))
                            .boxed(),
                        OutputsVariableSubject::WILDCARD,
                    ),
                    (
                        OutputsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::Address))
                            .with_id_value(Some(Bytes32::from(*to)))
                            .boxed(),
                        OutputsByIdSubject::WILDCARD,
                    ),
                ],

                Output::ContractCreated { contract_id, .. } => vec![
                    (
                        OutputsContractCreatedSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index as u16))
                            .with_contract_id(Some((*contract_id).into()))
                            .boxed(),
                        OutputsContractCreatedSubject::WILDCARD,
                    ),
                    (
                        OutputsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some(Bytes32::from(*contract_id)))
                            .boxed(),
                        OutputsByIdSubject::WILDCARD,
                    ),
                ],
            };

        maybe_include_predicate_and_script_subjects(
            &mut subjects,
            &predicate_tag,
            &script_tag,
        );

        publish_all(
            stream,
            subjects,
            output,
            metrics,
            chain_id,
            block_producer,
        )
        .await;
    }

    Ok(())
}
