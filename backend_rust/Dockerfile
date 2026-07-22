# syntax=docker/dockerfile:1

# --- Stage 1: Build ---
FROM rust:1.92-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy only dependency files first for layer caching
COPY Cargo.toml Cargo.lock ./

# Pre-build dependencies (cached unless Cargo.toml/Cargo.lock change)
RUN RUSTC_WRAPPER=sccache cargo build --release -j $(nproc) --bin backend_rust 2>/dev/null || true

# Copy source code
COPY src/ src/

# Full build with sccache
RUN RUSTC_WRAPPER=sccache cargo build --release -j $(nproc) --bin backend_rust

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/backend_rust /app/backend_rust

# Expose volume for persistent SQLite database
VOLUME /app/data

# The app defaults to ../portfolio.db relative to its working dir.
# When run with: docker run -v $(pwd)/portfolio.db:/app/data/portfolio.db ...
# we set DATABASE_URL so the path is correct.
ENV DATABASE_URL=sqlite:///app/data/portfolio.db?mode=rwc

RUN useradd -r -s /bin/false app
RUN mkdir -p /app/data && chown app:app /app/data
USER app

EXPOSE 8000

ENTRYPOINT ["./backend_rust"]
