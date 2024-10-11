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

use crate::prefix_subject;

pub async fn publish(
    stream: &Stream<fuel_core_types::fuel_tx::Output>,
    chain_id: &ChainId,
    transaction: &Transaction,
    subject_prefix: Option<String>,
) -> anyhow::Result<()> {
    let tx_id = transaction.id(chain_id);
    let outputs = transaction.outputs();
    for (index, output) in outputs.iter().enumerate() {
        let (subject, by_id_subject): (
            Box<dyn IntoSubject>,
            OutputsByIdSubject,
        ) = match output {
            Output::Coin { to, asset_id, .. } => (
                OutputsCoinSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_to(Some((*to).into()))
                    .with_asset_id(Some((*asset_id).into()))
                    .boxed(),
                OutputsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some(Bytes32::from(*to))),
            ),
            Output::Contract(contract) => {
                let input_index = contract.input_index as usize;
                let contract_id = if let Input::Contract(input_contract) =
                    &transaction.inputs()[input_index]
                {
                    input_contract.contract_id
                } else {
                    anyhow::bail!("Contract input not found");
                };
                (
                    OutputsContractSubject::new()
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
                        .with_contract_id(Some(contract_id.into()))
                        .boxed(),
                    OutputsByIdSubject::new()
                        .with_id_kind(Some(IdentifierKind::ContractID))
                        .with_id_value(Some(Bytes32::from(*contract_id))),
                )
            }
            Output::Change { to, asset_id, .. } => (
                OutputsChangeSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_to(Some((*to).into()))
                    .with_asset_id(Some((*asset_id).into()))
                    .boxed(),
                OutputsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some(Bytes32::from(*to))),
            ),
            Output::Variable { to, asset_id, .. } => (
                OutputsVariableSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_to(Some((*to).into()))
                    .with_asset_id(Some((*asset_id).into()))
                    .boxed(),
                OutputsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::Address))
                    .with_id_value(Some(Bytes32::from(*to))),
            ),
            Output::ContractCreated { contract_id, .. } => (
                OutputsContractCreatedSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_contract_id(Some((*contract_id).into()))
                    .boxed(),
                OutputsByIdSubject::new()
                    .with_id_kind(Some(IdentifierKind::ContractID))
                    .with_id_value(Some(Bytes32::from(*contract_id))),
            ),
        };

        let subject = prefix_subject(&subject_prefix, &*subject);
        let by_id_subject = prefix_subject(&subject_prefix, &by_id_subject);

        stream.publish_raw(&subject, output).await?;
        stream.publish_raw(&by_id_subject, output).await?;
    }

    Ok(())
}
