use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// transactions from a Fuel network. It connects to a streaming service,
// subscribes to a transaction stream, and prints incoming transactions.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Staging).await?;
    let mut connection = client.connect().await?;

    println!("Listening for transactions...");

    // Create a subject for all transactions
    let subject =
        TransactionsSubject::new().with_kind(Some(TransactionKind::Script)); // Example: filter for script transactions

    // Subscribe to the transaction stream with the specified configuration
    let mut stream = connection
        .subscribe::<Transaction>(subject, DeliverPolicy::Last)
        .await?;

    // Process incoming transactions
    while let Some(msg) = stream.next().await {
        println!("Received transaction: {:?}", msg.payload);
    }

    Ok(())
}
