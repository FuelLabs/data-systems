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

    // Create a subject for all blocks
    let subject = BlocksSubject::new();

    // Subscribe to blocks with last delivery policy
    let mut stream = connection
        .subscribe::<Block>(subject, DeliverPolicy::New)
        .await?;

    while let Some(block) = stream.next().await {
        println!("Received block: {:?}", block);
    }

    Ok(())
}
```

### Custom Connection Options

```rust,no_run
use fuel_streams::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with custom connection options
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
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
    let subject = TransactionsSubject::new()
        .with_kind(Some(TransactionKind::Script));

    // Subscribe to the filtered transaction stream
    let mut stream = connection
        .subscribe::<Transaction>(subject, DeliverPolicy::New)
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

Each subject builder provides specific filtering methods relevant to its data type. For example, `TransactionsSubject` allows filtering by transaction kind using the `with_kind()` method.

### `DeliverPolicy` Options

The `DeliverPolicy` enum provides control over message delivery in your subscriptions:

- `All`: Delivers all messages in the stream
- `Last`: Delivers only the last message for the selected subjects
- `New`: Delivers only new messages that arrive after subscription
- `ByStartSequence(u64)`: Delivers messages starting from a specific sequence number
- `ByStartTime(DateTime<Utc>)`: Delivers messages starting from a specific time

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This project is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
