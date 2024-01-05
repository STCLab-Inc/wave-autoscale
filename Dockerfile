# syntax=docker/dockerfile:1

#
# Rust build stage
#
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

#
# Node build stage
#
FROM node:18.19.0-bullseye-slim AS node-base

### Dependencies
FROM node-base AS node-deps
# Check https://github.com/nodejs/docker-node/tree/b4117f9333da4138b03a546ec926ef50a31506c3#nodealpine to understand why libc6-compat might be needed.
# RUN apk add --no-cache libc6-compat
WORKDIR /app
# Install dependencies based on the preferred package manager
COPY core/web-app/package.json core/web-app/package-lock.json* ./
RUN npm ci

### Builder
FROM node-base AS node-builder
WORKDIR /app
COPY --from=node-deps /app/node_modules ./node_modules
COPY core/web-app/ .
RUN npm run build

### Runtime
# FROM debian:bullseye-slim
FROM node-base AS runtime
RUN apt-get update
RUN apt-get install -y openssl ca-certificates
WORKDIR /usr/local/bin
# Copy the binary from the rust build stage
COPY --from=builder /usr/src/wave-autoscale/target/release/wave-autoscale .
# Copy the built web-app from the node build stage
COPY --from=node-builder /app/.next/standalone ./wave-autoscale-ui
COPY --from=node-builder /app/.next/static ./wave-autoscale-ui/.next/static
COPY --from=node-builder /app/public ./wave-autoscale-ui/public

# Expose the port of api-server
EXPOSE 3024
# Expose the port of web-app
EXPOSE 3025

CMD ["./wave-autoscale"]