# Running

Clone `chain-configuration`:

```
git clone git@github.com:FuelLabs/chain-configuration.git
```

Generate a P2P secret:

```
fuel-core-keygen new --key-type peering -p
```

Connect to `devnet`:

```
#!/bin/bash

ETH_RPC_ENDPOINT="https://sepolia.infura.io/v3/{API_KEY}"
P2P_SECRET="{GENERATED_P2P_SECRET}"

cargo run --all-features --bin fuel-core-nats -- \
	--service-name "NATS Publisher Node" \
	--ip 0.0.0.0 \
	--port 4000 \
	--db-path fuel-devnet-db \
	--peering-port 30333 \
	--reserved-nodes /dns4/p2p-devnet.fuel.network/tcp/30333/p2p 16Uiu2HAm6pmJUedRFjennk4A8yWL6zCApHCuykzRRroqMjjxZ8o6,/dns4/p2p-devnet.fuel.network/tcp/30334/p2p 16Uiu2HAm8dBwTRzqazCMqQDdR8thMa7BKiW4ep2B4DoQQp6Qhyfd \
	--utxo-validation \
	--poa-instant false \
	--enable-p2p \
	--keypair $P2P_SECRET \
	--sync-header-batch-size=100 \
	--enable-relayer \
	--relayer ${ETH_RPC_ENDPOINT} \
	--relayer-v2-listening-contracts=0x768f9459E3339A1F7d59CcF24C80Eb4A711a01FB \
	--relayer-da-deploy-height=5791365 \
	--relayer-log-page-size=2000 \
	--snapshot chain-configuration/ignition-dev
```

# Topics

### 1. `receipts.{height}.{contract_id}.{topic}...`

The data in a `LogData` receipt can be tagged for publishing to specified topics.

In Sway, this is done by wrapping the `log` message in a `Topic<T>` struct:

```rust
struct Topic<T> {
    header: u64,
    topics: [b256; 3],
    payload: T
}

impl<T> Topic<T> {
    fn new(topics: Vec<str>, payload: T) -> Topic<T> {
        assert(topics.len() <= 3);
        let mut i = 0;
        let mut result = [b256::min(); 3];
        while i < topics.len() {
            let topic = topics.get(i).unwrap();
            topic.as_ptr().copy_to::<u8>(__addr_of(result[i]), topic.len());
            i += 1;
        }
        Topic {
            header: 0x12345678,
            topics: result,
            payload
        }
    }
}
```

For example:

```rust
impl Counter for Contract {
   #[storage(read, write)]
    fn incr(amount: u64) -> u64 {
        let incremented = storage.counter.read() + amount;
        storage.counter.write(incremented);
        let mut topics = Vec::new();
        topics.push("counter");
        topics.push("incr");
        log(Topic::new(topics, incremented));
        incremented
    }
}
```

NATS Publisher recognizes the predefined prefix, unwraps the data, and publishes it. The data published from the `incr` call above is the 8 bytes of the `u64` incremented value.

```
nats sub "receipts.*.*.counter.>" --last --headers-only
```

```
12:38:41 Subscribing to JetStream Stream holding messages with subject receipts.*.*.counter.> starting with the last message received
[#1] Received JetStream message: stream: fuel seq 68 / subject: receipts.6.0000000000000000000000000000000000000000000000000000000000000000.counter.incr / time: 2024-05-20T12:06:59+02:00
Nats-Msg-Size: 8
```

### 2. `receipts.{height}.{contract_id}.{kind}`

```
[#1] Received JetStream message: stream: fuel seq 2497717 / subject: receipts.1247851.0000000000000000000000000000000000000000000000000000000000000000.script_result / time: 2024-05-24T18:55:31+02:00
{
  "ScriptResult": {
    "result": "Success",
    "gas_used": 636
  }
}
```

### 3. `blocks.{height}`

```
[#1005] Received JetStream message: stream: fuel seq 3458215 / subject: blocks.424405 / time: 2024-06-05T16:17:16+02:00
{
  "V1": {
    "header": {
      "V1": {
        "application": {
          "da_height": 5863309,
          "consensus_parameters_version": 0,
          "state_transition_bytecode_version": 0,
          "generated": {
            "transactions_count": 1,
            "message_receipt_count": 0,
            "transactions_root": "2afe56527d946e05af116f042e058aaffb77daca6440a96813167d15c2111b35",
            "message_outbox_root": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "event_inbox_root": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
          }
        },
        "consensus": {
          "prev_root": "77e19c68cd1dc9f1b97833b778059edc68b885463598e94d5cc9fd6acba8cf31",
          "height": 424405,
          "time": [
            64,
            0,
            0,
            0,
            102,
            59,
            238,
            168
          ],
          "generated": {
            "application_hash": "2996cde96e1dc9fd59227b59a3d9276a32ff68f293a131b5201b50312cfc5396"
          }
        }
      }
    },
    "transactions": [
      {
        "Mint": {
          "tx_pointer": {
            "block_height": 424405,
            "tx_index": 0
          },
          "input_contract": {
            "utxo_id": {
              "tx_id": "7e2abf5739892dc9869df06135b62fce83e19d7fd3775b6c42feeb471c2d5dd9",
              "output_index": 0
            },
            "balance_root": "80a40a209c900889ddf5833b3ce3776c156a19e879bcdf9a52be817d1f1ab6c9",
            "state_root": "0000000000000000000000000000000000000000000000000000000000000000",
            "tx_pointer": {
              "block_height": 424404,
              "tx_index": 0
            },
            "contract_id": "7777777777777777777777777777777777777777777777777777777777777777"
          },
          "output_contract": {
            "input_index": 0,
            "balance_root": "80a40a209c900889ddf5833b3ce3776c156a19e879bcdf9a52be817d1f1ab6c9",
            "state_root": "0000000000000000000000000000000000000000000000000000000000000000"
          },
          "mint_amount": 0,
          "mint_asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
          "gas_price": 1
        }
      }
    ]
  }
}
```

### 4. `transactions.{height}.{index}.{kind}`

```
[#1244268] Received JetStream message: stream: fuel seq 2500269 / subject: transactions.1249127.0.mint / time: 2024-05-24T19:58:23+02:00
{
  "Mint": {
    "tx_pointer": {
      "block_height": 1249127,
      "tx_index": 0
    },
    "input_contract": {
      "utxo_id": {
        "tx_id": "5206082b3d7f71595c5c02b830469cad0145d1149ba5a764343eaf000251a80e",
        "output_index": 0
      },
      "balance_root": "8b8d7c5dfa3e7caf6efd8e697616f78e04a5ccdfc7607bf017e7a3cff2be1bae",
      "state_root": "0000000000000000000000000000000000000000000000000000000000000000",
      "tx_pointer": {
        "block_height": 1249126,
        "tx_index": 0
      },
      "contract_id": "7777777777777777777777777777777777777777777777777777777777777777"
    },
    "output_contract": {
      "input_index": 0,
      "balance_root": "8b8d7c5dfa3e7caf6efd8e697616f78e04a5ccdfc7607bf017e7a3cff2be1bae",
      "state_root": "0000000000000000000000000000000000000000000000000000000000000000"
    },
    "mint_amount": 0,
    "mint_asset_id": "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
    "gas_price": 1
  }
}
```
