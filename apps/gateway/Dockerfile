# Build stage
FROM rustlang/rust:nightly-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifests from root
COPY Cargo.toml Cargo.lock ./

# Copy all Cargo.toml files
COPY apps/auth/Cargo.toml ./apps/auth/
COPY apps/driver/Cargo.toml ./apps/driver/
COPY apps/gateway/Cargo.toml ./apps/gateway/
COPY libs/common/Cargo.toml ./libs/common/
COPY libs/utils/Cargo.toml ./libs/utils/

# Create dummy source files for dependency caching
RUN mkdir -p apps/auth/src apps/driver/src apps/gateway/src libs/common/src libs/utils/src && \
    echo "fn main() {}" > apps/auth/src/main.rs && \
    echo "fn main() {}" > apps/driver/src/main.rs && \
    echo "fn main() {}" > apps/gateway/src/main.rs && \
    echo "pub fn dummy() {}" > libs/common/src/lib.rs && \
    echo "pub fn dummy() {}" > libs/utils/src/lib.rs

# Build dependencies
RUN cargo build --release -p gateway

# Remove dummy sources
RUN rm -rf apps/gateway/src libs/common/src libs/utils/src

# Copy actual source code
COPY apps/gateway ./apps/gateway
COPY libs/common ./libs/common
COPY libs/utils ./libs/utils

# Build the application
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
COPY apps/gateway/config.yaml  /app/config.yaml
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

ENV RUST_LOG=info
ENV CONFIG_PATH=/app/apps/gateway/config.yaml

CMD ["./gateway"]