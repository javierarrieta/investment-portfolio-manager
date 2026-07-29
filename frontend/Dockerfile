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

# Remove default nginx site and config
RUN rm -rf /usr/share/nginx/html/* /etc/nginx/conf.d/default.conf

# Copy built frontend assets from builder stage
COPY --from=builder /build/dist /usr/share/nginx/html

# Copy custom nginx config (proxies /api and /api-docs to the backend sidecar)
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

ENTRYPOINT ["nginx", "-g", "daemon off;"]
