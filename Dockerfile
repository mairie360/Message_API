# --- Étape 1 : Build ---
FROM rust:1.88-slim AS builder

RUN apt update && apt install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# --- Étape 2 : Runtime (Ultra-light) ---
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

# On copie le binaire compilé
COPY --from=builder /usr/src/app/target/release/message_api /app/message-api

# Pas de shell, pas de root, sécurité et performance max
CMD ["/app/message-api"]