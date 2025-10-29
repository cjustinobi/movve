#!/bin/bash

echo "Setting up movve monorepo..."

# Copy environment file
if [ ! -f .env ]; then
    cp .env.template .env
    echo "Created .env file from template"
fi

# Install cargo-make if not installed
if ! command -v cargo-make &> /dev/null; then
    echo "Installing cargo-make..."
    cargo install cargo-make
fi

# Install cargo-watch for hot reloading
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch with macOS AppKit framework fix..."
    RUSTFLAGS="-C link-arg=-framework -C link-arg=AppKit" cargo install cargo-watch
else
    echo "cargo-watch already installed ✅"
fi

# Install sqlx-cli
if ! command -v sqlx &> /dev/null; then
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Setup database
echo "Setting up database..."
cargo make db-setup

echo "Setup complete! Run 'cargo make run-all' to start all services"