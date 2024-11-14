use std::{
    iter::{self, once},
    sync::Arc,
};

use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::fuel_tx::field::ScriptData;
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use super::identifiers::{Identifier, IdsExtractable};
use crate::{
    publisher::{
        packets::{PublishError, PublishOpts, PublishPacket},
        payloads::{
            identifiers::*,
            inputs::publish_tasks as publish_inputs,
            logs::publish_tasks as publish_logs,
            outputs::publish_tasks as publish_outputs,
            receipts::publish_tasks as publish_receipts,
            utxos::publish_tasks as publish_utxos,
        },
    },
    sha256,
    FuelCoreLike,
    Streams,
};

pub fn publish_all_tasks(
    transactions: &[Transaction],
    streams: Streams,
    opts: &Arc<PublishOpts>,
    fuel_core: &dyn FuelCoreLike,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let offline_db = fuel_core.database().off_chain().latest_view().unwrap();

    transactions
        .iter()
        .enumerate()
        .flat_map(|tx_item| {
            let (_, tx) = tx_item;
            let tx_id: Bytes32 = tx.id(&opts.chain_id).into();
            let tx_status: TransactionStatus = offline_db
                .get_tx_status(&tx_id.to_owned().into_inner())
                .unwrap()
                .map(|status| status.into())
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
                tx,
                &tx_id,
                &streams.receipts,
                opts,
                &receipts,
            )))
            .chain(once(publish_logs(&tx_id, &streams.logs, opts, &receipts)))
            .chain(once(publish_utxos(
                tx,
                &tx_id,
                &streams.utxos,
                opts,
                fuel_core,
            )))
            .flatten()
        })
        .collect()
}

fn publish_tasks(
    tx_item: (usize, &Transaction),
    tx_id: &Bytes32,
    tx_status: &TransactionStatus,
    stream: &Stream<Transaction>,
    opts: &Arc<PublishOpts>,
    receipts: &Vec<Receipt>,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let block_height = &opts.block_height;
    packets_from_tx(tx_item, tx_id, tx_status, block_height, receipts)
        .iter()
        .map(|packet| packet.publish(Arc::new(stream.to_owned()), opts))
        .collect()
}

fn packets_from_tx(
    (index, tx): (usize, &Transaction),
    tx_id: &Bytes32,
    tx_status: &TransactionStatus,
    block_height: &BlockHeight,
    receipts: &Vec<Receipt>,
) -> Vec<PublishPacket<Transaction>> {
    let kind = TransactionKind::from(tx.to_owned());
    let packets_from_inputs: Vec<PublishPacket<Transaction>> = tx
        .inputs()
        .par_iter()
        .flat_map(|item| {
            let ids = item.extract_ids(tx, tx_id, index as u8);
            tx.packets_from_ids(ids)
        })
        .collect();

    let packets_from_outputs: Vec<PublishPacket<Transaction>> = tx
        .outputs()
        .par_iter()
        .flat_map(|item| {
            let ids = item.extract_ids(tx, tx_id, index as u8);
            tx.packets_from_ids(ids)
        })
        .collect();

    let packets_from_receipts: Vec<PublishPacket<Transaction>> = receipts
        .par_iter()
        .flat_map(|item| {
            let ids = item.extract_ids(tx, tx_id, index as u8);
            tx.packets_from_ids(ids)
        })
        .collect();

    iter::once(PublishPacket::new(
        tx,
        TransactionsSubject {
            block_height: Some(block_height.to_owned()),
            index: Some(index),
            tx_id: Some(tx_id.to_owned()),
            status: Some(tx_status.to_owned()),
            kind: Some(kind),
        }
        .arc(),
    ))
    .par_bridge()
    .chain(packets_from_inputs)
    .chain(packets_from_outputs)
    .chain(packets_from_receipts)
    .collect()
}

impl IdsExtractable for Transaction {
    fn extract_ids(
        &self,
        _tx: &Transaction,
        tx_id: &Bytes32,
        index: u8,
    ) -> Vec<Identifier> {
        match self {
            Transaction::Script(tx) => {
                let script_tag = sha256(tx.script_data());
                iter::once(Identifier::ScriptID(
                    tx_id.to_owned(),
                    index,
                    script_tag,
                ))
                .collect()
            }
            _ => Vec::new(),
        }
    }
}
