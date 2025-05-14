# Fuel WebSocket Service

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml)
[![Coverage](https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj)](https://codecov.io/gh/FuelLabs/data-systems)

## About

The Fuel WebSocket Service is a real-time data streaming service that consumes blockchain events from Fuel and makes them available to clients via WebSocket connections. It allows clients to subscribe to various blockchain events and receive updates as they occur, enabling real-time applications and dashboards.

> [!Note]
> To use the WebSocket service locally with real-time blockchain data, you first need to sync blocks using the Publisher and Consumer services. See the [Syncing Blocks Locally](../publisher/README.md#syncing-blocks-locally) section in the Publisher README for instructions.

## Features

- **WebSocket API**: Streaming interface for real-time blockchain data
- **Subscription Management**: Support for topic-based subscriptions
- **Message Consumption**: Consumes messages from a message broker (NATS)
- **Connection Management**: Handles client connections, disconnections, and reconnections
- **Authentication**: Optional API key-based authentication
- **Metrics Collection**: Monitors performance with Prometheus metrics

## Architecture

The service consists of several key components:

- **WebSocket Server**: Handles client connections and subscriptions
- **Message Handler**: Processes messages from the message broker
- **Subscription Manager**: Manages client subscriptions to different topics
- **Connection Manager**: Tracks active connections and handles lifecycle events
- **Authentication**: Verifies client credentials when required
- **Metrics Collection**: Tracks service performance and health

### Subscription Flow

1. Clients connect to the WebSocket server
2. Clients subscribe to specific topics (e.g., new blocks, transactions for an address)
3. The service subscribes to relevant topics in the message broker
4. When new data is available, it's forwarded to subscribed clients
5. Clients can unsubscribe or disconnect at any time

## Getting Started

### Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install)
- NATS server (connected to the Publisher service)
- [Docker](https://www.docker.com/get-started/) (optional)

### Environment Setup

Create a `.env` file by copying the example:

```bash
make create-env
```

This will include the necessary variables for NATS connection and service configuration.

### Dependencies

Start the required Docker dependencies:

```bash
# Start NATS server
make start-nats
```

### Running the Service

The WebSocket service can be run using the Makefile commands:

```bash
# Run with default configuration (testnet)
make run-webserver

# Run against mainnet in development mode
make run-webserver-mainnet-dev

# Run against testnet in development mode
make run-webserver-testnet-dev

# Run against mainnet in profiling mode
make run-webserver-mainnet-profiling

# Run against testnet in profiling mode
make run-webserver-testnet-profiling

# Run with custom port
make run-webserver PORT=8080
```

## WebSocket API

### Connection

Connect to the WebSocket endpoint:

```
ws://localhost:9003/ws
```

### Subscription Format

Send a JSON message to subscribe to topics:

```json
{
    "deliver_policy": "new",
    "subscribe": [
        {
            "id": "blocks"
        },
        {
            "id": "transactions"
        }
    ]
}
```

Available topic IDs:

- `blocks`: All new blocks
- `transactions`: All new transactions
- `accounts/{address}`: Activity for a specific account
- `contracts/{id}`: Activity for a specific contract
- `tx/{hash}`: Updates for a specific transaction

### Unsubscription Format

Send a JSON message to unsubscribe:

```json
{
    "deliver_policy": "new",
    "unsubscribe": [
        {
            "id": "blocks"
        }
    ]
}
```

### Example Client Code

```javascript
const ws = new WebSocket("ws://localhost:9003/ws");

ws.onopen = () => {
    // Subscribe to new blocks
    ws.send(
        JSON.stringify({
            deliver_policy: "new",
            subscribe: [
                {
                    id: "blocks",
                },
            ],
        }),
    );
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log("Received data:", data);
};

// To unsubscribe
function unsubscribe() {
    ws.send(
        JSON.stringify({
            deliver_policy: "new",
            unsubscribe: [
                {
                    id: "blocks",
                },
            ],
        }),
    );
}
```

## Monitoring

The service exposes Prometheus metrics for monitoring performance and connection statistics.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../../LICENSE) file for details.
