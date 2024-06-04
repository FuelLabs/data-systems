# fuel-nats-stream

Start NATS serve with JetStream support:

```sh
nats-server -js
```

Fetch and publish blocks from Fuel Network. Use `latest` to start at the latest block height. Omit the parameter to start at the last block in the stream (which may be zero).

```sh
cargo run -- beta-5.fuel.network latest
```

Deploy and call a contract:

```sh
git clone https://github.com/lostman/fuel-counter
cd fuel-counter
make deploy && make call
```

Subscribe to NATS stream:

```sh
nats sub "receipts.*.*.counter.>" --last --headers-only
# 12:56:39 Subscribing to JetStream Stream holding messages with subject receipts.*.*.counter.> starting with the last message received
# [#1] Received JetStream message: stream: fuel seq 8664 / subject: receipts.11330484.0x6dc4a9176a319bf20f35de5d6d817344351755163dbf1704400ba43f5e41d15f.counter.incr / time: 2024-05-03T12:55:27+02:00
# Nats-Msg-Size: 8
```
