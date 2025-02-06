use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// blocks from a Fuel network. It connects to a streaming service,
// subscribes to a block stream, and prints incoming blocks.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Local).with_api_key("test");
    let mut connection = client.connect().await?;

    println!("Listening for blocks and transactions...");

    let block_subject = BlocksSubject::new();
    let tx_subject = TransactionsSubject::new();
    let filter_subjects = vec![block_subject.into(), tx_subject.into()];

    // Subscribe to the block stream with the specified configuration
    let mut stream = connection
        .subscribe(filter_subjects, DeliverPolicy::FromBlock {
            block_height: 0.into(),
        })
        .await?;

    // Process incoming blocks
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        match &msg.payload {
            MessagePayload::Block(block) => {
                println!("Received block: {:?}", block)
            }
            MessagePayload::Transaction(tx) => {
                println!("Received transaction: {:?}", tx)
            }
            _ => panic!("Wrong data"),
        };
    }

    Ok(())
}
