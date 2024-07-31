# Stage 1: Build
FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1.75.0 AS chef

ARG TARGETPLATFORM
RUN cargo install cargo-chef && rustup target add wasm32-unknown-unknown
WORKDIR /build/

COPY --from=xx / /

# hadolint ignore=DL3008
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    lld \
    clang \
    libclang-dev \
    && xx-apt-get update  \
    && xx-apt-get install -y libc6-dev g++ binutils \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*


FROM chef as planner
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef as builder
ARG DEBUG_SYMBOLS=false
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_PROFILE_RELEASE_DEBUG=$DEBUG_SYMBOLS
COPY --from=planner /build/recipe.json recipe.json
RUN echo $CARGO_PROFILE_RELEASE_DEBUG
# Build our project dependencies, not our application!
RUN xx-cargo chef cook --release --no-default-features -p fuel-streams-publisher --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
RUN xx-cargo build --release --no-default-features -p fuel-streams-publisher \
    && xx-verify ./target/$(xx-cargo --print-target-triple)/release/fuel-streams-publisher \
    && mv ./target/$(xx-cargo --print-target-triple)/release/fuel-streams-publisher ./target/release/fuel-streams-publisher \
    && mv ./target/$(xx-cargo --print-target-triple)/release/fuel-streams-publisher.d ./target/release/fuel-streams-publisher.d

# Stage 2: Run
FROM ubuntu:22.04 as run

ARG IP=0.0.0.0
ARG PORT=4000
ARG P2P_PORT=30333
ARG DB_PATH=/mnt/db/
ARG POA_INSTANT=false
ARG RELAYER_LOG_PAGE_SIZE=2000
ARG SERVICE_NAME="NATS Publisher Node"
ARG SYNC_HEADER_BATCH_SIZE=100
ARG RESERVED_NODE_DNS=/dns4/p2p-testnet.fuel.network/tcp/30333/p2p/16Uiu2HAmDxoChB7AheKNvCVpD4PHJwuDGn8rifMBEHmEynGHvHrf

ENV IP=$IP
ENV PORT=$PORT
ENV DB_PATH=$DB_PATH
ENV POA_INSTANT=false
ENV RELAYER_LOG_PAGE_SIZE=$RELAYER_LOG_PAGE_SIZE
ENV SERVICE_NAME=$SERVICE_NAME
ENV SYNC_HEADER_BATCH_SIZE=$SYNC_HEADER_BATCH_SIZE

ENV KEYPAIR=
ENV RELAYER=
ENV RELAYER_V2_LISTENING_CONTRACTS=
ENV RELAYER_DA_DEPLOY_HEIGHT=
ENV NATS_URL=
ENV NATS_NKEY_SEED=
ENV RESERVED_NODE_DNS=

WORKDIR /usr/src

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/fuel-streams-publisher .
COPY --from=builder /build/target/release/fuel-streams-publisher.d .

COPY /docker/chain-config ./chain-config

# https://stackoverflow.com/a/44671685
# https://stackoverflow.com/a/40454758
# hadolint ignore=DL3025
CMD exec ./fuel-streams-publisher \
    --service-name "${SERVICE_NAME}" \
    --keypair $KEYPAIR \
    --relayer $RELAYER \
    --ip $IP \
    --port $PORT \
    --peering-port $P2P_PORT \
    --db-path "${DB_PATH}" \
    --utxo-validation \
    --poa-instant $POA_INSTANT \
    --snapshot ./chain-config \
    --enable-p2p \
    --reserved-nodes $RESERVED_NODE_DNS \
    --sync-header-batch-size $SYNC_HEADER_BATCH_SIZE \
    --enable-relayer \
    --relayer-v2-listening-contracts $RELAYER_V2_LISTENING_CONTRACTS \
    --relayer-da-deploy-height $RELAYER_DA_DEPLOY_HEIGHT \
    --relayer-log-page-size $RELAYER_LOG_PAGE_SIZE \
    --sync-block-stream-buffer-size 30 \
    --nats-url "${NATS_URL}" \
    --nats-nkey "${NATS_NKEY_SEED}"
