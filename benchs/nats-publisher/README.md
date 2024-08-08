# Running

1. First generate your key pair:

```sh
fuel-core-keygen new --key-type peering -p
```

2. Make sure you have NATS server running within the workspace root:

```
make start/nats
```

2. The, you can start local node and start publishing on NATS:

```
cargo run -- --service-name "test-jetstream" \
            --keypair <YOUR_KEYPAIR> \
            --relayer https://sepolia.infura.io/v3/<YOUR_INFURA_KEY> \
            --ip 0.0.0.0 \
            --port 4004 \
            --peering-port 30333 \
            --db-path ../../docker/db \
            --snapshot ../../docker/chain-config \
            --enable-p2p \
            --reserved-nodes /dns4/p2p-testnet-temp.fuel.network/tcp/30339/p2p/16Uiu2HAmKRLmFHbtm5aUucY1o4WVEPQw877pvwuSKcKh9KSFyiwd \
            --sync-header-batch-size 100 \
            --enable-relayer \
            --relayer-v2-listening-contracts 0x01855B78C1f8868DE70e84507ec735983bf262dA \
            --relayer-da-deploy-height 5827607 \
            --relayer-log-page-size 10 \
            --sync-block-stream-buffer-size 30
```
