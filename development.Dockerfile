# Same toolchain as the production `Dockerfile`, pinned by digest.
FROM rust:1.99-bookworm@sha256:5c1a1732ac3f77635b24d397ceb67b4d9ea61de348f68c0aa54a79fd01e65c69 AS development

RUN apt update && apt install -y curl && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch --locked

# Run as an unprivileged user: it owns the sources, `target/` and the cargo registry cache so
# `cargo watch` can rebuild and Compose `develop.watch` can sync files into the container.
RUN useradd --create-home --uid 1000 dev \
    && mkdir -p /usr/src/message \
    && chown -R dev:dev /usr/src/message /usr/local/cargo
COPY --chmod=755 entrypoint.sh /usr/local/bin/entrypoint.sh
USER dev

# Must match the `develop.watch` targets of docker-compose.yml and entrypoint.sh
WORKDIR /usr/src/message

# --- DEPENDENCY CACHE ---
COPY --chown=dev:dev Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
# This layer stays cached as long as Cargo.toml and Cargo.lock do not change
RUN cargo build --locked && rm -rf src
# -----------------------------

COPY --chown=dev:dev src ./src

EXPOSE 3003
CMD ["/usr/local/bin/entrypoint.sh"]
