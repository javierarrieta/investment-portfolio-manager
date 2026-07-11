# Docker Release Images — Design Spec

**Date**: 2026-07-11
**Status**: Approved

## Goal

Publish Docker images for the Rust backend and React frontend to GitHub Container Registry (GHCR) when a version tag (e.g. `v1.0.0`) is pushed to the repository.

## Architecture

### `backend_rust/Dockerfile` — Multi-stage build

| Stage | Base image | Purpose |
|-------|-----------|---------|
| `builder` | `rust:bookworm-slim` | Install `libsqlite3-dev`, copy Cargo files + source, `cargo build --release` |
| `runtime` | `debian:bookworm-slim` | Install `libsqlite3-2`, copy release binary, expose 8000 |

- `VOLUME /app/data` for the persistent `portfolio.db`
- `ENTRYPOINT ["./backend_rust"]`
- Binary runs as non-root user where possible

### `frontend/Dockerfile` — Multi-stage build

| Stage | Base image | Purpose |
|-------|-----------|---------|
| `builder` | `node:20-alpine` | `npm ci` + `npm run build`, outputs to `dist/` |
| `runtime` | `nginx:alpine` | Copy `dist/` to `/usr/share/nginx/html`, expose 80 |

- Static files served by nginx (no Node.js runtime needed in final image)
- Expected size: ~25MB

### `.github/workflows/release.yml` — Release workflow

**Trigger**: `push: tags: ['v*']`

**Job: `docker-build-push`**
1. Checkout code
2. Extract version from `${{ github.ref_name }}`
3. Login to GHCR using `GITHUB_TOKEN` (scoped: `packages: write`)
4. Build and push backend image — tagged with both `${VERSION}` and `latest`
5. Build and push frontend image — tagged with both `${VERSION}` and `latest`

**Image tags** (GHCR):
- `ghcr.io/javierarrieta/investment-portfolio-manager/backend-rust:v1.0.0`
- `ghcr.io/javierarrieta/investment-portfolio-manager/backend-rust:latest`
- `ghcr.io/javierarrieta/investment-portfolio-manager/frontend:v1.0.0`
- `ghcr.io/javierarrieta/investment-portfolio-manager/frontend:latest`

## What stays unchanged

- `.github/workflows/ci.yml` — existing PR/push test workflow remains as-is
- No changes to application code
- `start.sh` is not affected

## Release process (user workflow)

1. Bump version in `Cargo.toml` and `package.json`
2. Commit changes
3. `git tag v1.0.0` (or use GitHub "Create Release" UI — both work)
4. `git push origin main --tags`
5. The `release.yml` workflow runs automatically, publishes both images to GHCR

## Files to create

1. `backend_rust/Dockerfile`
2. `frontend/Dockerfile`
3. `.github/workflows/release.yml`

## Files to NOT modify

- `backend_rust/Cargo.toml` (only version number bump by user)
- `frontend/package.json` (only version number bump by user)
- `.github/workflows/ci.yml`
- Application source code
