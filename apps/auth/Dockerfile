FROM rust:1.77-alpine AS builder

WORKDIR /app
RUN apk add --no-cache musl-dev gcc

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src/ && echo "fn main() {println!(\"Hello!\");}" > src/main.rs && \
    cargo build --release --locked --target x86_64-unknown-linux-musl && \
    rm -rf src/main.rs target/x86_64-unknown-linux-musl/release/deps/user_service* && \
    rm -rf target/x86_64-unknown-linux-musl/release/.fingerprint/user_service*

COPY . .

ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN CGO_ENABLED=0 cargo build --release --locked --target x86_64-unknown-linux-musl

FROM alpine:latest

WORKDIR /app

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/user_service .

EXPOSE 8080

CMD ["./user-service"]