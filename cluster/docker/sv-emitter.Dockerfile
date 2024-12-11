# Stage 1: Build
FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1.81.0 AS chef

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


FROM chef AS planner
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
ARG DEBUG_SYMBOLS=false
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_PROFILE_RELEASE_DEBUG=$DEBUG_SYMBOLS
COPY --from=planner /build/recipe.json recipe.json
RUN echo $CARGO_PROFILE_RELEASE_DEBUG
# Build our project dependencies, not our application!
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/build/target \
    xx-cargo chef cook --release --no-default-features -p sv-emitter --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
# build application
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/build/target \
    xx-cargo build --release --no-default-features -p sv-emitter \
    && xx-verify ./target/$(xx-cargo --print-target-triple)/release/sv-emitter \
    && cp ./target/$(xx-cargo --print-target-triple)/release/sv-emitter /root/sv-emitter \
    && cp ./target/$(xx-cargo --print-target-triple)/release/sv-emitter.d /root/sv-emitter.d

# Stage 2: Run
FROM ubuntu:22.04 AS run

ENV PORT=$PORT
ENV ARGS=""
ENV NETWORK_ARGS=""

WORKDIR /usr/src

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /root/sv-emitter .
COPY --from=builder /root/sv-emitter.d .

COPY /cluster/chain-config ./chain-config
EXPOSE ${PORT}

# https://stackoverflow.com/a/44671685
# https://stackoverflow.com/a/40454758
# hadolint ignore=DL3025
CMD exec ./sv-emitter $ARGS $NETWORK_ARGS
