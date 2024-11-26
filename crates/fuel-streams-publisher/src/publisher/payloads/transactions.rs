use std::{iter::once, sync::Arc};

use fuel_core_types::fuel_tx::field::ScriptData;
use fuel_streams_core::prelude::*;
use rayon::prelude::*;
use tokio::task::JoinHandle;

use super::{
    inputs::{self, publish_tasks as publish_inputs},
    logs::publish_tasks as publish_logs,
    outputs::{self, publish_tasks as publish_outputs},
    receipts::{self, publish_tasks as publish_receipts},
    sha256,
    utxos::publish_tasks as publish_utxos,
};
use crate::{publish, FuelCoreLike, PublishOpts, Streams};

pub fn publish_all_tasks(
    transactions: &[FuelCoreTransaction],
    streams: Streams,
    opts: &Arc<PublishOpts>,
    fuel_core: &dyn FuelCoreLike,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let offchain_database = Arc::clone(&opts.offchain_database);

    transactions
        .iter()
        .enumerate()
        .flat_map(|tx_item| {
            let (_, tx) = tx_item;
            let tx_id: Bytes32 = tx.id(&opts.chain_id).into();
            let tx_status: TransactionStatus = offchain_database
                .get_tx_status(&tx_id.to_owned().into_inner())
                .unwrap()
                .map(|status| (&status).into())
                .unwrap_or_default();

            let receipts = fuel_core
                .get_receipts(&tx_id.to_owned().into_inner())
                .unwrap_or_default()
                .unwrap_or_default();

            once(publish_tasks(
                tx_item,
                &tx_id,
                &tx_status,
                &streams.transactions,
                opts,
                &receipts,
            ))
            .chain(once(publish_inputs(tx, &tx_id, &streams.inputs, opts)))
            .chain(once(publish_outputs(tx, &tx_id, &streams.outputs, opts)))
            .chain(once(publish_receipts(
                &tx_id,
                &streams.receipts,
                opts,
                &receipts,
            )))
            .chain(once(publish_logs(&tx_id, &streams.logs, opts, &receipts)))
            .chain(once(publish_utxos(tx, &tx_id, &streams.utxos, opts)))
            .flatten()
        })
        .collect()
}

fn publish_tasks(
    tx_item: (usize, &FuelCoreTransaction),
    tx_id: &Bytes32,
    tx_status: &TransactionStatus,
    stream: &Stream<Transaction>,
    opts: &Arc<PublishOpts>,
    receipts: &Vec<FuelCoreReceipt>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let block_height = &opts.block_height;
    let base_asset_id = &opts.base_asset_id;

    packets_from_tx(
        tx_item,
        tx_id,
        tx_status,
        base_asset_id,
        block_height,
        receipts,
    )
    .iter()
    .map(|packet| publish(packet, Arc::new(stream.to_owned()), opts))
    .collect()
}

fn packets_from_tx(
    (index, tx): (usize, &FuelCoreTransaction),
    tx_id: &Bytes32,
    tx_status: &TransactionStatus,
    base_asset_id: &FuelCoreAssetId,
    block_height: &BlockHeight,
    receipts: &Vec<FuelCoreReceipt>,
) -> Vec<PublishPacket<Transaction>> {
    let main_subject = TransactionsSubject {
        block_height: Some(block_height.to_owned()),
        index: Some(index),
        tx_id: Some(tx_id.to_owned()),
        status: Some(tx_status.to_owned()),
        kind: Some(tx.into()),
    }
    .arc();

    let transaction =
        Transaction::new(tx_id, tx, tx_status, base_asset_id, receipts);
    let mut packets = vec![transaction.to_packet(main_subject)];

    packets.extend(
        identifiers(tx, tx_id, index as u8)
            .into_par_iter()
            .map(|identifier| identifier.into())
            .map(|subject: TransactionsByIdSubject| subject.arc())
            .map(|subject| transaction.to_packet(subject))
            .collect::<Vec<_>>(),
    );

    let packets_from_inputs: Vec<PublishPacket<Transaction>> = tx
        .inputs()
        .par_iter()
        .flat_map(|input| {
            inputs::identifiers(input, tx_id, index as u8)
                .into_par_iter()
                .map(|identifier| identifier.into())
                .map(|subject: TransactionsByIdSubject| subject.arc())
                .map(|subject| transaction.to_packet(subject))
        })
        .collect();

    packets.extend(packets_from_inputs);

    let packets_from_outputs: Vec<PublishPacket<Transaction>> = tx
        .outputs()
        .par_iter()
        .flat_map(|output| {
            outputs::identifiers(output, tx, tx_id, index as u8)
                .into_par_iter()
                .map(|identifier| identifier.into())
                .map(|subject: TransactionsByIdSubject| subject.arc())
                .map(|subject| transaction.to_packet(subject))
        })
        .collect();

    packets.extend(packets_from_outputs);

    let packets_from_receipts: Vec<PublishPacket<Transaction>> = receipts
        .par_iter()
        .flat_map(|receipt| {
            receipts::identifiers(receipt, tx_id, index as u8)
                .into_par_iter()
                .map(|identifier| identifier.into())
                .map(|subject: TransactionsByIdSubject| subject.arc())
                .map(|subject| transaction.to_packet(subject))
        })
        .collect();

    packets.extend(packets_from_receipts);

    packets
}

fn identifiers(
    tx: &FuelCoreTransaction,
    tx_id: &Bytes32,
    index: u8,
) -> Vec<Identifier> {
    match tx {
        FuelCoreTransaction::Script(tx) => {
            let script_tag = sha256(tx.script_data());
            vec![Identifier::ScriptID(tx_id.to_owned(), index, script_tag)]
        }
        _ => Vec::new(),
    }
}
