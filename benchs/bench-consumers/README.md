# Running


Generate a P2P secret:

```
fuel-core-keygen new --key-type peering -p
```

Connect to `devnet`:

```
#!/bin/bash

ETH_RPC_ENDPOINT="https://sepolia.infura.io/v3/{API_KEY}"
P2P_SECRET="{GENERATED_P2P_SECRET}"

cargo run --all-features --bin fuel-streams-publisher -- \
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
