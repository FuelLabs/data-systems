use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// transactions from a Fuel network. It connects to a streaming service,
// subscribes to a transaction stream, and prints incoming transactions.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Mainnet).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for transactions...");

    // Create a subject for all transactions of a specific kind
    let subject =
        TransactionsSubject::new().with_kind(Some(TransactionKind::Script)); // Example: filter for script transactions
    let filter_subjects = vec![subject.into()];

    // Subscribe to the transaction stream with the specified configuration
    let mut stream = connection
        .subscribe(filter_subjects, DeliverPolicy::New)
        .await?;

    // Process incoming transactions
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let transaction = msg.payload.as_transaction()?;
        println!("Received transaction: {:?}", transaction);
    }

    Ok(())
}
