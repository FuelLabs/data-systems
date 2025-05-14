# Fuel API Service

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml)
[![Coverage](https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj)](https://codecov.io/gh/FuelLabs/data-systems)

## About

The Fuel API Service provides a comprehensive REST API for retrieving blockchain data from the Fuel network. It exposes endpoints for querying blocks, transactions, receipts, accounts, contracts, and more from an indexed database.

> **Official API Documentation (Mainnet)**: https://api-rest-mainnet.fuel.network/swagger-ui

> [!Note]
> To use the API service locally with blockchain data, you first need to sync blocks using the Publisher and Consumer services. See the [Syncing Blocks Locally](../publisher/README.md#syncing-blocks-locally) section in the Publisher README for instructions.

## Features

- **Comprehensive Data Access**: Query blocks, transactions, receipts, accounts, contracts, inputs, outputs, UTXOs, predicates, and messages
- **API Key Authentication**: Secure access with API key-based authentication and authorization
- **Rate Limiting**: Built-in rate limiting to prevent abuse
- **OpenAPI Documentation**: Interactive Swagger UI documentation at `/swagger-ui`
- **Metrics Endpoint**: Prometheus metrics for monitoring at `/metrics`
- **Health Checks**: Health monitoring at `/health`

## Architecture

The service implements a clean architecture with:

- **Server Module**: HTTP server setup, routing, and middleware
- **Handlers**: Business logic for each endpoint
- **State Management**: Application state with database connections and API key management
- **Error Handling**: Standardized error responses
- **Metrics Collection**: Performance and usage metrics

### API Endpoints

The service provides the following endpoint categories:

- `/api/v1/blocks`: Block queries by height with associated data
- `/api/v1/transactions`: Transaction queries by ID with associated data
- `/api/v1/accounts`: Account-specific queries for transactions, inputs, outputs, UTXOs, and receipts
- `/api/v1/contracts`: Contract-specific queries for transactions, inputs, outputs, UTXOs, and receipts
- `/api/v1/inputs`: Query different types of transaction inputs
- `/api/v1/outputs`: Query different types of transaction outputs
- `/api/v1/receipts`: Query transaction receipts by type
- `/api/v1/utxos`: Query unspent transaction outputs
- `/api/v1/predicates`: Query predicate information
- `/api/v1/messages`: Query messages
- `/api/v1/keys`: API key management endpoints

## Getting Started

### Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install)
- PostgreSQL database
- [Docker](https://www.docker.com/get-started/) (optional)

### Environment Setup

Create a `.env` file by copying the example:

```bash
make create-env
```

This will include the necessary variables for connecting to the database and configuring the API service.

### Running the Service

The API service can be run using the Makefile commands:

```bash
# Run with default configuration (testnet)
make run-api

# Run against mainnet in development mode
make run-api-mainnet-dev

# Run against testnet in development mode
make run-api-testnet-dev

# Run with custom port
make run-api PORT=8080
```

### Database Setup

Before running the API service, make sure the database is set up:

```bash
# Start required Docker containers
make start-docker

# Set up the database schema
make setup-db
```

## Usage Examples

### Query Blocks

```bash
curl -X GET "http://localhost:9004/api/v1/blocks?limit=10" \
    -H "Authorization: Bearer your-api-key"
```

### Query Transactions for an Account

```bash
curl -X GET "http://localhost:9004/api/v1/accounts/fuel1unc0unf95xvtvr4f6ayz23gkwcq6qsckvraghuwjgwn9tqz9qf9qf8qjs8/transactions" \
    -H "Authorization: Bearer your-api-key"
```

### Query Contract Receipts

```bash
curl -X GET "http://localhost:9004/api/v1/contracts/0x123.../receipts" \
    -H "Authorization: Bearer your-api-key"
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../../LICENSE) file for details.
