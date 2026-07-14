# syntax=docker/dockerfile:1

# --- Stage 1: Build ---
FROM rust:1.92-slim AS builder

# Install SQLite dev headers required by sqlx at compile time, plus zig for cross-compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libsqlite3-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://ziglang.org/download/0.14.0/zig-linux-x86_64-0.14.0.tar.gz | tar -xzf - -C /usr/local && \
    ln -s /usr/local/zig-linux-x86_64-0.14.0/zig /usr/local/bin/zig

RUN cargo install cargo-zigbuild

WORKDIR /build

# Copy dependency files first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

ARG TARGET=x86_64-unknown-linux-gnu
RUN cargo zigbuild --release --target $TARGET --bin backend_rust

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim AS runtime

ARG TARGET=x86_64-unknown-linux-gnu

RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from builder stage (path includes target triple for cross-compilation)
COPY --from=builder /build/target/${TARGET}/release/backend_rust /app/backend_rust

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
