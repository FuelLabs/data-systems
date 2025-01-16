use std::{sync::Arc, time::Duration};

use anyhow::Result;
use fuel_message_broker::MessageBrokerClient;
use fuel_streams_core::{
    blocks::BlocksSubject,
    inputs::InputsCoinSubject,
    outputs::OutputsCoinSubject,
    subjects::{ReceiptsLogSubject, SubjectBuildable, TransactionsSubject},
    types::{Block, Input, Output, Receipt, Transaction, Utxo},
    utxos::UtxosSubject,
    FuelStreams,
};
use fuel_streams_store::db::{Db, DbConnectionOpts};
use tokio::task::JoinHandle;

use super::{
    results::LoadTestTracker,
    runner_streamable::run_streamable_consumer,
};

pub struct LoadTesterEngine {
    max_subscriptions: u16,
    step_size: u16,
    nats_url: String,
    db_url: String,
}

impl LoadTesterEngine {
    pub fn new(
        nats_url: String,
        db_url: String,
        max_subscriptions: u16,
        step_size: u16,
    ) -> Self {
        Self {
            nats_url,
            db_url,
            max_subscriptions,
            step_size,
        }
    }
}

impl LoadTesterEngine {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let msg_broker =
            MessageBrokerClient::Nats.start(&self.nats_url).await?;
        let db = Db::new(DbConnectionOpts {
            connection_str: self.db_url.clone(),
            ..Default::default()
        })
        .await?
        .arc();
        let fuel_streams = FuelStreams::new(&msg_broker, &db).await.arc();
        let mut handles: Vec<JoinHandle<()>> = vec![];
        // blocks
        let blocks_test_tracker =
            Arc::new(LoadTestTracker::new("Blocks Consumer".into()));
        let blocks_test_tracker_printer = Arc::clone(&blocks_test_tracker);

        // inputs
        let inputs_test_tracker =
            Arc::new(LoadTestTracker::new("Inputs Consumer".into()));
        let inputs_test_tracker_printer = Arc::clone(&inputs_test_tracker);

        // txs
        let txs_test_tracker =
            Arc::new(LoadTestTracker::new("Txs Consumer".into()));
        let txs_test_tracker_printer = Arc::clone(&txs_test_tracker);

        // receipts
        let receipts_test_tracker =
            Arc::new(LoadTestTracker::new("Receipts Consumer".into()));
        let receipts_test_tracker_printer = Arc::clone(&receipts_test_tracker);

        // utxos
        let utxos_test_tracker =
            Arc::new(LoadTestTracker::new("Utxos Consumer".into()));
        let utxos_test_tracker_printer = Arc::clone(&utxos_test_tracker);

        // outputs
        let outputs_test_tracker =
            Arc::new(LoadTestTracker::new("Outputs Consumer".into()));
        let outputs_test_tracker_printer = Arc::clone(&outputs_test_tracker);

        // print regularly the tracked metrics
        handles.push(tokio::spawn(async move {
            loop {
                // blocks
                blocks_test_tracker_printer.refresh();
                println!("{}", blocks_test_tracker_printer);

                // inputs
                inputs_test_tracker_printer.refresh();
                println!("{}", inputs_test_tracker_printer);

                // txs
                txs_test_tracker_printer.refresh();
                println!("{}", txs_test_tracker_printer);

                // utxos
                utxos_test_tracker_printer.refresh();
                println!("{}", utxos_test_tracker_printer);

                // receipts
                receipts_test_tracker_printer.refresh();
                println!("{}", receipts_test_tracker_printer);

                // outputs
                outputs_test_tracker_printer.refresh();
                println!("{}", outputs_test_tracker_printer);

                // do a short pause
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }));

        // Incrementally increase subscriptions
        for current_subs in
            (1..=self.max_subscriptions).step_by(self.step_size as usize)
        {
            let fuel_streams = fuel_streams.clone();
            let blocks_test_tracker = Arc::clone(&blocks_test_tracker);
            for _ in 0..current_subs {
                // blocks
                {
                    let fuel_streams = fuel_streams.clone();
                    let blocks_test_tracker = Arc::clone(&blocks_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) =
                            run_streamable_consumer::<BlocksSubject, Block>(
                                BlocksSubject::new().with_block_height(None),
                                fuel_streams,
                                blocks_test_tracker,
                            )
                            .await
                        {
                            eprintln!(
                                "Error in blocks subscriptions - {:?}",
                                e
                            );
                        }
                    }));
                }
                // inputs
                {
                    let fuel_streams = fuel_streams.clone();
                    let inputs_test_tracker = Arc::clone(&inputs_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) =
                            run_streamable_consumer::<InputsCoinSubject, Input>(
                                InputsCoinSubject::new(),
                                fuel_streams,
                                inputs_test_tracker,
                            )
                            .await
                        {
                            eprintln!(
                                "Error in inputs subscriptions - {:?}",
                                e
                            );
                        }
                    }));
                }
                // txs
                {
                    let fuel_streams = fuel_streams.clone();
                    let txs_test_tracker = Arc::clone(&txs_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<
                            TransactionsSubject,
                            Transaction,
                        >(
                            TransactionsSubject::new(),
                            fuel_streams,
                            txs_test_tracker,
                        )
                        .await
                        {
                            eprintln!("Error in txs subscriptions - {:?}", e);
                        }
                    }));
                }
                // outputs
                {
                    let fuel_streams = fuel_streams.clone();
                    let outputs_test_tracker =
                        Arc::clone(&outputs_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<
                            OutputsCoinSubject,
                            Output,
                        >(
                            OutputsCoinSubject::new(),
                            fuel_streams,
                            outputs_test_tracker,
                        )
                        .await
                        {
                            eprintln!(
                                "Error in outputs subscriptions - {:?}",
                                e
                            );
                        }
                    }));
                }
                // utxos
                {
                    let fuel_streams = fuel_streams.clone();
                    let utxos_test_tracker = Arc::clone(&utxos_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) =
                            run_streamable_consumer::<UtxosSubject, Utxo>(
                                UtxosSubject::new(),
                                fuel_streams,
                                utxos_test_tracker,
                            )
                            .await
                        {
                            eprintln!("Error in utxos subscriptions - {:?}", e);
                        }
                    }));
                }
                // receipts
                {
                    let fuel_streams = fuel_streams.clone();
                    let utxos_test_tracker = Arc::clone(&utxos_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<
                            ReceiptsLogSubject,
                            Receipt,
                        >(
                            ReceiptsLogSubject::new(),
                            fuel_streams,
                            utxos_test_tracker,
                        )
                        .await
                        {
                            eprintln!(
                                "Error in receipts subscriptions - {:?}",
                                e
                            );
                        }
                    }));
                }
            }

            // Small pause between test iterations
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        // cleanup
        for handle in handles.iter() {
            handle.abort();
        }

        Ok(())
    }
}
