# syntax=docker/dockerfile:1

# --- Stage 1: Build ---
FROM rust:1.82-slim AS builder

# Install SQLite dev headers required by sqlx at compile time
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy dependency files first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

# Build release binary
RUN cargo build --release --bin backend_rust

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from builder stage
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
