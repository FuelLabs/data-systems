## 0.0.2 (2024-08-16)

### 🚀 Features

-   Add fuel-core-nats v2 (#12)
-   Bootstrap the stream from Fuel nodes storage (#22)
-   Include nats server config in docker-compose (#34)
-   Add nkey auth (#32)
-   Add first version of streams-core crate (#61)
-   Add nats publishing deduplication logic and tests (#107)
-   Add Client (#114)
-   Data-parser implementation (#111)
-   Add Stream and Filter (#125)
-   Refactor core and publisher to integrate coherently (#124)

### 🐛 Fixes

-   Use u32 for block height to avoid hex outputs in block subjects (#66)
-   Publish compact json rather than pretty json (#105)
-   Temporary solution for nats storage size (#120)
-   Block compress for bincode (#122)
