<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://global.discourse-cdn.com/business6/uploads/fuel/original/2X/5/57d5a345cc15a64b636e0d56e042857f8a0e80b1.png" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams Core</h3>
    <p align="center">
        The core library for data streaming in the Fuel Data Systems project.
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-streams-core" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-streams-core">
        </a>
        <a href="https://crates.io/crates/fuel-streams-core" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-streams-core?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-streams-core/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-streams-core/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-streams-core">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

Fuel Streams Core is a library for building data streaming applications on the Fuel blockchain. It provides tools for efficient handling of real-time blockchain data, using NATS for scalable streaming and offering support for Fuel-specific data types.

> [!NOTE]
> This crate is specifically modeled for the Fuel Data Systems project, and is not intended for general use outside of the project.

## 🛠️ Installing

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
fuel-streams-core = "*"
```

## 🚀 Usage

Here's a simple example to get you started with Fuel Streams Core:

```rust,no_run
use std::sync::Arc;
use fuel_streams_core::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> BoxedResult<()> {
    // Connect to NATS server
    let nats_opts = NatsClientOpts::new(FuelNetwork::Local);
    let nats_client = NatsClient::connect(&nats_opts).await?;

    let s3_opts = S3ClientOpts::new(FuelNetwork::Local);
    let s3_client = Arc::new(S3Client::new(&s3_opts).await?);

    // Create a stream for blocks
    let stream = Stream::<Block>::new(&nats_client, &s3_client).await;

    // Subscribe to the stream
    let wildcard = BlocksSubject::wildcard(None, None); // blocks.*.*
    let mut subscription = stream.subscribe(None).await?;

    // Process incoming blocks
    while let Some(block) = subscription.next().await {
        dbg!(block);
    }

    Ok(())
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
