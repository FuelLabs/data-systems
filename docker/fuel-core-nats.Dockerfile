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
RUN xx-cargo chef cook --release --no-default-features -p fuel-core-nats --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
RUN xx-cargo build --release --no-default-features -p fuel-core-nats \
    && xx-verify ./target/$(xx-cargo --print-target-triple)/release/fuel-core-nats \
    && mv ./target/$(xx-cargo --print-target-triple)/release/fuel-core-nats ./target/release/fuel-core-nats \
    && mv ./target/$(xx-cargo --print-target-triple)/release/fuel-core-nats.d ./target/release/fuel-core-nats.d

# Stage 2: Run
FROM ubuntu:22.04 as run

ARG IP=0.0.0.0
ARG PORT=4000
ARG P2P_PORT=30333
ARG DB_PATH=./mnt/db/
ARG POA_INSTANT=false
ARG RELAYER_LOG_PAGE_SIZE=2000
ARG SERVICE_NAME="NATS Publisher Node"
ARG SYNC_HEADER_BATCH_SIZE=100

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

WORKDIR /usr/src

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/fuel-core-nats .
COPY --from=builder /build/target/release/fuel-core-nats.d .

EXPOSE $PORT
EXPOSE $P2P_PORT

# https://stackoverflow.com/a/44671685
# https://stackoverflow.com/a/40454758
# hadolint ignore=DL3025
CMD exec ./fuel-core-nats \
    --ip $IP \
    --port $PORT \
    --db-path "${DB_PATH}" \
    --enable-p2p \
    --poa-instant $POA_INSTANT \
    --utxo-validation \
    --keypair $KEYPAIR \
    --enable-relayer \
    --relayer $RELAYER \
    --relayer-v2-listening-contracts $RELAYER_V2_LISTENING_CONTRACTS \
    --relayer-da-deploy-height $RELAYER_DA_DEPLOY_HEIGHT \
    --relayer-log-page-size $RELAYER_LOG_PAGE_SIZE \
    --service-name "${SERVICE_NAME}" \
    --sync-header-batch-size $SYNC_HEADER_BATCH_SIZE \
    --nats-url "${NATS_URL}" \
