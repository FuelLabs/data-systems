use std::{sync::Arc, time::Duration};

use anyhow::Result;
use fuel_streams::client::Client;
use fuel_streams_core::prelude::*;
use tokio::task::JoinHandle;

use super::{
    results::LoadTestTracker,
    runner_streamable::run_streamable_consumer,
};

pub struct LoadTesterEngine {
    max_subscriptions: u16,
    step_size: u16,
    fuel_network: FuelNetwork,
}

impl LoadTesterEngine {
    pub fn new(
        fuel_network: FuelNetwork,
        max_subscriptions: u16,
        step_size: u16,
    ) -> Self {
        Self {
            fuel_network,
            max_subscriptions,
            step_size,
        }
    }
}

impl LoadTesterEngine {
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        let client = Client::connect(self.fuel_network).await?;
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

        // logs
        let logs_test_tracker =
            Arc::new(LoadTestTracker::new("Logs Consumer".into()));
        let logs_test_tracker_printer = Arc::clone(&logs_test_tracker);

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

                // logs
                logs_test_tracker_printer.refresh();
                println!("{}", logs_test_tracker_printer);

                // do a short pause
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }));

        // Incrementally increase subscriptions
        for current_subs in
            (1..=self.max_subscriptions).step_by(self.step_size as usize)
        {
            let client = client.clone();
            let blocks_test_tracker = Arc::clone(&blocks_test_tracker);
            for _ in 0..current_subs {
                // blocks
                {
                    let client = client.clone();
                    let blocks_test_tracker = Arc::clone(&blocks_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<Block>(
                            &client,
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
                // logs
                {
                    let client = client.clone();
                    let logs_test_tracker = Arc::clone(&logs_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<Log>(
                            &client,
                            logs_test_tracker,
                        )
                        .await
                        {
                            eprintln!("Error in logs subscriptions - {:?}", e);
                        }
                    }));
                }
                // inputs
                {
                    let client = client.clone();
                    let inputs_test_tracker = Arc::clone(&inputs_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<Input>(
                            &client,
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
                    let client = client.clone();
                    let txs_test_tracker = Arc::clone(&txs_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<Transaction>(
                            &client,
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
                    let client = client.clone();
                    let outputs_test_tracker =
                        Arc::clone(&outputs_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<Output>(
                            &client,
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
                    let client = client.clone();
                    let utxos_test_tracker = Arc::clone(&utxos_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<Utxo>(
                            &client,
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
                    let client = client.clone();
                    let receipts_test_tracker =
                        Arc::clone(&receipts_test_tracker);
                    handles.push(tokio::spawn(async move {
                        if let Err(e) = run_streamable_consumer::<Receipt>(
                            &client,
                            receipts_test_tracker,
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
