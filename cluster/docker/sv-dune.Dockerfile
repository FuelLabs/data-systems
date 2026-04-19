FROM --platform=$BUILDPLATFORM tonistiigi/xx AS xx
FROM --platform=$BUILDPLATFORM rust:1.90.0-bookworm AS chef

ARG TARGETPLATFORM
RUN cargo install cargo-chef
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
    xx-cargo chef cook --release --no-default-features -p sv-dune --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
# build application
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry/index \
    --mount=type=cache,target=/usr/local/cargo/registry/cache \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/build/target \
    xx-cargo build --release --no-default-features -p sv-dune \
    && xx-verify ./target/$(xx-cargo --print-target-triple)/release/sv-dune \
    && cp ./target/$(xx-cargo --print-target-triple)/release/sv-dune /root/sv-dune \
    && cp ./target/$(xx-cargo --print-target-triple)/release/sv-dune.d /root/sv-dune.d

# Stage 2: Run
FROM debian:bookworm-slim AS run

WORKDIR /usr/src

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates curl jq less unzip zip procps \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /root/sv-dune .
COPY --from=builder /root/sv-dune.d .

# run app as non-root user
USER nobody

CMD ["./sv-dune"]
