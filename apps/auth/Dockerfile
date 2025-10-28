# Build stage
FROM rust:1.83-slim-bookworm AS builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY apps/auth/Cargo.toml ./apps/auth/
COPY apps/driver/Cargo.toml ./apps/driver/
COPY apps/gateway/Cargo.toml ./apps/gateway/
COPY libs/common/Cargo.toml ./libs/common/
COPY libs/utils/Cargo.toml ./libs/utils/

# Create dummy source files to build dependencies
RUN mkdir -p apps/auth/src apps/driver/src apps/gateway/src libs/common/src libs/utils/src && \
    echo "fn main() {}" > apps/auth/src/main.rs && \
    echo "fn main() {}" > apps/driver/src/main.rs && \
    echo "fn main() {}" > apps/gateway/src/main.rs && \
    echo "pub fn dummy() {}" > libs/common/src/lib.rs && \
    echo "pub fn dummy() {}" > libs/utils/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release -p auth

# Remove dummy source files
RUN rm -rf apps/auth/src apps/driver/src apps/gateway/src libs/common/src libs/utils/src

# Copy actual source code
COPY apps/auth ./apps/auth
COPY libs/common ./libs/common
COPY libs/utils ./libs/utils

# Build the actual application
RUN cargo build --release -p auth

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/auth /app/auth

# Copy config file
COPY apps/auth/config.yaml /app/config.yaml

# Copy migrations
COPY apps/auth/migrations /app/migrations

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8001

# Set environment variables
ENV RUST_LOG=info

# Run the binary
CMD ["./auth"]