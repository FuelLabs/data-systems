# Fuel Web Utils

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

## About

`fuel-web-utils` is a comprehensive library providing reusable components and utilities for building web services in the Fuel ecosystem. It offers a collection of tools for creating robust, scalable, and secure web applications with minimal boilerplate code.

## Features

- **Server Framework**: Easy-to-use builder pattern for creating HTTP servers with sensible defaults
- **Router Builder**: Simplified API for constructing route hierarchies with prefix support
- **API Key Management**: Complete system for API key authentication, authorization, and rate limiting
- **Telemetry**: Integrated metrics collection, logging, and monitoring capabilities
- **Graceful Shutdown**: Utilities for handling application shutdown with proper resource cleanup
- **Tracing**: Preconfigured tracing setup for improved observability
- **HTTP Components**: Common HTTP handlers, middleware, and utilities

## Architecture

The crate is organized into several modules:

- **server**: Server construction and lifecycle management

    - `server_builder.rs`: Builder pattern for HTTP server configuration
    - `state.rs`: Application state management
    - `http/`: HTTP-specific components and handlers

- **router_builder**: Tools for constructing route hierarchies

    - Path normalization and prefix handling
    - Type-safe route registration

- **api_key**: Complete API key management system

    - Key generation and validation
    - Role-based access control
    - Rate limiting
    - Storage abstractions

- **telemetry**: Observability components

    - Prometheus metrics integration
    - ElasticSearch logging
    - System resource monitoring
    - Runtime performance tracking

- **shutdown**: Graceful shutdown utilities

    - Signal handling
    - Resource cleanup coordination
    - Timeout management

- **tracing**: Preconfigured tracing setup
    - Environment-based configuration
    - Structured logging

## Usage

### Server Setup

```rust
use fuel_web_utils::server::{ServerBuilder, state::StateProvider};

#[derive(Clone)]
struct AppState {
    // Your application state here
}

impl StateProvider for AppState {}

async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    fuel_web_utils::tracing::init_tracing()?;

    // Create application state
    let state = AppState {};

    // Build and run server
    let server = ServerBuilder::build(&state, 8080);
    server.run().await
}
```

### Router Construction

```rust
use axum::routing::get;
use fuel_web_utils::router_builder::RouterBuilder;

async fn handler() -> &'static str {
    "Hello, world!"
}

// Create a router with prefix
let (path, router) = RouterBuilder::<AppState>::new("/items")
    .with_prefix("/api/v1")
    .root(get(handler))
    .related("/count", get(handler))
    .build();

// Result: "/api/v1/items" with routes for "/" and "/count"
```

### API Key Authentication

```rust
use fuel_web_utils::api_key::{ApiKeyManager, ApiKeyProps, Role};

// Create API key manager
let manager = ApiKeyManager::new(storage).await?;

// Create a new API key
let props = ApiKeyProps::new("service-name", vec![Role::Admin]);
let api_key = manager.create_key(props).await?;

// Validate an API key
let validation = manager.validate_key("api-key-value").await?;
```

### Telemetry Setup

```rust
use fuel_web_utils::telemetry::{Telemetry, metrics::AppMetrics};

// Create custom metrics
struct MyMetrics;

impl TelemetryMetrics for MyMetrics {
    // Implementation details
}

// Initialize telemetry
let telemetry = Telemetry::new(Some(MyMetrics::new())).await?;
telemetry.start().await?;

// Log information
telemetry.log_info("Application started");
```

### Graceful Shutdown

```rust
use std::sync::Arc;
use fuel_web_utils::shutdown::ShutdownController;

// Create shutdown controller
let shutdown = Arc::new(ShutdownController::new());

// Spawn signal handler
let shutdown_clone = shutdown.clone().spawn_signal_handler();

// Wait for shutdown signal
shutdown.wait_for_shutdown().await;
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.
