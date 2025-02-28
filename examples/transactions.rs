use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ⚠️ Please change here for FuelNetwork::Mainnet if you want to connect to the mainnet
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for transactions...");

    // Create a subject for all transactions of a specific type
    let subject =
        TransactionsSubject::new().with_tx_type(Some(TransactionType::Script)); // Example: filter for script transactions
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
