# Changelog

## [0.0.28](https://github.com/FuelLabs/data-systems/compare/v0.0.27...v0.0.28) - 2025-03-13





### 🐛 Bug Fixes

- *(fuel-streams)*: Handle bynari message receiving when sequence ([6a27bcd](https://github.com/FuelLabs/data-systems/commit/6a27bcd7b538129fe2e93fa18bb2da9e4edec00a)  @pedronauck)

### 💪🏼 Contributors

* @pedronauck


Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---
## [0.0.27](https://github.com/FuelLabs/data-systems/compare/v0.0.26...v0.0.27) - 2025-03-13

### ⭐ Features

- _(repo)_: Added open-api documentation ([#428](https://github.com/FuelLabs/data-systems/pull/428)) ([9883a75](https://github.com/FuelLabs/data-systems/commit/9883a755c51027b6faeb33d76b8d4dc683f25ae6) @0xterminator)

### 🔄 Refactor

- _(fuel-streams-store)_: Improve historical data query performance ([bd81409](https://github.com/FuelLabs/data-systems/commit/bd814098c6c02ef9cb8c42bcc871b5b22cfcc9ab) @pedronauck)

- _(sv-webserver)_: Improve performance on subscribing ([#435](https://github.com/FuelLabs/data-systems/pull/435)) ([2a35a17](https://github.com/FuelLabs/data-systems/commit/2a35a17eb55660efd1f80a59311edda3e150274d) @pedronauck)

### 🏗️ Build

- _(repo)_: Update rust to v1.85.0 ([90e5425](https://github.com/FuelLabs/data-systems/commit/90e5425d851460b25a92b8fb9b3cee0ab41c20c4) @pedronauck)

- _(repo)_: Adjust Cargo.toml dependencies ([54b7c86](https://github.com/FuelLabs/data-systems/commit/54b7c863d1bb5dd4c48c04603e5ef795b86eabcd) @pedronauck)

### 💪🏼 Contributors

- @pedronauck
- @0xterminator

Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---

## [0.0.26](https://github.com/FuelLabs/data-systems/compare/v0.0.25...v0.0.26) - 2025-03-11

### ⭐ Features

- _(repo)_: Add timestamps to all database entities ([#422](https://github.com/FuelLabs/data-systems/pull/422)) ([fac3af6](https://github.com/FuelLabs/data-systems/commit/fac3af66f2a95da16be0a5865743cabffb88066a) @pedronauck)

- _(repo)_: Added open-api documentation ([#428](https://github.com/FuelLabs/data-systems/pull/428)) ([9883a75](https://github.com/FuelLabs/data-systems/commit/9883a755c51027b6faeb33d76b8d4dc683f25ae6) @0xterminator)

- _(repo)_: Add historical limit on API keys ([#423](https://github.com/FuelLabs/data-systems/pull/423)) ([2695bd4](https://github.com/FuelLabs/data-systems/commit/2695bd4ee90dbd134f910b32483ecbf7b1c3f7ac) @pedronauck)

- _(repo)_: Add more granularity within API key rules ([#420](https://github.com/FuelLabs/data-systems/pull/420)) ([d94183d](https://github.com/FuelLabs/data-systems/commit/d94183d107b9b71cbcdd48dc90a61687980de7f0) @pedronauck)

- _(repo)_: Updated fuel-core version ([#417](https://github.com/FuelLabs/data-systems/pull/417)) ([a524a41](https://github.com/FuelLabs/data-systems/commit/a524a41a40edc9705b39e4b9a25de52d21ef541d) @0xterminator)

- _(repo)_: Add pointers inside the StreamResponse ([#404](https://github.com/FuelLabs/data-systems/pull/404)) ([efd339c](https://github.com/FuelLabs/data-systems/commit/efd339cb89dbd82bb42689c62fadb8ab2f3c8f82) @pedronauck)

### 🐛 Bug Fixes

- _(repo)_: Create wrapped int types to fix serialization ([#407](https://github.com/FuelLabs/data-systems/pull/407)) ([a5ad221](https://github.com/FuelLabs/data-systems/commit/a5ad2210aeb9b4fc91902b2b7205863b71b2ded0) @pedronauck)

### 💪🏼 Contributors

- @pedronauck
- @0xterminator

Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---

## [0.0.25](https://github.com/FuelLabs/data-systems/compare/v0.0.24...v0.0.25) - 2025-02-12

### ⭐ Features

- _(repo)_: Add support to subscribe to multiple subjects at once (#402) ([aefdd0c](https://github.com/FuelLabs/data-systems/commit/aefdd0c80429bb40a1b240f49cef0f64dc5145ad) @pedronauck)

### 🐛 Bug Fixes

- _(fuel-streams)_: Network and connection definition ([abbb143](https://github.com/FuelLabs/data-systems/commit/abbb1439366497fa0ca37d2e9c68ce9a6dbba477) @pedronauck)

- _(webserver)_: Handle unsubscribe messages ([1d2da62](https://github.com/FuelLabs/data-systems/commit/1d2da62a8e51fc71699daaa2906bf1d81cac2945) @pedronauck)

- _(domains)_: Implement custom deserializer for transaction ([f913749](https://github.com/FuelLabs/data-systems/commit/f9137493f9d6c932004889754e4bc2eff26f3e4b) @pedronauck)

- _(repo)_: Rollback fuel-core to v0.40.3 ([014ac20](https://github.com/FuelLabs/data-systems/commit/014ac2015befafc9364fb5ed5737c97efed21a0c) @pedronauck)

### 🏗️ Build

- _(repo)_: Change from PNPM to Bun (#400) ([6e23dfe](https://github.com/FuelLabs/data-systems/commit/6e23dfe45119bc66081da069348b1cc3cc97219f) @pedronauck)

### 💪🏼 Contributors

- @pedronauck

Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---

## [0.0.24](https://github.com/FuelLabs/data-systems/compare/v0.0.23...v0.0.24) - 2025-02-04

### ⭐ Features

- _(repo)_: Added API key rate limiting on active ws sessions (#398) ([5f02830](https://github.com/FuelLabs/data-systems/commit/5f02830cf76051f4f8395cd84874a7a3586db8dd) @0xterminator)

### 🐛 Bug Fixes

- _(repo)_: Bump fuel-core to v0.40.4 ([49ee9e9](https://github.com/FuelLabs/data-systems/commit/49ee9e9d366bc5c1f79a5ff360b3ebf6f29334f8) @pedronauck)

### 📚 Documentation

- _(repo)_: Change Fuel logo on READMEs ([9b50b8f](https://github.com/FuelLabs/data-systems/commit/9b50b8f207296edbabfca80d3e25f0776a3e882c) @pedronauck)

### 💪🏼 Contributors

- @pedronauck
- @0xterminator

Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---

## [0.0.23](https://github.com/FuelLabs/data-systems/compare/v0.0.22...v0.0.23) - 2025-01-28

### ⭐ Features

- **(subject)**: Add description field on subjects ([0a4ccdd](https://github.com/FuelLabs/data-systems/commit/0a4ccdd875076390b99922e94ba93974605e34e3) by @pedronauck)

### 🐛 Bug Fixes

- **(sv-publisher)**: Recover mechanism for tx status none ([1b1083d](https://github.com/FuelLabs/data-systems/commit/1b1083dbda9791e27d2e00a9e16b91662dbf86e7) by @pedronauck)

### 🔄 Refactor

- **(repo)**: Move services to a specific folder ([d62e206](https://github.com/FuelLabs/data-systems/commit/d62e20688490b1d99427c111a7e7d0a3896308e0) by @pedronauck)
- **(subject)**: Use IndexMap when building subjects schema ([90e9866](https://github.com/FuelLabs/data-systems/commit/90e986686c8802132fa643f2ff77c04108a31e2d) by @pedronauck)

### 🏗️ Build

- **(repo)**: Fix docs.rs build generation ([b93f057](https://github.com/FuelLabs/data-systems/commit/b93f0578914c370e14606cbc855feeba396c694d) by @pedronauck)
- **(repo)**: Change mainnet endpoint ([097201f](https://github.com/FuelLabs/data-systems/commit/097201f51382e640ea42cd0ecfe0ac0d2c275da4) by @pedronauck)

### 💪🏼 Contributors

- @pedronauck

Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---

## 0.0.18 (2025-01-24)

### 🐛 Fixes

- Patched unstable deps (#388)

## 0.0.17 (2025-01-24)

### 🚀 Features

- Added web-utils crate and added telemetry to all svc (#373)
- Database integration (#374)
- Added api key auth (#377)
- Add db transaction for each block insertion (#378)
- Added load-tester and benchmarks (#379)

### 🐛 Fixes

- Publish in order strictly (#372)
- Never panic when sending wrong data format (#380)
- Adjust historical streaming to stream block by block
- Fix find_by_many_subject query slowness
- Filtering on inputs, receipts and outputs
- Db connection pool timeout
- Add retry mechanism for DB transactions
- Connection opts for specific databases (#384)
- Adjust int types to fit with Aurora (#385)
- Parsing user_id in a valid format

## 0.0.16 (2024-12-28)

### 🚀 Features

- Add TLS configuration for WebServer (#371)
- Add subject at response payload

### 🐛 Fixes

- Add ack_policy for historical data consumers

## 0.0.15 (2024-12-27)

### 🐛 Fixes

- Use JWT on query string instead of header (#367)

## 0.0.14 (2024-12-27)

### 🚀 Features

- Updated crates and npm packages (#341)
- Add historical data with S3 and Webserver

### 🐛 Fixes

- Resolve block publication race conditions and synchronization issues (#329)
- Fixed telemetry envs and port (#330)
- Mainnet DNS value for NATS (#339)
- Rollback NATS_URL for Publisher (#343)
- Fix last published logic to honor retention policy (#344)
- Simplify and test BlocksStream for old and new blocks (#346)
- Rollback fuel-core to v0.40.0
- Allow healthcheck while awaiting relayer node sync (#347)
- Update cargo.lock
- Using fuel-core from branch release/v0.40.3
- Solving publisher slowness (#352)
- Adjust DeliverPolicy to receive from WebSocket
- S3 payload and encoding (#366)

## 0.0.13 (2024-11-28)

### 🚀 Features

- Introduce FuelStreams Types (#320)
- Ensure types are compatible with TS interfaces (#323)

### 🐛 Fixes

- Include `transaction_ids` in blocks (#324)
- Adjust types for better TS SDK compatibility (#325)

## 0.0.12 (2024-11-21)

### 🚀 Features

- Add initial indexer project with SurrealDB (#244)
- Republish last block to ensure no missing data (#281)
- Added publisher logging (#280)
- Upgraded to latest nightly rust version (#304)
- Introduce Telemetry (#299)
- Added entries max retention time (#309)
- Restructured publisher into smaller modules (#310)
- Added local websocket for nats (#315)

### 🐛 Fixes

- Rollback fuel-core version to 0.38
- Added panic hooks and reworked graceful shutdown (#278)
- Missing by id subjects (#282)
- Enable publish multi ById subjects (#294)
- Fixed profiling script (#311)

## 0.0.11 (2024-10-15)

### 🚀 Features

- Parallel publishing of multiple streams (optimization) (#240)
- Publish predicates (#243)
- Updated fuel-core and other deps (#247)
- Add an example of streaming contract events. DS-64 (#224)
- Publish scripts (#255)
- Upgraded fuel-core to v0.38 (#258)
- Upgraded fuel-core version to 0.40.0 (#262)

## 0.0.10 (2024-09-26)

### 🚀 Features

- Add Inputs subjects and publisher logic (#202)
- Publish receipts (#208)
- Added prometheus and grafana & other implementations (#175)
- Added nextest and upgraded crates (#218)
- Publish outputs (#211)
- Publish logs (#220)
- Added examples folder with sample Rust projects (#219)
- Added streaming utxos (#223)

## 0.0.9 (2024-09-09)

### 🐛 Fixes

- Adjust information for release

## 0.0.8 (2024-09-06)

### 🐛 Fixes

- Avoid using cached tx id (#200)
- Inline public info (#201)

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

- Add NATS POC version (#12)
- Bootstrap the stream from Fuel nodes storage (#22)
- Add nkey as auth mechanism (#32)
- Add first version of streams-core crate (#61)
- Add nats publishing deduplication logic and tests (#107)
- Add Client (#114)
- Data-parser implementation (#111)
- Add Stream and Filter (#125)

### 🐛 Fixes

- Use u32 for block height to avoid hex outputs in block subjects (#66)
- Publish compact json rather than pretty json (#105)
- Temporary solution for nats storage size (#120)
- Block compress for bincode (#122)
