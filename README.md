# Investment Portfolio Manager

Full-stack portfolio management application with tax lot tracking (FIFO/LIFO/Hybrid) and analytics.

- **Backend**: Rust (Rocket) + SQLite (SQLx)
- **Frontend**: React 19 + Vite + TypeScript + Recharts

---

## Local Development

```bash
# Install prerequisites
# Rust toolchain, Node.js 20+

# Run everything (backend on :8000, frontend on :5173)
./start.sh
```

- Frontend: <http://localhost:5173>
- Backend API: <http://127.0.0.1:8000>
- OpenAPI docs: <http://127.0.0.1:8000/api-docs/openapi.json>

---

## API Routes

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api-docs/openapi.json` | OpenAPI spec |
| `POST` | `/api/portfolios/` | Create portfolio |
| `GET` | `/api/portfolios/` | List portfolios |
| `GET` | `/api/portfolios/:id` | Get portfolio |
| `DELETE` | `/api/portfolios/:id` | Delete portfolio |
| `POST` | `/api/portfolios/:portfolio_id/assets` | Add asset |
| `DELETE` | `/api/assets/:id` | Remove asset |
| `POST` | `/api/portfolios/:portfolio_id/assets/:asset_id/transactions` | Add transaction |
| `GET` | `/api/portfolios/:portfolio_id/transactions` | List transactions |
| `DELETE` | `/api/transactions/:id` | Delete transaction |
| `GET` | `/api/portfolios/:id/tax-summary` | Tax summary |
| `GET` | `/api/portfolios/:id/performance` | Performance analytics |

---

## Kubernetes Deployment

The application is packaged as two Docker images (backend and frontend) and can be deployed to any Kubernetes cluster using the manifests below.

### Prerequisites

- Kubernetes cluster (v1.24+)
- `kubectl` configured to communicate with the cluster
- Container registry accessible by the cluster (images published via `ghcr.io`)

### Building and Pushing Images

```bash
# Build and push backend
docker build -t ghcr.io/<org>/investment-portfolio-manager/backend:latest -f backend_rust/Dockerfile .
docker push ghcr.io/<org>/investment-portfolio-manager/backend:latest

# Build and push frontend
docker build -t ghcr.io/<org>/investment-portfolio-manager/frontend:latest -f frontend/Dockerfile frontend/
docker push ghcr.io/<org>/investment-portfolio-manager/frontend:latest
```

Replace `<org>` with your GitHub username or organization.

### Single-File Deployment

Apply all resources in one step:

```bash
kubectl apply -f k8s/
```

### Resource Overview

```
k8s/
  namespace.yaml          # Namespace
  storageclass.yaml       # StorageClass
  configmap.yaml          # Backend configuration
  pv.yaml                 # Persistent volume for SQLite database
  pvc.yaml                # Persistent volume claim
  app-deployment.yaml     # Single pod: frontend + backend containers
  app-service.yaml        # ClusterIP service (:80)
  ingress.yaml            # Ingress routing all traffic to frontend
```

### Architecture

```
Ingress (host: invest.example.com)
  └── /          → app-service:80

Single Pod
  ├── frontend (nginx, :80)
  │   ├── serves static React build at /
  │   └── proxies /api/* and /api-docs → 127.0.0.1:8000
  │
  └── backend (Rocket, :8000)
      └── SQLite via PVC → /app/data/portfolio.db
```

Both containers run in the same pod and communicate over `127.0.0.1`. The frontend nginx proxies all API requests (`/api/*`, `/api-docs/*`) to the backend. The browser only talks to the frontend — no cross-service calls are needed.

An init container (`busybox:curl`) waits for the backend to become healthy before the frontend starts, ensuring the API is ready when nginx begins proxying.

### Storage

The backend uses a SQLite database (`portfolio.db`). A persistent volume and claim are included to retain data across pod restarts. The volume is mounted at `/app/data/`.

For production, replace the included `NFS` StorageClass with a cloud provider StorageClass (e.g., `gp2`, `ssd`, `azure-disk`) or use a managed database.

### Environment Variables

Backend environment variables are configured in the ConfigMap:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:///app/data/portfolio.db?mode=rwc` | SQLite connection string |

### Scaling

The backend can be horizontally scaled if needed. For SQLite this requires a different storage strategy (e.g., SQLite with WAL mode and NFS, or migration to PostgreSQL). The manifests are designed to be easily adapted for a stateless backend with a managed database.

```bash
kubectl scale deployment backend -n investment-portfolio-manager --replicas=3
```

### Scaling

The default deployment runs a single pod. For higher availability, increase replicas:

```bash
kubectl scale deployment app -n investment-portfolio-manager --replicas=3
```

Note: scaling beyond 1 replica with SQLite requires migration to a shared database (e.g., PostgreSQL). The nginx proxy pass in the frontend container makes horizontal scaling straightforward once the storage layer is externalized.

### Cleanup

```bash
kubectl delete namespace investment-portfolio-manager
```

---

## Project Structure

```
.
├── backend_rust/          # Rust backend (Rocket + SQLx)
│   ├── src/
│   │   ├── api_routes/    # HTTP route handlers
│   │   ├── engines/       # Tax engine (FIFO/LIFO/Hybrid) + analytics
│   │   └── services/      # Domain services
│   ├── Dockerfile
│   └── Cargo.toml
├── frontend/              # React + Vite frontend
│   ├── src/
│   ├── Dockerfile
│   ├── nginx.conf         # Proxies /api/* to backend sidecar
│   └── package.json
├── k8s/                   # Kubernetes manifests (single-pod deployment)
├── scripts/               # Utility scripts
└── start.sh               # Local dev launcher
```

---

## License

Apache License 2.0 — see [LICENSE](LICENSE).
