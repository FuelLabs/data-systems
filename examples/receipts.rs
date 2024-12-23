use fuel_streams::prelude::*;
use futures::StreamExt;

/// The contract ID to stream the receipts for. For this example, we're using the contract ID of the https://thundernft.market/
const CONTRACT_ID: &str =
    "0x243ef4c2301f44eecbeaf1c39fee9379664b59a2e5b75317e8c7e7f26a25ed4d";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Staging).await?;
    let mut connection = client.connect().await?;

    println!("Listening for receipts...");

    // Create a subject for all receipt types related to the contract
    let subject = ReceiptsByIdSubject::new()
        .with_id_kind(Some(IdentifierKind::ContractID))
        .with_id_value(Some(CONTRACT_ID.into()));

    // Subscribe to the receipt stream with the specified configuration
    let mut stream = connection
        .subscribe::<Receipt>(subject, DeliverPolicy::All)
        .await?;

    // Process incoming receipts
    while let Some(receipt) = stream.next().await {
        println!("Received receipt: {:?}", receipt);
    }

    Ok(())
}
