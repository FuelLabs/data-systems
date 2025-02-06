use std::{env, fs, path::Path};

use fuel_streams_domains::{
    blocks::subjects::*,
    inputs::subjects::*,
    outputs::subjects::*,
    receipts::subjects::*,
    transactions::subjects::*,
    utxos::subjects::*,
};
use fuel_streams_subject::subject::{IndexMap, *};

fn main() {
    let block_schema = BlocksSubject::new().schema();
    let transaction_schema = TransactionsSubject::new().schema();
    let utxos_schema = UtxosSubject::new().schema();

    let mut inputs_schema = InputsSubject::new().schema();
    let inputs_coin_schema = InputsCoinSubject::new().schema();
    let inputs_contract_schema = InputsContractSubject::new().schema();
    let inputs_message_schema = InputsMessageSubject::new().schema();
    inputs_schema.set_variant("coin".to_string(), inputs_coin_schema);
    inputs_schema.set_variant("contract".to_string(), inputs_contract_schema);
    inputs_schema.set_variant("message".to_string(), inputs_message_schema);

    let mut outputs_schema = OutputsSubject::new().schema();
    let outputs_coin_schema = OutputsCoinSubject::new().schema();
    let outputs_contract_schema = OutputsContractSubject::new().schema();
    let outputs_change_schema = OutputsChangeSubject::new().schema();
    let outputs_variable_schema = OutputsVariableSubject::new().schema();
    let outputs_contract_created_schema =
        OutputsContractCreatedSubject::new().schema();
    outputs_schema.set_variant("coin".to_string(), outputs_coin_schema);
    outputs_schema.set_variant("contract".to_string(), outputs_contract_schema);
    outputs_schema.set_variant("change".to_string(), outputs_change_schema);
    outputs_schema.set_variant("variable".to_string(), outputs_variable_schema);
    outputs_schema.set_variant(
        "contract_created".to_string(),
        outputs_contract_created_schema,
    );

    let mut receipts_schema = ReceiptsSubject::new().schema();
    let receipts_call_schema = ReceiptsCallSubject::new().schema();
    let receipts_return_schema = ReceiptsReturnSubject::new().schema();
    let receipts_return_data_schema = ReceiptsReturnDataSubject::new().schema();
    let receipts_panic_schema = ReceiptsPanicSubject::new().schema();
    let receipts_revert_schema = ReceiptsRevertSubject::new().schema();
    let receipts_log_schema = ReceiptsLogSubject::new().schema();
    let receipts_log_data_schema = ReceiptsLogDataSubject::new().schema();
    let receipts_transfer_schema = ReceiptsTransferSubject::new().schema();
    let receipts_transfer_out_schema =
        ReceiptsTransferOutSubject::new().schema();
    let receipts_script_result_schema =
        ReceiptsScriptResultSubject::new().schema();
    let receipts_message_out_schema = ReceiptsMessageOutSubject::new().schema();
    let receipts_mint_schema = ReceiptsMintSubject::new().schema();
    let receipts_burn_schema = ReceiptsBurnSubject::new().schema();
    receipts_schema.set_variant("call".to_string(), receipts_call_schema);
    receipts_schema.set_variant("return".to_string(), receipts_return_schema);
    receipts_schema
        .set_variant("return_data".to_string(), receipts_return_data_schema);
    receipts_schema.set_variant("panic".to_string(), receipts_panic_schema);
    receipts_schema.set_variant("revert".to_string(), receipts_revert_schema);
    receipts_schema.set_variant("log".to_string(), receipts_log_schema);
    receipts_schema
        .set_variant("log_data".to_string(), receipts_log_data_schema);
    receipts_schema
        .set_variant("transfer".to_string(), receipts_transfer_schema);
    receipts_schema
        .set_variant("transfer_out".to_string(), receipts_transfer_out_schema);
    receipts_schema.set_variant(
        "script_result".to_string(),
        receipts_script_result_schema,
    );
    receipts_schema
        .set_variant("message_out".to_string(), receipts_message_out_schema);
    receipts_schema.set_variant("mint".to_string(), receipts_mint_schema);
    receipts_schema.set_variant("burn".to_string(), receipts_burn_schema);

    let final_schema = IndexMap::from([
        ("blocks".to_string(), block_schema),
        ("transactions".to_string(), transaction_schema),
        ("inputs".to_string(), inputs_schema),
        ("outputs".to_string(), outputs_schema),
        ("receipts".to_string(), receipts_schema),
        ("utxos".to_string(), utxos_schema),
    ]);

    let schema_json = serde_json::to_string_pretty(&final_schema).unwrap();
    let dest_path =
        Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("schema.json");
    fs::write(dest_path, schema_json).unwrap();
}
