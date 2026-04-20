# Build stage
FROM rust:latest AS builder
WORKDIR /app

# Copy all source files
COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists
COPY --from=builder /app/target/release/dex-account /usr/local/bin/

EXPOSE 3000

CMD ["dex-account"]