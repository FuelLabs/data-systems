## 0.0.12 (2024-10-18)

### 🚀 Features

-   Add initial indexer project with SurrealDB (#244)

### 🐛 Fixes

-   Rollback fuel-core version to 0.38

## 0.0.11 (2024-10-15)

### 🚀 Features

-   Parallel publishing of multiple streams (optimization) (#240)
-   Publish predicates (#243)
-   Updated fuel-core and other deps (#247)
-   Add an example of streaming contract events. DS-64 (#224)
-   Publish scripts (#255)
-   Upgraded fuel-core to v0.38 (#258)
-   Upgraded fuel-core version to 0.40.0 (#262)

## 0.0.10 (2024-09-26)

### 🚀 Features

-   Add Inputs subjects and publisher logic (#202)
-   Publish receipts (#208)
-   Added prometheus and grafana & other implementations (#175)
-   Added nextest and upgraded crates (#218)
-   Publish outputs (#211)
-   Publish logs (#220)
-   Added examples folder with sample Rust projects (#219)
-   Added streaming utxos (#223)

## 0.0.9 (2024-09-09)

### 🐛 Fixes

-   Adjust information for release

## 0.0.8 (2024-09-06)

### 🐛 Fixes

-   Avoid using cached tx id (#200)
-   Inline public info (#201)

## 0.0.6 (2024-08-29)

### 🐛 Fixes

#### Documentation Update Release

This is a version bump to create a new release that includes updated documentation. No functional changes have been made to the codebase; this release is solely to publish the latest documentation updates.

## 0.0.5 (2024-08-29)

### 🐛 Fixes

#### Documentation Update Release

This is a version bump to create a new release that includes updated documentation. No functional changes have been made to the codebase; this release is solely to publish the latest documentation updates.

## 0.0.4 (2024-08-24)

### 🚀 Features

-   Add NATS POC version (#12)
-   Bootstrap the stream from Fuel nodes storage (#22)
-   Add nkey as auth mechanism (#32)
-   Add first version of streams-core crate (#61)
-   Add nats publishing deduplication logic and tests (#107)
-   Add Client (#114)
-   Data-parser implementation (#111)
-   Add Stream and Filter (#125)

### 🐛 Fixes

-   Use u32 for block height to avoid hex outputs in block subjects (#66)
-   Publish compact json rather than pretty json (#105)
-   Temporary solution for nats storage size (#120)
-   Block compress for bincode (#122)
