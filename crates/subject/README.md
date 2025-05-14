<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams Subject</h3>
    <p align="center">
        Macros and utilities for implementing subject-based messaging in the Fuel Data Systems project
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-streams-subject" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-streams-subject">
        </a>
        <a href="https://crates.io/crates/fuel-streams-subject" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-streams-subject?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-streams-subject/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-streams-subject/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-streams-subject/">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

Fuel Streams Subject provides a powerful framework for defining, parsing, and working with message subjects in the Fuel Data Systems project. It enables structured communication between services using a hierarchical subject system, with automatic code generation through procedural macros.

> [!NOTE]
> This crate is specifically designed for the Fuel Data Systems project, and is not intended for general use outside of the project.

## 🚀 Features

- **Subject Definition**: Define message subjects with a structured format
- **Automatic Parsing**: Convert between structured data and string subjects
- **Builder Pattern**: Create subjects with a fluent builder API
- **SQL Generation**: Automatically generate SQL WHERE and SELECT clauses
- **Schema Generation**: Generate JSON schema for subjects
- **Payload Conversion**: Convert between subjects and serializable payloads
- **Field Descriptions**: Add documentation to subject fields
- **Field Aliases**: Support alternative field names for flexibility

## 🛠️ Usage

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
fuel-streams-subject = "*"
```

### Basic Subject Definition

```rust
use fuel_streams_subject::subject::*;
use serde::{Serialize, Deserialize};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "blocks")]
#[subject(entity = "Block")]
#[subject(query_all = "blocks.>")]
#[subject(format = "blocks.{height}.{hash}")]
struct BlockSubject {
    pub height: Option<u64>,
    pub hash: Option<String>,
}

// Create a subject
let subject = BlockSubject {
    height: Some(123),
    hash: Some("0xabc123".to_string()),
};

// Convert to string representation
assert_eq!(subject.parse(), "blocks.123.0xabc123");

// Create with builder pattern
let subject = BlockSubject::new()
    .with_height(Some(123))
    .with_hash(Some("0xabc123".to_string()));

assert_eq!(subject.parse(), "blocks.123.0xabc123");

// Create a wildcard subject for all blocks
let all_blocks = BlockSubject::new();
assert_eq!(all_blocks.parse(), "blocks.>");
```

### SQL Query Generation

The Subject derive macro can automatically generate SQL queries based on the subject fields:

```rust
use fuel_streams_subject::subject::*;
use serde::{Serialize, Deserialize};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "transactions")]
#[subject(entity = "Transaction")]
#[subject(query_all = "transactions.>")]
#[subject(format = "transactions.{block_height}.{tx_id}")]
#[subject(custom_where = "deleted_at IS NULL")]
struct TransactionSubject {
    #[subject(sql_column = "block_height")]
    pub block_height: Option<u64>,

    #[subject(sql_column = "tx_id")]
    pub tx_id: Option<String>,
}

// Create a subject with specific block height
let subject = TransactionSubject {
    block_height: Some(123),
    tx_id: None,
};

// Generate SQL WHERE clause
assert_eq!(
    subject.to_sql_where(),
    Some("block_height = '123' AND deleted_at IS NULL".to_string())
);

// Generate SQL SELECT clause
assert_eq!(
    subject.to_sql_select(),
    Some("block_height".to_string())
);
```

### Schema Generation

Subjects can generate schema information for documentation and validation:

```rust
use fuel_streams_subject::subject::*;
use serde::{Serialize, Deserialize};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.>")]
#[subject(format = "receipts.{tx_id}.{index}")]
struct ReceiptSubject {
    #[subject(description = "Transaction ID")]
    pub tx_id: Option<String>,

    #[subject(description = "Receipt index in the transaction")]
    pub index: Option<u32>,
}

// Get schema information
let subject = ReceiptSubject::new();
let schema = subject.schema();

// Convert schema to JSON
let schema_json = schema.to_json();
println!("{}", schema_json);
```

### Payload Conversion

Subjects can be converted to and from serializable payloads:

```rust
use fuel_streams_subject::subject::*;
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "events")]
#[subject(entity = "Event")]
#[subject(query_all = "events.>")]
#[subject(format = "events.{contract_id}.{event_type}")]
struct EventSubject {
    pub contract_id: Option<String>,
    pub event_type: Option<String>,
}

// Create a subject
let subject = EventSubject {
    contract_id: Some("0xabc123".to_string()),
    event_type: Some("Transfer".to_string()),
};

// Convert to payload
let payload = subject.to_payload();
assert_eq!(payload.subject, "events");
assert_eq!(
    payload.params,
    json!({"contract_id": "0xabc123", "event_type": "Transfer"})
);

// Convert back to subject
let reconstructed = EventSubject::try_from(payload).unwrap();
assert_eq!(reconstructed.contract_id, subject.contract_id);
assert_eq!(reconstructed.event_type, subject.event_type);
```

### Field Aliases

You can define aliases for fields to support alternative field names:

```rust
use fuel_streams_subject::subject::*;
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "accounts")]
#[subject(entity = "Account")]
#[subject(query_all = "accounts.>")]
#[subject(format = "accounts.{address}")]
struct AccountSubject {
    #[subject(alias = "addr")]
    pub address: Option<String>,
}

// Create from payload with regular field name
let payload1 = SubjectPayload {
    subject: "accounts".to_string(),
    params: json!({"address": "0xabc123"}),
};
let subject1 = AccountSubject::try_from(payload1).unwrap();
assert_eq!(subject1.address, Some("0xabc123".to_string()));

// Create from payload with alias
let payload2 = SubjectPayload {
    subject: "accounts".to_string(),
    params: json!({"addr": "0xabc123"}),
};
let subject2 = AccountSubject::try_from(payload2).unwrap();
assert_eq!(subject2.address, Some("0xabc123".to_string()));
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
