# Build stage
FROM rust:latest AS builder
WORKDIR /app

# Etapa 1: Copiar só deps ( muda só quando Cargo.* muda )
COPY Cargo.toml Cargo.lock* ./
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists
RUN cargo build --release && touch /app/src/.keep

# Etapa 2: Copiar código ( só isso invalida rebuild do código )
COPY src/ ./src/
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists
COPY --from=builder /app/target/release/dex-account /usr/local/bin/

EXPOSE 3000

CMD ["dex-account"]