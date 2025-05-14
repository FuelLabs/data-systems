# Fuel Consumer Service

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml)
[![Coverage](https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj)](https://codecov.io/gh/FuelLabs/data-systems)

## About

The Fuel Consumer Service consumes blockchain data published by the Publisher service, processes it, and stores it in a database for indexing and querying. It acts as the data processing and persistence layer in the Fuel data systems architecture.

## Features

- **Message Consumption**: Consumes blockchain data from a message broker (NATS)
- **Block Processing**: Processes blocks, transactions, receipts, and other blockchain entities
- **Database Persistence**: Stores processed data in a PostgreSQL database
- **Retry Mechanism**: Handles failures with configurable retry policies
- **Metrics Collection**: Monitors performance with Prometheus metrics
- **HTTP Server**: Optional HTTP server for health checks and metrics

## Architecture

The service consists of several key components:

- **Block Executor**: Processes blockchain blocks and their components
- **Block Event Executor**: Handles blockchain events and transforms them into database records
- **Retry Handler**: Manages retries for failed operations
- **Server**: Optional HTTP server for health checks and metrics
- **Metrics Collection**: Tracks processing performance and health

### Data Flow

1. The service connects to a NATS message broker and subscribes to block topics
2. Each received block is processed and broken down into its components (transactions, receipts, etc.)
3. Data is transformed into database records and stored in a PostgreSQL database
4. Failed operations are retried according to configurable policies
5. Metrics are collected throughout the process for monitoring

## Getting Started

### Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install)
- PostgreSQL database
- NATS server (connected to the Publisher service)
- [Docker](https://www.docker.com/get-started/) (optional)

### Environment Setup

Create a `.env` file by copying the example:

```bash
make create-env
```

This will include the necessary variables for database connection, NATS connection, and service configuration.

### Dependencies

Start the required Docker dependencies:

```bash
# Start NATS server
make start-nats

# Start PostgreSQL database
make start-postgres

# Set up the database schema
make setup-db
```

### Running the Service

The Consumer service can be run using the Makefile commands:

```bash
# Run with default configuration
make run-consumer

# Run with custom NATS URL
make run-consumer NATS_URL=nats://localhost:4222

# Run with custom port
make run-consumer PORT=8080
```

## Syncing Blocks Locally

The Consumer service must work together with the Publisher service to sync blockchain data locally. The Publisher fetches blocks from the blockchain and the Consumer processes and stores them in the database.

For detailed instructions on setting up both services together, please see the [Syncing Blocks Locally](../publisher/README.md#syncing-blocks-locally) section in the Publisher README.

> [!Important]
> Always ensure the Consumer is running when the Publisher is active to prevent the build-up of unprocessed messages in the NATS broker.

## Monitoring

The service exposes Prometheus metrics for monitoring performance and processing statistics.

## Database Schema

The service creates and maintains a database schema for storing blockchain data:

- `blocks`: Information about blockchain blocks
- `transactions`: Transaction data
- `receipts`: Transaction receipts
- `inputs`: Transaction inputs
- `outputs`: Transaction outputs
- `utxos`: Unspent transaction outputs
- And more related tables

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../../LICENSE) file for details.
