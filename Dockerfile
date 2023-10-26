# syntax=docker/dockerfile:1

### Cargo-chef
FROM rust:1.70.0-slim AS chef
WORKDIR /usr/src/wave-autoscale
RUN set -eux; \
    cargo install cargo-chef; \
    rm -rf $CARGO_HOME/registry

### Planner
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

### Builder
FROM chef AS builder
RUN apt-get update && apt -y install pkg-config libssl-dev build-essential libc6-dev
COPY --from=planner /usr/src/wave-autoscale/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

### Runtime
FROM debian:bullseye-slim
RUN apt-get update
RUN apt-get install -y openssl ca-certificates
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/wave-autoscale/target/release/wave-autoscale .
CMD ["./wave-autoscale"]