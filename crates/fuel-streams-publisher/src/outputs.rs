use fuel_core_types::fuel_tx::{field::Outputs, Output, UniqueIdentifier};
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
    types::{ChainId, IdentifierKind, Transaction},
    Stream,
};

macro_rules! get_outputs {
    ($transaction:expr, $($variant:ident),+) => {
        match $transaction {
            Transaction::Mint(_) => vec![],
            $(Transaction::$variant(tx) => tx.outputs().to_vec(),)+
        }
    };
}

fn outputs_from_transaction(transaction: &Transaction) -> Vec<Output> {
    get_outputs!(transaction, Script, Blob, Create, Upload, Upgrade)
}

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

pub async fn publish(
    stream: &Stream<fuel_core_types::fuel_tx::Output>,
    chain_id: &ChainId,
    transactions: &[Transaction],
) -> anyhow::Result<()> {
    for transaction in transactions {
        let tx_id = transaction.id(chain_id);
        let outputs = outputs_from_transaction(transaction);
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
                        .with_id_value(Some((*to).into())),
                ),
                Output::Contract(contract) => {
                    let input_index = contract.input_index as usize;
                    let contract_id = if let Input::Contract(input_contract) =
                        &inputs_from_transaction(transaction)[input_index]
                    {
                        input_contract.contract_id
                    } else {
                        anyhow::bail!("Contract input not found");
                    };
                    (
                        OutputsContractSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index as u16))
                            .with_contract_id(Some(contract_id))
                            .boxed(),
                        OutputsByIdSubject::new()
                            .with_id_kind(Some(IdentifierKind::ContractID))
                            .with_id_value(Some(contract_id.into())),
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
                        .with_id_value(Some((*to).into())),
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
                        .with_id_value(Some((*to).into())),
                ),
                Output::ContractCreated { contract_id, .. } => (
                    OutputsContractCreatedSubject::new()
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
                        .with_contract_id(Some(*contract_id))
                        .boxed(),
                    OutputsByIdSubject::new()
                        .with_id_kind(Some(IdentifierKind::ContractID))
                        .with_id_value(Some((*contract_id).into())),
                ),
            };

            stream.publish(&*subject, &output).await?;
            stream.publish(&by_id_subject, &output).await?;
        }
    }

    Ok(())
}
