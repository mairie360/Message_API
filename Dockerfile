# Étape 1 : Build (Utilisation de la toute dernière version 1.88)
FROM rust:1.88-slim AS builder

# Installation des dépendances système nécessaires
RUN apt update && apt install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

# Compilation
RUN cargo build --release

# Étape 2 : Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Installation des certificats CA, de libssl et de CURL pour le healthcheck
RUN apt update && apt install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copie du binaire
COPY --from=builder /usr/src/app/target/release/message_api /app/message-api

# On lance le binaire
CMD ["./message-api"]