# Subjects Schema Generator

This Rust script generates a JSON schema that defines the subject definitions used in the Fuel Streams SDK.

## Purpose

The schema generator creates a standardized definition of all possible subjects that can be used for subscribing to Fuel blockchain events. This schema is used to automate the generation of subject definitions in other SDKs, particularly in the JavaScript implementation of Fuel Streams.

## How It Works

The script uses the subject definitions from the `fuel-streams-domains` crate and the schema generation capabilities provided by the `fuel-streams-subject` crate. It follows these steps:

1. Imports all subject types defined in the domains crate (blocks, transactions, inputs, outputs, etc.)
2. For each subject type, calls the `.schema()` method provided by the `Subject` derive macro
3. Combines all subject schemas into a comprehensive JSON schema
4. Properly handles variant relationships for complex subjects (like inputs and outputs)
5. Outputs the schema to a JSON file

Here's a simplified example of how the script generates schemas:

```rust
// Import subject types from domains
use fuel_streams_domains::{
    blocks::subjects::*,
    transactions::subjects::*,
    // ... other imports
};

// Use the Subject trait from the subject crate
use fuel_streams_subject::subject::*;

// Generate schema for each subject
let block_schema = BlocksSubject::new().schema();
let transaction_schema = TransactionsSubject::new().schema();
// ... other schemas

// Combine into final schema
let final_schema = IndexMap::from([
    ("blocks".to_string(), block_schema),
    ("transactions".to_string(), transaction_schema),
    // ... other entries
]);
```

## Usage

The generated schema is used by the [Fuel Streams JS SDK](https://github.com/FuelLabs/fuel-streams-js/blob/main/packages/fuel-streams/src/subjects-def.ts) to create TypeScript definitions that maintain consistency with the Rust implementation.

To regenerate the schema:

```bash
cargo run --package subjects-schema
```

This will create or update the `schema.json` file in this directory.

## Benefits

- **Consistency**: Ensures the subject definitions are identical across different SDK implementations
- **Automation**: Reduces manual work and potential for errors when updating subject definitions
- **Documentation**: Provides a clear reference for all available subjects and their structure
- **Type Safety**: The generated schemas help ensure type safety in TypeScript implementations
