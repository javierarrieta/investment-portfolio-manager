# Docker Release Images Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish Docker images for the Rust backend and React frontend to GHCR when a `v*` tag is pushed.

**Architecture:** Two multi-stage Dockerfiles (one per service) + a single GitHub Actions release workflow that builds and pushes both images on tag push. Backend uses `debian-slim` for SQLite support; frontend uses `nginx:alpine` to serve static Vite output.

**Tech Stack:** Docker, GitHub Actions, GitHub Container Registry (GHCR), Rust (Rocket), Node.js (Vite), nginx.

---

### Task 1: Create `backend_rust/Dockerfile`

**Files:**
- Create: `backend_rust/Dockerfile`

- [ ] **Step 1: Write the Dockerfile**

Create `backend_rust/Dockerfile` with this exact content:

```dockerfile
# syntax=docker/dockerfile:1

# --- Stage 1: Build ---
FROM rust:1.82-bookworm-slim AS builder

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
    libsqlite3-2 \
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
EXPOSE 8000

ENTRYPOINT ["./backend_rust"]
```

- [ ] **Step 2: Verify Dockerfile syntax**

Run locally to check the file is valid Docker syntax:

```bash
docker buildx validate
```

If Docker is not available locally, just verify the file content by reading it back:

```bash
cat backend_rust/Dockerfile
```

Expected: file contains both `builder` and `runtime` stages, COPY from builder, ENTRYPOINT set.

- [ ] **Step 3: Commit**

```bash
git add backend_rust/Dockerfile
git commit -m "feat: add multi-stage Dockerfile for Rust backend"
```

---

### Task 2: Create `frontend/Dockerfile`

**Files:**
- Create: `frontend/Dockerfile`

- [ ] **Step 1: Write the Dockerfile**

Create `frontend/Dockerfile` with this exact content:

```dockerfile
# syntax=docker/dockerfile:1

# --- Stage 1: Build ---
FROM node:20-alpine AS builder

WORKDIR /build

# Copy dependency files first for layer caching
COPY package.json package-lock.json ./

RUN npm ci

# Copy source and build
COPY . .
RUN npm run build

# --- Stage 2: Serve with nginx ---
FROM nginx:1.27-alpine AS runtime

# Remove default nginx site
RUN rm -rf /usr/share/nginx/html/*

# Copy built frontend assets from builder stage
COPY --from=builder /build/dist /usr/share/nginx/html

EXPOSE 80

ENTRYPOINT ["nginx", "-g", "daemon off;"]
```

- [ ] **Step 2: Verify Dockerfile syntax**

Read the file back to confirm:

```bash
cat frontend/Dockerfile
```

Expected: file contains `builder` (node:20-alpine) and `runtime` (nginx:1.27-alpine) stages, `npm run build` in builder, nginx serving from `/usr/share/nginx/html`.

- [ ] **Step 3: Commit**

```bash
git add frontend/Dockerfile
git commit -m "feat: add multi-stage Dockerfile for frontend (nginx)"
```

---

### Task 3: Create `.github/workflows/release.yml`

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Write the release workflow**

Create `.github/workflows/release.yml` with this exact content:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  packages: write

jobs:
  docker-build-push:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - service: backend-rust
            context: backend_rust
            dockerfile: backend_rust/Dockerfile
          - service: frontend
            context: frontend
            dockerfile: frontend/Dockerfile

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Extract version
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/}" >> $GITHUB_OUTPUT

      - name: Log in to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build and push ${{ matrix.service }}
        uses: docker/build-push-action@v6
        with:
          context: .
          file: ${{ matrix.dockerfile }}
          push: true
          tags: |
            ghcr.io/${{ github.repository }}/${{ matrix.service }}:${{ steps.version.outputs.VERSION }}
            ghcr.io/${{ github.repository }}/${{ matrix.service }}:latest
```

- [ ] **Step 2: Verify workflow syntax**

Read the file back to confirm:

```bash
cat .github/workflows/release.yml
```

Expected: triggers on `tags: ['v*']`, uses `docker/login-action`, `docker/build-push-action`, matrix includes both `backend-rust` and `frontend`, tags with both `${VERSION}` and `latest`.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add release workflow for Docker image publishing to GHCR"
```

---

## Verification

After all tasks are complete, verify the setup:

1. **Lint check** — ensure CI workflow still passes:

```bash
# Backend tests still run
cd backend_rust && cargo test && cd ..

# Frontend tests still run
cd frontend && npm test && cd ..
```

2. **Dry-run build** (if Docker is available locally) — test that each Dockerfile builds without error:

```bash
docker build -t test-backend -f backend_rust/Dockerfile .
docker build -t test-frontend -f frontend/Dockerfile frontend/
```

3. **Push a test tag** to trigger the workflow (use a non-production tag like `v0.0.1-test`):

```bash
git push origin main --tags
```

Then check the Actions tab at `https://github.com/javierarrieta/investment-portfolio-manager/actions`.

---

## Files Summary

| File | Action | Description |
|------|--------|-------------|
| `backend_rust/Dockerfile` | Create | Multi-stage Rust build + slim runtime |
| `frontend/Dockerfile` | Create | Multi-stage Node build + nginx serve |
| `.github/workflows/release.yml` | Create | GHCR publish on `v*` tag push |
| `.github/workflows/ci.yml` | No change | Existing CI stays as-is |
| All source code | No change | No application code modifications needed |
