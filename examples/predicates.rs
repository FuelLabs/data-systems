use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ⚠️ Please change here for FuelNetwork::Mainnet if you want to connect to the mainnet
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for predicates...");

    // Create a subject for all predicates
    let subject = PredicatesSubject::new();
    let filter_subjects = vec![subject.into()];

    // Subscribe to the predicate stream with the specified configuration
    let mut stream = connection
        .subscribe(filter_subjects, DeliverPolicy::New)
        .await?;

    // Process incoming predicates
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let predicate = msg.payload.as_predicate()?;
        println!("Received predicate: {:?}", predicate);
    }

    Ok(())
}
