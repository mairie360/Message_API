FROM rust:1.98-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

# --- Stage 2: runtime (distroless, non-root) ---
# The `nonroot` variant runs as uid/gid 65532. `USER` is repeated numerically so
# Kubernetes can enforce `runAsNonRoot: true`. The API binds an unprivileged
# port (`PORT`, 3000+) and never writes to the filesystem. The binary stays
# owned by root (read + execute only for the runtime user).
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app

COPY --from=builder /usr/src/app/target/release/message_api /app/message-api

USER 65532:65532

CMD ["/app/message-api"]
