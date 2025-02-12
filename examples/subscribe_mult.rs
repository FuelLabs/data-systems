use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ⚠️ Please change here for FuelNetwork::Mainnet if you want to connect to the mainnet
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
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
