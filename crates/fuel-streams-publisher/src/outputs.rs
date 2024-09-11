<<<<<<< HEAD
use fuel_core_types::fuel_tx::{field::Outputs, Output, UniqueIdentifier};
||||||| parent of b1d06dd (feat(publisher): publish outputs)
use fuel_core::schema::contract;
use fuel_core_types::fuel_tx::{
    field::Outputs,
    UniqueIdentifier,
};
=======
use fuel_core::schema::contract;
use fuel_core_types::fuel_tx::{field::Outputs, Output, UniqueIdentifier};
>>>>>>> b1d06dd (feat(publisher): publish outputs)
use fuel_streams_core::{
<<<<<<< HEAD
    outputs::{
        OutputsChangeSubject,
        OutputsCoinSubject,
        OutputsContractCreatedSubject,
        OutputsContractSubject,
        OutputsVariableSubject,
    },
    prelude::{IntoSubject, *},
    types::{ChainId, Transaction},
||||||| parent of b1d06dd (feat(publisher): publish outputs)
    outputs::{
        OutputsSubject,
        OutputsByIdSubject,
    },
    types::{Bytes32, ChainId, IdentifierKind, Transaction},
=======
    outputs::{OutputsByIdSubject, OutputsSubject},
    prelude::*,
    types::{Bytes32, ChainId, IdentifierKind, Transaction},
>>>>>>> b1d06dd (feat(publisher): publish outputs)
    Stream,
};
<<<<<<< HEAD
||||||| parent of b1d06dd (feat(publisher): publish outputs)
use fuel_streams_macros::subject::IntoSubject;
use fuel_core_types::fuel_tx::Output;
use fuel_streams_core::prelude::*;
=======
use fuel_streams_macros::subject::IntoSubject;
>>>>>>> b1d06dd (feat(publisher): publish outputs)

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
<<<<<<< HEAD
            let subject: Box<dyn IntoSubject> = match output {
                Output::Coin { to, asset_id, .. } => OutputsCoinSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_to(Some((*to).into()))
                    .with_asset_id(Some(asset_id.into()))
                    .boxed(),
                Output::Contract(contract) => {
                    let input_index = contract.input_index as usize;
                    let contract_id =
                        if let Some(Input::Contract(input_contract)) =
                            inputs_from_transaction(transaction)
                                .get(input_index)
                        {
                            Some(input_contract.contract_id)
                        } else {
                            None
                        };
                    OutputsContractSubject::new()
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
                        .with_contract_id(contract_id)
                        .boxed()
                }
                Output::Change { to, asset_id, .. } => {
                    OutputsChangeSubject::new()
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
                        .with_to(Some((*to).into()))
                        .with_asset_id(Some(asset_id.into()))
                        .boxed()
                }
                Output::Variable { to, asset_id, .. } => {
                    OutputsVariableSubject::new()
||||||| parent of b1d06dd (feat(publisher): publish outputs)
            let subject = match output {
                Output::Coin { to, amount, asset_id } =>
                    OutputsSubject::new()
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
                        .with_output_type(Some(OutputType::Coin(
                            CoinSubject::new()
                                .with_from(todo!())
                                .with_to(Some((*to).into()))
                                .with_asset_id(Some(*asset_id))
                        ))),
                Output::Change { to, asset_id, .. } =>
                    OutputsSubject::new()
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
                        .with_output_type(Some(OutputType::Change(
                            ChangeSubject::new()
                                .with_to(Some((*to).into()))
                                .with_asset_id(Some(*asset_id))
                        ))),
                Output::Contract(contract) =>
                    OutputsSubject::new()
                            .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index as u16))
                            .with_output_type(Some(OutputType::Contract(
                                ContractSubject::new()
                                    .with_contract_id(Some(todo!()))
                            ))),
                Output::ContractCreated { contract_id, .. } =>
                    OutputsSubject::new()
=======
            let subject = match output {
                Output::Coin {
                    to,
                    amount,
                    asset_id,
                } => OutputsSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_output_type(Some(OutputType::Coin(
                        CoinSubject::new()
                            .with_from(todo!())
                            .with_to(Some((*to).into()))
                            .with_asset_id(Some(*asset_id)),
                    ))),
                Output::Change { to, asset_id, .. } => OutputsSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_output_type(Some(OutputType::Change(
                        ChangeSubject::new()
                            .with_to(Some((*to).into()))
                            .with_asset_id(Some(*asset_id)),
                    ))),
                Output::Contract(contract) => OutputsSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_output_type(Some(OutputType::Contract(
                        ContractSubject::new().with_contract_id(Some(todo!())),
                    ))),
                Output::ContractCreated { contract_id, .. } => {
                    OutputsSubject::new()
>>>>>>> b1d06dd (feat(publisher): publish outputs)
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
<<<<<<< HEAD
                        .with_to(Some((*to).into()))
                        .with_asset_id(Some(asset_id.into()))
                        .boxed()
                }
                Output::ContractCreated { contract_id, .. } => {
                    OutputsContractCreatedSubject::new()
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
                        .with_contract_id(Some(*contract_id))
                        .boxed()
                }
||||||| parent of b1d06dd (feat(publisher): publish outputs)
                        .with_output_type(Some(OutputType::ContractCreated(
                            ContractCreatedSubject::new()
                                .with_contract_id(Some(*contract_id))
                        ))),
                Output::Variable { to, asset_id, .. } =>
                    OutputsSubject::new()
                        .with_tx_id(Some(tx_id.into()))
                            .with_index(Some(index as u16))
                            .with_output_type(Some(OutputType::Variable(
                                VariableSubject::new()
                                    .with_to(Some((*to).into()))
                                    .with_asset_id(Some(*asset_id))))),
=======
                        .with_output_type(Some(OutputType::ContractCreated(
                            ContractCreatedSubject::new()
                                .with_contract_id(Some(*contract_id)),
                        )))
                }
                Output::Variable { to, asset_id, .. } => OutputsSubject::new()
                    .with_tx_id(Some(tx_id.into()))
                    .with_index(Some(index as u16))
                    .with_output_type(Some(OutputType::Variable(
                        VariableSubject::new()
                            .with_to(Some((*to).into()))
                            .with_asset_id(Some(*asset_id)),
                    ))),
>>>>>>> b1d06dd (feat(publisher): publish outputs)
            };

            stream.publish(&*subject, &output).await?;
        }
    }

    Ok(())
}
