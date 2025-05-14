<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Message Broker</h3>
    <p align="center">
        A message broker implementation for the Fuel Data Systems project
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-message-broker" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-message-broker">
        </a>
        <a href="https://crates.io/crates/fuel-message-broker" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-message-broker?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-message-broker/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-message-broker/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-message-broker">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

Fuel Message Broker provides a high-performance message broker implementation for the Fuel Data Systems project. It offers a unified interface for message publishing and subscription, with a primary implementation based on NATS JetStream.

> [!NOTE]
> This crate is specifically modeled for the Fuel Data Systems project, and is not intended for general use outside of the project.

## 🏗️ Architecture

The crate is built around the following key components:

- **Message Broker Interface**: A trait-based abstraction for message broker operations
- **NATS Implementation**: A concrete implementation using NATS JetStream
- **Work Queues**: Support for reliable work queues with message acknowledgment
- **Namespaces**: Logical separation of message subjects/topics

## 🚀 Features

- **Pub/Sub Messaging**: Publish and subscribe to messages with topic-based routing
- **Work Queues**: Reliable work queues with message acknowledgment and redelivery
- **Namespacing**: Logical separation of message topics
- **Health Monitoring**: Built-in health checks and metrics for monitoring
- **Error Handling**: Comprehensive error types for all messaging operations

## 🛠️ Usage

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
fuel-message-broker = "*"
```

### Basic Setup

```rust
use std::sync::Arc;
use fuel_message_broker::{NatsMessageBroker, MessageBrokerError};

#[tokio::main]
async fn main() -> Result<(), MessageBrokerError> {
    // Connect to NATS server
    let broker = NatsMessageBroker::setup(
        "nats://localhost:4222",
        Some("my-namespace")
    ).await?;

    // Check connection status
    if broker.is_connected() {
        println!("Connected to NATS server!");
    }

    Ok(())
}
```

### Publishing Messages

```rust
use fuel_message_broker::{NatsMessageBroker, MessageBrokerError};

async fn publish_example(broker: &NatsMessageBroker) -> Result<(), MessageBrokerError> {
    // Publish a message to a topic
    let payload = "Hello, world!".as_bytes();
    broker.publish("my-topic", payload.into()).await?;

    println!("Message published successfully");
    Ok(())
}
```

### Subscribing to Messages

```rust
use fuel_message_broker::{NatsMessageBroker, MessageBrokerError};
use futures::StreamExt;

async fn subscribe_example(broker: &NatsMessageBroker) -> Result<(), MessageBrokerError> {
    // Subscribe to a topic
    let mut stream = broker.subscribe("my-topic").await?;

    println!("Waiting for messages...");

    // Process incoming messages
    while let Some(msg_result) = stream.next().await {
        let msg = msg_result?;
        let payload = msg.as_ref();

        println!("Received message: {:?}", payload);
    }

    Ok(())
}
```

### Using Work Queues

```rust
use fuel_message_broker::{NatsQueue, NatsSubject, NatsMessageBroker, MessageBrokerError};
use futures::StreamExt;
use std::sync::Arc;

async fn work_queue_example() -> Result<(), MessageBrokerError> {
    // Connect to NATS server
    let broker = NatsMessageBroker::setup("nats://localhost:4222", None).await?;

    // Create a work queue
    let queue = NatsQueue::BlockImporter(broker);

    // Publish a task to the queue
    queue.publish(&NatsSubject::BlockSubmitted(123), vec![1, 2, 3]).await?;

    // Subscribe to process tasks
    let mut stream = queue.subscribe(10).await?;

    // Process tasks
    while let Some(msg_result) = stream.next().await {
        let msg = msg_result?;
        let payload = msg.payload();

        println!("Processing task: {:?}", payload);

        // Acknowledge the message when done
        msg.ack().await?;
    }

    Ok(())
}
```

### Health Monitoring

```rust
use fuel_message_broker::{NatsMessageBroker, MessageBrokerError};
use std::sync::Arc;

async fn health_check_example(broker: &NatsMessageBroker) -> Result<(), MessageBrokerError> {
    // Check if broker is healthy
    let is_healthy = broker.is_healthy().await;

    // Get detailed health information
    let uptime_secs = 3600; // Example uptime
    let health_info = broker.get_health_info(uptime_secs).await?;

    println!("Broker health: {}", is_healthy);
    println!("Health details: {}", serde_json::to_string_pretty(&health_info)?);

    Ok(())
}
```

## 🔧 Configuration

The NATS implementation can be configured with various options:

```rust
use fuel_message_broker::{NatsOpts, Namespace, NatsMessageBroker};

async fn configure_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom options
    let opts = NatsOpts::new("nats://localhost:4222")
        .with_namespace("production")
        .with_timeout(10)  // Connection timeout in seconds
        .with_ack_wait(30); // Message acknowledgment timeout

    // Connect with custom options
    let broker = NatsMessageBroker::new(&opts).await?;

    Ok(())
}
```

## 🧪 Testing

The crate provides test helpers for setting up isolated test environments:

```rust
use fuel_message_broker::{NatsOpts, NatsMessageBroker};

#[tokio::test]
async fn test_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create options with a random namespace for test isolation
    let opts = NatsOpts::new("nats://localhost:4222")
        .with_rdn_namespace();

    // Connect with isolated namespace
    let broker = NatsMessageBroker::new(&opts).await?;

    // Run tests with isolated broker
    // ...

    Ok(())
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
