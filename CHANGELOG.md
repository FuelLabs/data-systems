# Changelog

## [0.0.30](https://github.com/FuelLabs/data-systems/compare/v0.0.29...v0.0.30) - 2025-09-10

### ⭐ Features

- _(sv-consumer)_: Introduce concurrent tasks configuration ([#483](https://github.com/FuelLabs/data-systems/pull/483)) ([f395620](https://github.com/FuelLabs/data-systems/commit/f395620b3d2aee8b251959dcb07766b5b0ed83be) @luizstacio)

- _(repo)_: Add messages ([#465](https://github.com/FuelLabs/data-systems/pull/465)) ([7a09099](https://github.com/FuelLabs/data-systems/commit/7a09099d2ee49a82c4dad6b1567df0fd373969ce) @pedronauck)

- _(repo)_: Limit queries to a smaller time range and number of items returned ([#495](https://github.com/FuelLabs/data-systems/pull/495)) ([a25fc1e](https://github.com/FuelLabs/data-systems/commit/a25fc1e4e8795b3cc4312a36774817a4a8c9fa72) @luizstacio)

- _(repo)_: Add block_time to predicate_transactions ([#492](https://github.com/FuelLabs/data-systems/pull/492)) ([63a2925](https://github.com/FuelLabs/data-systems/commit/63a2925d7817998f4efd58fe06dd6ca87f4c9965) @luizstacio)

- _(repo)_: Upgrade fuel-core version to 0.44 ([#491](https://github.com/FuelLabs/data-systems/pull/491)) ([a919e1c](https://github.com/FuelLabs/data-systems/commit/a919e1c1d6da8324cebe9ff353fbfefbfa01ed33) @luizstacio)

- _(sv-api)_: Add read-only database configuration ([#485](https://github.com/FuelLabs/data-systems/pull/485)) ([a00641c](https://github.com/FuelLabs/data-systems/commit/a00641c346e316e649a6acc9304f3eba3c18b7a2) @luizstacio)

### 🐛 Bug Fixes

- _(repo)_: Dune integration ([f487ceb](https://github.com/FuelLabs/data-systems/commit/f487cebfd873986f7312dd0d72d46cba47b2a9fb) @pedronauck)

- _(fuel-streams-domains)_: Adjust BlockHeaderVersion serialization ([7a421b7](https://github.com/FuelLabs/data-systems/commit/7a421b7dda0915d81cded4d9cb023614106efb89) @pedronauck)

- _(fuel-streams-domains)_: BlockVersion serde configuration ([b99705e](https://github.com/FuelLabs/data-systems/commit/b99705ea324a0cf98d9f02801ef3db6edc9ac199) @pedronauck)

- _(fuel-streams-domains)_: Improve accounts txs query performance ([#462](https://github.com/FuelLabs/data-systems/pull/462)) ([e2fb66e](https://github.com/FuelLabs/data-systems/commit/e2fb66e0ff6029e06bc9c53552db5ad1ec5039c3) @pedronauck)

- _(sv-publisher)_: Restore tx_pointer for transactions ([41c4cd8](https://github.com/FuelLabs/data-systems/commit/41c4cd8c56f916dbf8ad5acd379186a196f604d4) @pedronauck)

- _(repo)_: Change metrics endpoints to return plain text ([#496](https://github.com/FuelLabs/data-systems/pull/496)) ([ff4d435](https://github.com/FuelLabs/data-systems/commit/ff4d435edab7492d6b50c48ab1c5615d02077690) @luizstacio)

- _(fuel-web-utils)_: Load api-keys on demand from database ([#471](https://github.com/FuelLabs/data-systems/pull/471)) ([554ecd3](https://github.com/FuelLabs/data-systems/commit/554ecd3cd6765ca8ea6528b16c2d9142a30120d6) @luizstacio)

- _(repo)_: Builder subscriptions count error ([b5a179f](https://github.com/FuelLabs/data-systems/commit/b5a179ffa508af512e129e2312d3b0d3dd91a227) @pedronauck)

### 🔄 Refactor

- _(sv-dune)_: Improve Avro types ([#460](https://github.com/FuelLabs/data-systems/pull/460)) ([10a27dd](https://github.com/FuelLabs/data-systems/commit/10a27dd8dee72adc8e89568c7e0fcefa39246dc4) @pedronauck)

### 📚 Documentation

- _(repo)_: Improve docs on core and data-parser ([2d6401c](https://github.com/FuelLabs/data-systems/commit/2d6401c96ef55c7555cc78ddd1296b1acfd25d81) @pedronauck)

- _(repo)_: Improve project docs ([4ead022](https://github.com/FuelLabs/data-systems/commit/4ead022c1d4c10db4174a512d4b1864b18b0e56d) @pedronauck)

- _(fuel-streams-core)_: Fix code example ([0729afb](https://github.com/FuelLabs/data-systems/commit/0729afbc3c2f3adff3ecd8b9ed7f484d4028008c) @pedronauck)

### 🧪 Testing

- _(repo)_: Fix wrong tests ([a70976a](https://github.com/FuelLabs/data-systems/commit/a70976a710c5273e270e80120dd27a474b7474be) @pedronauck)

### 💪🏼 Contributors

- @pedronauck
- @luizstacio

Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---

## [0.0.29](https://github.com/FuelLabs/data-systems/compare/v0.0.28...v0.0.29) - 2025-04-18

### ⭐ Features

- _(repo)_: Normalize database ([#454](https://github.com/FuelLabs/data-systems/pull/454)) ([d7a3205](https://github.com/FuelLabs/data-systems/commit/d7a3205c28910d07398075ac31b272df661cdf83) @pedronauck)

- _(repo)_: Add dune integration ([#455](https://github.com/FuelLabs/data-systems/pull/455)) ([1199501](https://github.com/FuelLabs/data-systems/commit/1199501caba5f500cd1fbd73601c1133ae73f6d6) @pedronauck)

- [**breaking**] _(repo)_: Introduce new API and data scheme ([86f63a8](https://github.com/FuelLabs/data-systems/commit/86f63a8a78965e527338ef405633f077afdca294) @pedronauck)

- _(repo)_: Add predicates ([#451](https://github.com/FuelLabs/data-systems/pull/451)) ([4ad513d](https://github.com/FuelLabs/data-systems/commit/4ad513d6eb30e649187d750e9c56ff2fd72303e4) @pedronauck)

- _(sv-api)_: Updated postman and swagger-ui frontend ([#450](https://github.com/FuelLabs/data-systems/pull/450)) ([e566d7b](https://github.com/FuelLabs/data-systems/commit/e566d7b2da8ecf971a124ca277d3a9c04067aead) @0xterminator)

### 🐛 Bug Fixes

- _(repo)_: Adjust retro compatibility ([c023c46](https://github.com/FuelLabs/data-systems/commit/c023c46c541426386c7663ad6f16d8769ee0c1f1) @pedronauck)

- _(domains)_: BlockVersion deserialization ([70382e6](https://github.com/FuelLabs/data-systems/commit/70382e6b3cb07fcfe0c5ec05da58072b1b2fc818) @pedronauck)

- _(domains)_: Performance issue on transaction query ([8ece474](https://github.com/FuelLabs/data-systems/commit/8ece474f8ece1dc5028b98b63ed13e072ae9c533) @pedronauck)

- _(sv-api)_: Fixed pagination cursor ([#448](https://github.com/FuelLabs/data-systems/pull/448)) ([3a2168e](https://github.com/FuelLabs/data-systems/commit/3a2168eadd9cc3e2960b4ca62718e896964fbd0e) @0xterminator)

- _(repo)_: Serializing block time as integer ([7c46a3a](https://github.com/FuelLabs/data-systems/commit/7c46a3ae9982b28d96e4763f42a08c866f1b4a90) @pedronauck)

- _(sv-api)_: Generate api keys endpoint ([899c5b1](https://github.com/FuelLabs/data-systems/commit/899c5b1bbe31a6c526476f3fec08d6c97af56650) @pedronauck)

- _(web-utils)_: Remove db tx when creating or updating keys ([4830ae5](https://github.com/FuelLabs/data-systems/commit/4830ae59995fd3f3d2a7b6ff64684bff13579c7c) @pedronauck)

- _(sv-api)_: Removed system-based metrics from metrics endpoint ([#446](https://github.com/FuelLabs/data-systems/pull/446)) ([36674e3](https://github.com/FuelLabs/data-systems/commit/36674e3bf3f4c078a64a77a62130456ffd7bab33) @0xterminator)

- _(repo)_: Adjust SwaggerUI to work with custom servers ([cb08080](https://github.com/FuelLabs/data-systems/commit/cb0808032882cd3d49de0d1ab4459cbeff6297e4) @pedronauck)

- _(web-utils)_: Metrics standard endpoint using state instead of extension ([2430656](https://github.com/FuelLabs/data-systems/commit/2430656db10b471c67a1ca76787cc25558982921) @pedronauck)

- _(sv-api)_: Rest API minor fixes ([#445](https://github.com/FuelLabs/data-systems/pull/445)) ([9da695a](https://github.com/FuelLabs/data-systems/commit/9da695ada96d9c0f689b5d63bc7c14b4c09fa07b) @pedronauck)

### 🔄 Refactor

- _(repo)_: Change from actix to axum ([#443](https://github.com/FuelLabs/data-systems/pull/443)) ([2a45d2f](https://github.com/FuelLabs/data-systems/commit/2a45d2ff2be1d5469cab1204eecc0ab816704c4c) @pedronauck)

### 🏗️ Build

- _(repo)_: Fix clippy warnings ([0062b86](https://github.com/FuelLabs/data-systems/commit/0062b86926240dd35aadcd42cc4ed91e2325ede2) @pedronauck)

- _(repo)_: Fix dune on helm ([ceeaf6d](https://github.com/FuelLabs/data-systems/commit/ceeaf6dd3b29191e6df3f76160aa22751ed4573c) @pedronauck)

- _(repo)_: Fix chart tests and new version ([492fd12](https://github.com/FuelLabs/data-systems/commit/492fd121678c33477f0a3a922e5aa80dc8d43b06) @pedronauck)

### 💪🏼 Contributors

- @pedronauck
- @0xterminator

Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---

## [0.0.28](https://github.com/FuelLabs/data-systems/compare/v0.0.27...v0.0.28) - 2025-03-13

### 🐛 Bug Fixes

- _(fuel-streams)_: Handle bynari message receiving when sequence ([6a27bcd](https://github.com/FuelLabs/data-systems/commit/6a27bcd7b538129fe2e93fa18bb2da9e4edec00a) @pedronauck)

### 💪🏼 Contributors

- @pedronauck

Want to contribute? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md) guide!

---

## [0.0.27](https://github.com/FuelLabs/data-systems/compare/v0.0.26...v0.0.27) - 2025-03-13

### ⭐ Features

- _(repo)_: Added open-api documentation ([#428](https://github.com/FuelLabs/data-systems/pull/428)) ([9883a75](https://github.com/FuelLabs/data-systems/commit/9883a755c51027b6faeb33d76b8d4dc683f25ae6) @0xterminator)

### 🔄 Refactor

- _(fuel-streams-store)_: Improve historical data query performance ([bd81409](https://github.com/FuelLabs/data-systems/commit/bd814098c6c02ef9cb8c42bcc871b5b22cfcc9ab) @pedronauck)

- _(sv-webserver)_: Improve performance on subscribing ([#435](https://github.com/FuelLabs/data-systems/pull/435)) ([2a35a17](https://github.com/FuelLabs/data-systems/commit/2a35a17eb55660efd1f80a59311edda3e150274d) @pedronauck)

### 🏗️ Build

- _(repo)_: Update rust to v1.85.1 ([90e5425](https://github.com/FuelLabs/data-systems/commit/90e5425d851460b25a92b8fb9b3cee0ab41c20c4) @pedronauck)

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
