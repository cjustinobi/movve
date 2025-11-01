# Build stage
FROM rust:1.83-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY ../../Cargo.toml ../../Cargo.lock ./COPY apps/auth/Cargo.toml ./apps/auth/
COPY apps/driver/Cargo.toml ./apps/driver/
COPY apps/gateway/Cargo.toml ./apps/gateway/
COPY libs/common/Cargo.toml ./libs/common/
COPY libs/utils/Cargo.toml ./libs/utils/

RUN mkdir -p apps/auth/src apps/driver/src apps/gateway/src libs/common/src libs/utils/src && \
    echo "fn main() {}" > apps/auth/src/main.rs && \
    echo "fn main() {}" > apps/driver/src/main.rs && \
    echo "fn main() {}" > apps/gateway/src/main.rs && \
    echo "pub fn dummy() {}" > libs/common/src/lib.rs && \
    echo "pub fn dummy() {}" > libs/utils/src/lib.rs

RUN cargo build --release -p gateway

RUN rm -rf apps/gateway/src libs/common/src libs/utils/src

COPY apps/gateway ./apps/gateway
COPY libs/common ./libs/common
COPY libs/utils ./libs/utils

RUN cargo build --release -p gateway

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 appuser

WORKDIR /app

COPY --from=builder /app/target/release/gateway /app/gateway
COPY config.yaml /app/config.yaml

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8000

ENV RUST_LOG=info

CMD ["./gateway"]