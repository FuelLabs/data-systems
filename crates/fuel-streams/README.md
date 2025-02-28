<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams</h3>
    <p align="center">
        A library for working with streams of Fuel blockchain data
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-streams" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-streams">
        </a>
        <a href="https://crates.io/crates/fuel-streams" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-streams?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-streams/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-streams/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-streams">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

Fuel Streams is a Rust library designed for working with streams of Fuel blockchain data. It provides an efficient and user-friendly interface for developers to interact with real-time and historical blockchain data, offering support for Fuel-specific data types and leveraging NATS for scalable streaming.

## 🚀 Features

- Real-time streaming of Fuel blockchain data
- Historical streaming of Fuel blockchain data
- Support for Fuel-specific data types
- Efficient data handling using NATS
- Easy-to-use API for subscribing to and processing blockchain events
- Customizable filters for targeted data retrieval
- Seamless integration with other Fuel ecosystem tools

## 🛠️ Installing

First, add these dependencies to your project:

```sh
cargo add fuel-streams futures tokio
```

## 📊 Usage

Here are some examples to get you started with Fuel Streams:

### Basic Connection and Subscription

```rust,no_run
use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client and establish connection
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for blocks...");

    // Choose which subjects do you wanna filter
    let subjects = vec![BlocksSubject::new().into()];

    // Subscribe to blocks with last deliver policy
    let mut stream = connection
        .subscribe(subjects, DeliverPolicy::New)
        .await?;

    while let Some(block) = stream.next().await {
        println!("Received block: {:?}", block);
    }

    Ok(())
}
```

### Subject Types and Filtering

Each data type has its own subject builder for filtering. Here's an example using transaction filtering:

```rust,no_run
use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for transactions...");

    // Create a subject for script transactions
    let subjects = vec![
        TransactionsSubject::new()
            .with_tx_type(Some(TransactionType::Script))
            .into()
    ];

    // Subscribe to the filtered transaction stream
    let mut stream = connection
        .subscribe(subjects, DeliverPolicy::New)
        .await?;

    while let Some(transaction) = stream.next().await {
        println!("Received transaction: {:?}", transaction);
    }

    Ok(())
}
```

Available subject builders include:

- `BlocksSubject::new()`
- `TransactionsSubject::new()`
- `InputsSubject::new()`
- `OutputsSubject::new()`
- `LogsSubject::new()`
- `UtxosSubject::new()`

Each subject builder provides specific filtering methods relevant to its data type. For example, `TransactionsSubject` allows filtering by transaction type using the `with_tx_type()` method.

### Multiple Subscriptions

The Fuel Streams library allows you to subscribe to multiple types of data simultaneously. You can create instances of different subjects, such as `BlocksSubject` and `TransactionsSubject`, and pass them as a vector to the `subscribe` method:

```rust,no_run
use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = Client::new(FuelNetwork::Local).with_api_key("test");
    let mut connection = client.connect().await?;

    println!("Listening for blocks and transactions...");

    let block_subject = BlocksSubject::new();
    let tx_subject = TransactionsSubject::new();
    let filter_subjects = vec![block_subject.into(), tx_subject.into()];

    // Subscribe to the block and transaction streams with the specified configuration
    let mut stream = connection
        .subscribe(filter_subjects, DeliverPolicy::FromBlock {
            block_height: 0.into(),
        })
        .await?;

    // Process incoming blocks and transactions
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
```

### `DeliverPolicy` Options

The `DeliverPolicy` enum provides control over message Deliver in your subscriptions:

- `New`: Delivers only new messages that arrive after subscription
- `FromHeight(u64)`: Delivers messages starting from a specific block height

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This project is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
