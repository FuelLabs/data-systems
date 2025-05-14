# Fuel Publisher Service

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml)
[![Coverage](https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj)](https://codecov.io/gh/FuelLabs/data-systems)

## About

The Fuel Publisher Service subscribes to new blocks from the Fuel blockchain and publishes them to a message broker for further processing. It serves as the primary data ingestion component in the Fuel data systems architecture, ensuring all blockchain data is captured and made available to downstream services.

## Features

- **Block Subscription**: Subscribes to live blocks from Fuel Core nodes
- **Message Publishing**: Publishes blockchain data to a message broker (NATS)
- **Gap Detection**: Identifies and fills gaps in the blockchain data
- **Block Recovery**: Recovers missing blocks to ensure data completeness
- **Metrics Collection**: Monitors performance and data flow with Prometheus metrics
- **Graceful Shutdown**: Ensures proper closing of connections and data flushing

## Architecture

The service consists of several key components:

- **Block Publisher**: Subscribes to new blocks and publishes them to the message broker
- **Gap Detector**: Identifies missing blocks in the data stream
- **Block Recovery**: Retrieves missing blocks from the blockchain
- **State Management**: Maintains publisher state for resilience
- **Metrics Collection**: Tracks publishing performance and health

### Data Flow

1. The service connects to a Fuel Core node and subscribes to new blocks
2. Each new block is parsed and transformed into a standardized format
3. Blocks are published to a NATS message broker for downstream consumption
4. The gap detector periodically checks for missing blocks in the sequence
5. Any identified gaps are filled by retrieving the missing blocks
6. Metrics are collected throughout the process for monitoring

## Getting Started

### Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install)
- Running Fuel Core node
- NATS server
- [Docker](https://www.docker.com/get-started/) (optional)

### Environment Setup

Create a `.env` file by copying the example:

```bash
make create-env
```

This will include the necessary variables for connecting to the Fuel Core node and NATS message broker.

### NATS Message Broker

Start the NATS server using Docker:

```bash
# Start NATS
make start-nats
```

### Running the Service

The Publisher service can be run using the Makefile commands:

```bash
# Run with default configuration (testnet)
make run-publisher

# Run against mainnet in development mode
make run-publisher-mainnet-dev

# Run against testnet in development mode
make run-publisher-testnet-dev

# Run against mainnet in profiling mode
make run-publisher-mainnet-profiling

# Run against testnet in profiling mode
make run-publisher-testnet-profiling

# Run with custom parameters
make run-publisher NETWORK=mainnet PORT=4001 FROM_BLOCK=100
```

## Syncing Blocks Locally

To sync blockchain data locally, you need to run both the Publisher and Consumer services together. The Publisher fetches blocks from the Fuel node and publishes them to NATS, while the Consumer processes these blocks and stores them in the database.

Follow these steps to start syncing blocks locally:

1. Ensure all prerequisites are installed and running:

```bash
# Start required services
make start-docker

# Set up the database schema
make setup-db
```

2. Open two terminal tabs/windows

3. In the first terminal, start the Publisher:

```bash
make run-publisher
```

4. In the second terminal, start the Consumer:

```bash
make run-consumer
```

The Publisher will start subscribing to new blocks and publishing them to NATS, and the Consumer will process these blocks and store them in the database. This creates a complete local indexing system that can be used with the API and WebSocket services.

> [!Note]
> If you need to work with specific networks, you can use the appropriate commands such as `make run-publisher-mainnet-dev` and ensure both Publisher and Consumer are configured to work with the same network.

## Monitoring

The service exposes Prometheus metrics for monitoring performance and data flow.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../../LICENSE) file for details.
