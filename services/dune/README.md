# Fuel Dune Service

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml)
[![Coverage](https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj)](https://codecov.io/gh/FuelLabs/data-systems)

## About

The Fuel Dune Service processes live and historical blockchain data from the Fuel network for analytics purposes, specifically for integration with [Dune Analytics](https://dune.com/). It transforms blockchain data into Avro format and uploads it to S3 buckets for subsequent analytics processing.

## Features

- **Data Processing**: Transforms blockchain data into analytics-friendly Avro format
- **AWS S3 Integration**: Uploads processed data to S3 buckets with configurable retry mechanisms
- **Schema Management**: Manages Avro schemas for different blockchain entities
- **Redis State Management**: Uses Redis to track processing state and avoid duplications
- **Historical Backfilling**: Supports processing historical data in addition to live data
- **Error Handling**: Robust error handling with retry mechanisms for AWS operations

## Architecture

The service consists of several key components:

- **Processor**: Core data transformation logic that converts blockchain data to Avro records
- **S3 Client**: Handles communication with AWS S3, including uploads and error handling
- **Schema Management**: Defines and manages Avro schemas for different data types
- **Redis Integration**: Manages processing state and deduplication
- **CLI Interface**: Command-line interface for configuring and running the service

### Data Flow

1. The service queries blockchain data from a database or message broker
2. Data is transformed into Avro format according to predefined schemas
3. Avro records are batched and uploaded to configured S3 buckets
4. Processing state is maintained in Redis to ensure no duplicate processing
5. Error handling and retry mechanisms ensure data reliability

## Getting Started

### Prerequisites

- [Rust toolchain](https://www.rust-lang.org/tools/install)
- AWS S3 bucket and credentials
- Redis instance
- PostgreSQL database with Fuel blockchain data
- [Docker](https://www.docker.com/get-started/) (optional)

### Environment Setup

Create a `.env` file by copying the example:

```bash
make create-env
```

This will include the necessary variables for database connection, AWS credentials, Redis connection, and processing configuration.

### Docker Dependencies

Start the required dependencies using Docker:

```bash
# Start Redis
make start-redis

# Start S3 (MinIO)
make start-s3

# Start PostgreSQL
make start-postgres
```

### Running the Service

The Dune service can be run using the Makefile commands:

```bash
# Run with default configuration
make run-dune

# Run in development mode
make run-dune-dev

# Run in profiling mode
make run-dune-profiling
```

### Additional Make Commands

```bash
# Start all dependencies
make start-docker

# Reset dependencies (removes volumes and restarts)
make reset-docker
```

## S3 Data Organization

Data in S3 is organized following a specific structure:

```
{bucket}/
├── blocks/
│   └── year=YYYY/month=MM/day=DD/hour=HH/
│       └── blocks_YYYYMMDD_HH_XXXXX.avro
├── transactions/
│   └── year=YYYY/month=MM/day=DD/hour=HH/
│       └── transactions_YYYYMMDD_HH_XXXXX.avro
├── receipts/
│   └── year=YYYY/month=MM/day=DD/hour=HH/
│       └── receipts_YYYYMMDD_HH_XXXXX.avro
...
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../../LICENSE) file for details.
