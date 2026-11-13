# Images are pinned by digest (Renovate bumps tag and digest together): a re-pushed tag cannot
# change what gets built.
# Builder, runtime and development.Dockerfile share one Debian release (13 "trixie"), so the
# binary links against the glibc and OpenSSL the runtime image ships (MAIR-741: Debian 12's
# distroless libssl3 is unpatched).
FROM rust:1.100-slim-trixie@sha256:e9d95b74296f7126a83ae3d041aeaba1ff4c478f13abc5cc034fcf7c131c3d4a AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# --- DEPENDENCY CACHE ---
# Build the dependencies alone first: this layer stays cached as long as Cargo.toml and Cargo.lock
# do not change, so a code change only recompiles the API itself.
# `--locked`: build exactly the reviewed `Cargo.lock`, fail instead of resolving new versions.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && touch src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src
# -----------------------------

COPY . .
# Make sure cargo rebuilds the crate instead of keeping the placeholder built above.
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

# --- Stage 2: runtime (distroless, non-root) ---
# The `nonroot` variant runs as uid/gid 65532. `USER` is repeated numerically so
# Kubernetes can enforce `runAsNonRoot: true`. The API binds an unprivileged
# port (`PORT`, 3000+) and never writes to the filesystem. The binary stays
# owned by root (read + execute only for the runtime user).
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:cc66d222f0a7ec996b78a99fe76a969bc80b539a5016ad866c14d1949bc6d0ab
WORKDIR /app

COPY --from=builder /usr/src/app/target/release/message_api /app/message-api

USER 65532:65532

CMD ["/app/message-api"]
