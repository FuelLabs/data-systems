use fuel_core_types::fuel_tx::{field::Outputs, Output, UniqueIdentifier};
use fuel_streams_core::{
    outputs::{
        OutputsChangeSubject,
        OutputsCoinSubject,
        OutputsContractCreatedSubject,
        OutputsContractSubject,
        OutputsVariableSubject,
    },
    prelude::{IntoSubject, *},
    types::{ChainId, Transaction},
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
                        .with_tx_id(Some(tx_id.into()))
                        .with_index(Some(index as u16))
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
            };

            stream.publish(&*subject, &output).await?;
        }
    }

    Ok(())
}
