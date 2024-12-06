<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://global.discourse-cdn.com/business6/uploads/fuel/original/2X/5/57d5a345cc15a64b636e0d56e042857f8a0e80b1.png" alt="Logo" width="80" height="80">
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

> [!WARNING]
> This project is currently under development and is not yet ready for production use.

Fuel Streams is a Rust library designed for working with streams of Fuel blockchain data. It provides an efficient and user-friendly interface for developers to interact with real-time blockchain data, offering support for Fuel-specific data types and leveraging NATS for scalable streaming.

## 🚀 Features

- Real-time streaming of Fuel blockchain data
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

### Subscribing to all new blocks

```rust,no_run
use fuel_streams::types::FuelNetwork;
use fuel_streams::client::Client;
use fuel_streams::stream::{Stream, StreamEncoder};
use fuel_streams::blocks::Block;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), fuel_streams::Error> {
    let client = Client::connect(FuelNetwork::Local).await?;
    let stream = fuel_streams::Stream::<Block>::new(&client).await;

    let mut subscription = stream.subscribe().await?;
    while let Some(bytes) = subscription.next().await {
        let block = Block::decode(bytes.unwrap());
        println!("Received block: {:?}", block);
    }

    Ok(())
}
```

### Subscribing to all transactions (Filtering by block height)

```rust,no_run
use fuel_streams::types::FuelNetwork;
use fuel_streams::client::Client;
use fuel_streams::stream::{Filter, Stream, StreamEncoder, StreamConfig};
use fuel_streams::transactions::{Transaction, TransactionKind, TransactionsSubject};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), fuel_streams::Error> {
    let client = Client::connect(FuelNetwork::Local).await?;
    let mut stream = fuel_streams::Stream::<Transaction>::new(&client).await;

    // Filter transactions from block height 5
    let filter = Filter::<TransactionsSubject>::build()
      .with_block_height(Some(5.into()));

    let mut subscription = stream
        .with_filter(filter)
        .subscribe_with_config(StreamConfig::default())
        .await?;

    while let Some(message) = subscription.next().await {
        let payload = message?.payload.clone();
        let transaction = Transaction::decode(payload.into());
        println!("Received transaction: {:?}", transaction);
    }

    Ok(())
}
```

## Advanced

### `DeliverPolicy`

The `DeliverPolicy` provides fine-grained control over message delivery in your stream. This powerful feature allows you to customize how and when messages are received. Below is an illustrative example demonstrating how to subscribe to all blocks from the first block until the last block in the stream:

```rust,no_run
use fuel_streams::types::FuelNetwork;
use fuel_streams::client::Client;
use fuel_streams::stream::{Stream, StreamConfig, StreamEncoder, Filter};
use fuel_streams::blocks::{Block, BlocksSubject};
use fuel_streams::types::DeliverPolicy;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), fuel_streams::Error> {
    let client = Client::connect(FuelNetwork::Local).await?;
    let mut stream = fuel_streams::Stream::<Block>::new(&client).await;

    let filter = Filter::<BlocksSubject>::build();
    let mut subscription = stream
        .with_filter(filter)
        .subscribe_with_config(StreamConfig {
            // Set the deliver policy to `All` to receive all blocks
            // from the first block until the last block in the stream
            deliver_policy: DeliverPolicy::All,
        })
        .await?;

    while let Some(message) = subscription.next().await {
        let payload = message?.payload.clone();
        let block = Block::decode(payload.into());
        println!("Received block: {:?}", block);
    }

    Ok(())
}
```

Available `DeliverPolicy` options:

- `All`: Delivers all messages in the stream.
- `Last`: Delivers the last message for the selected subjects.
- `New`: Delivers only new messages that are received after the subscription is created.
- `ByStartSequence(u64)`: Delivers messages starting from a specific sequence number.
- `ByStartTime(DateTime<Utc>)`: Delivers messages starting from a specific time.

Choose the appropriate `DeliverPolicy` based on your application's requirements for historical data processing or real-time updates.

### Filters

Filters allow you to narrow down the data you receive from a stream based on specific criteria. This is particularly useful when you're only interested in a subset of the data. The `Stream` struct provides a `with_filter` method that allows you to apply filters to your subscription.

Here's an example of how to use filters with a stream of transactions:

```rust,no_run
use fuel_streams::types::FuelNetwork;
use fuel_streams::client::Client;
use fuel_streams::stream::{Stream, StreamConfig, StreamEncoder, Filter};
use fuel_streams::transactions::{Transaction, TransactionsSubject, TransactionKind};
use fuel_streams::types::Address;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), fuel_streams::Error> {
    let client = Client::connect(FuelNetwork::Local).await?;
    let mut stream = fuel_streams::Stream::<Transaction>::new(&client).await;

    // Create a filter for transactions from a specific block height and kind
    let filter = Filter::<TransactionsSubject>::build()
        .with_block_height(Some(1000.into()))
        .with_kind(Some(TransactionKind::Script));

    let mut subscription = stream
        .with_filter(filter)
        .subscribe_with_config(StreamConfig::default())
        .await?;

    while let Some(message) = subscription.next().await {
        let payload = message?.payload.clone();
        let transaction = Transaction::decode(payload.into());
        println!("Received filtered transaction: {:?}", transaction);
    }

    Ok(())
}
```

In this example, we're creating a filter that will only return transactions from a specific kind (`TransactionKind::Script`) and from a specific block height (1000).

Available filter methods depend on the subject type. The project currently supports subjects for the following data types:

- [Blocks](../fuel-streams-core/src/blocks/subjects.rs)
- [Transactions](../fuel-streams-core/src/transactions/subjects.rs)

Filters can be combined to create more specific queries. Each filter method narrows down the results further.

> [!NOTE]
> Remember that the effectiveness of filters depends on how the data is structured in the NATS streams. Filters are applied on the client side, so they can help reduce the amount of data your application needs to process, but they don't reduce the amount of data transferred over the network.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This project is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
