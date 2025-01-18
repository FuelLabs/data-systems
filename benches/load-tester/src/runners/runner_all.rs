use std::{sync::Arc, time::Duration};

use anyhow::Result;
use fuel_streams::prelude::*;
use fuel_streams_core::{
    blocks::BlocksSubject,
    inputs::InputsCoinSubject,
    outputs::OutputsCoinSubject,
    subjects::{ReceiptsLogSubject, SubjectBuildable, TransactionsSubject},
    types::{Block, Input, Output, Receipt, Transaction, Utxo},
    utxos::UtxosSubject,
};
use tokio::task::JoinHandle;

use super::{
    results::LoadTestTracker,
    runner_streamable::spawn_streamable_consumer,
};

pub struct LoadTesterEngine {
    max_subscriptions: u16,
    step_size: u16,
    api_key: String,
    network: FuelNetwork,
}

impl LoadTesterEngine {
    pub fn new(
        network: FuelNetwork,
        api_key: String,
        max_subscriptions: u16,
        step_size: u16,
    ) -> Self {
        Self {
            network,
            api_key,
            max_subscriptions,
            step_size,
        }
    }
}

impl LoadTesterEngine {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
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
            let blocks_test_tracker = Arc::clone(&blocks_test_tracker);
            for _ in 0..current_subs {
                // blocks
                handles.push(
                    spawn_streamable_consumer::<BlocksSubject, Block>(
                        self.network,
                        self.api_key.clone(),
                        BlocksSubject::new().with_height(None),
                        Arc::clone(&blocks_test_tracker),
                    )
                    .await?,
                );

                // inputs
                handles.push(
                    spawn_streamable_consumer::<InputsCoinSubject, Input>(
                        self.network,
                        self.api_key.clone(),
                        InputsCoinSubject::new(),
                        Arc::clone(&inputs_test_tracker),
                    )
                    .await?,
                );

                // txs
                handles.push(spawn_streamable_consumer::<TransactionsSubject, Transaction>(self.network,  self.api_key.clone(), TransactionsSubject::new(),  Arc::clone(&txs_test_tracker)).await?);

                // outputs
                handles.push(
                    spawn_streamable_consumer::<OutputsCoinSubject, Output>(
                        self.network,
                        self.api_key.clone(),
                        OutputsCoinSubject::new(),
                        Arc::clone(&outputs_test_tracker),
                    )
                    .await?,
                );

                // utxos
                handles.push(
                    spawn_streamable_consumer::<UtxosSubject, Utxo>(
                        self.network,
                        self.api_key.clone(),
                        UtxosSubject::new(),
                        Arc::clone(&utxos_test_tracker),
                    )
                    .await?,
                );

                // receipts
                handles.push(
                    spawn_streamable_consumer::<ReceiptsLogSubject, Receipt>(
                        self.network,
                        self.api_key.clone(),
                        ReceiptsLogSubject::new(),
                        Arc::clone(&receipts_test_tracker),
                    )
                    .await?,
                );
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
