# Build stage
FROM rust:latest AS builder
WORKDIR /app

COPY . .
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/dex-account /usr/local/bin/

EXPOSE 3000

CMD ["dex-account"]
