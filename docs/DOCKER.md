# Docker Build Guide

This guide provides instructions for building and running the Rocket Blog application using Docker, with production-ready nginx reverse proxy, automatic SSL certificates, and special consideration for NixOS users who may encounter build issues with native Rust compilation.

## Table of Contents
- [Quick Start](#quick-start)
- [Production Deployment with SSL](#production-deployment-with-ssl)
- [Development Setup](#development-setup)
- [SSL Certificate Management](#ssl-certificate-management)
- [Build Strategies](#build-strategies)
- [Environment Configuration](#environment-configuration)
- [Troubleshooting](#troubleshooting)
- [NixOS Specific Instructions](#nixos-specific-instructions)

## Quick Start

### Development (Local Testing)
```bash
# For local development with direct app access
docker-compose -f docker-compose.dev.yml up --build
# Access: http://localhost:8000
# pgAdmin: http://localhost:5050
```

### Production (with SSL)
```bash
# First time setup: generate SSL certificates
./scripts/setup-ssl.sh

# Start the full production stack
docker-compose up --build -d
# Access: https://blog.tdavis.dev
```

## Production Deployment with SSL

The production setup includes nginx reverse proxy with automatic SSL certificate generation and renewal using Let's Encrypt.

### Prerequisites
- Domain name pointing to your server (blog.tdavis.dev)
- Ports 80 and 443 open on your server
- Docker and docker-compose installed

### Initial Setup

1. **Clone and configure:**
```bash
git clone <repository>
cd rocket_blog
```

2. **Generate SSL certificates:**
```bash
# Run the SSL setup script (first time only)
./scripts/setup-ssl.sh
```

3. **Start production stack:**
```bash
# Start all services with nginx proxy
docker-compose up -d --build

# Check service status
docker-compose ps

# View logs
docker-compose logs nginx
docker-compose logs app
```

### Services in Production
- **nginx**: Reverse proxy with SSL termination (ports 80/443)
- **app**: Rocket blog application (internal only)
- **postgres**: Database (internal only)
- **pgAdmin**: Database admin interface (port 5050, optional)

### SSL Certificate Auto-Renewal
The nginx container automatically:
- Checks for certificate renewal every 12 hours
- Renews certificates 30 days before expiry
- Reloads nginx configuration after renewal

## Development Setup

For local development without SSL complexity:

```bash
# Use development compose file
docker-compose -f docker-compose.dev.yml up --build

# Access points:
# - App: http://localhost:8000
# - pgAdmin: http://localhost:5050
# - Database: localhost:5432
```

The development setup:
- Exposes app directly on port 8000
- Includes test data seeding (debug builds)
- Exposes database port for external tools
- No SSL/nginx complexity

## SSL Certificate Management

### Manual Certificate Operations

```bash
# Force certificate renewal
docker-compose exec nginx certbot renew --force-renewal

# Check certificate status
docker-compose exec nginx certbot certificates

# Test nginx configuration
docker-compose exec nginx nginx -t

# Reload nginx after config changes
docker-compose exec nginx nginx -s reload
```

### Certificate Troubleshooting

If certificate generation fails:

1. **Check domain DNS:**
```bash
nslookup blog.tdavis.dev
```

2. **Verify port 80 is accessible:**
```bash
curl -I http://blog.tdavis.dev/.well-known/acme-challenge/test
```

3. **Check nginx logs:**
```bash
docker-compose logs nginx
```

4. **Re-run setup:**
```bash
# Remove existing certificates and try again
docker volume rm rocket-blog_letsencrypt_data
./scripts/setup-ssl.sh
```

### Manual Certificate Generation

If the automated script fails, you can generate certificates manually:

```bash
# Create volumes
docker volume create letsencrypt_data
docker volume create certbot_webroot

# Generate certificate manually
docker run --rm \
  -v letsencrypt_data:/etc/letsencrypt \
  -v certbot_webroot:/var/www/certbot \
  -p 80:80 \
  certbot/certbot \
  certonly \
  --standalone \
  --email me@tdavis.dev \
  --agree-tos \
  --no-eff-email \
  -d blog.tdavis.dev
```

### Option 2: Using the Existing .Dockerfile (Runtime-only)

If you can build locally but want to run in Docker:

```bash
# Build the application locally first
cargo build --release

# Build using the runtime-only Dockerfile
docker build -f .Dockerfile -t rocket-blog-runtime .

# Run the container
docker run -p 8000:8000 rocket-blog-runtime
```

## Build Troubleshooting

### SSL Certificate Issues

If you encounter SSL certificate issues during the Docker build (common in some CI/CD environments), try these solutions:

#### Solution 1: Use a Different Base Image

Edit the `Dockerfile` to use a different Rust base image:

```dockerfile
# Try slim variant
FROM rust:1.89-slim-bookworm AS builder

# Or try Alpine
FROM rust:1.89-alpine AS builder
RUN apk add --no-cache musl-dev
```

#### Solution 2: Build with Host Network

```bash
# Build with host network to use host's SSL certificates
docker build --network=host -t rocket-blog .
```

#### Solution 3: Disable SSL Verification (Development Only)

Add to the builder stage in `Dockerfile`:

```dockerfile
# Add before cargo build --release
ENV CARGO_HTTP_CHECK_REVOKE=false
RUN mkdir -p ~/.cargo && \
    echo '[http]' > ~/.cargo/config.toml && \
    echo 'check-revoke = false' >> ~/.cargo/config.toml
```

#### Solution 4: Pre-built Binary Approach

If SSL issues persist, you can build on a different machine and copy the binary:

1. Build on a machine without SSL issues:
   ```bash
   cargo build --release
   ```

2. Use the runtime-only Dockerfile:
   ```bash
   docker build -f .Dockerfile -t rocket-blog .
   ```

### NixOS Specific Instructions

For NixOS users who can't build locally:

#### Option 1: Use GitHub Actions / CI

Set up GitHub Actions to build the Docker image:

```yaml
# .github/workflows/docker-build.yml
name: Build Docker Image
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Docker image
        run: docker build -t rocket-blog .
      - name: Save Docker image
        run: docker save rocket-blog | gzip > rocket-blog.tar.gz
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: docker-image
          path: rocket-blog.tar.gz
```

Then download and load the image:
```bash
docker load < rocket-blog.tar.gz
```

#### Option 2: Use Remote Docker Build

Use a remote Docker daemon or Docker BuildKit:

```bash
# Using buildx for multi-platform builds
docker buildx create --use
docker buildx build --platform linux/amd64 -t rocket-blog .
```

#### Option 3: Cross-compilation

Build for Linux on NixOS using cross-compilation:

```bash
# In your flake.nix or nix-shell
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu --release

# Then use the runtime Dockerfile with the cross-compiled binary
```

## Environment Variables

The Docker containers support these environment variables:

- `ROCKET_DATABASES__SEA_ORM__URL`: Database connection string
- `ROCKET_DATA_PATH`: Directory for uploaded files (default: `/app/data`)
- `ROCKET_SECRET_KEY`: Secret key for sessions
- `ROCKET_ADDRESS`: Bind address (default: `0.0.0.0`)
- `ROCKET_PORT`: Port to listen on (default: `8000`)

## Docker Compose Configuration

The included `docker-compose.yml` provides a complete setup with PostgreSQL:

```yaml
services:
  app:
    build: .
    ports:
      - "8000:8000"
    environment:
      ROCKET_DATABASES__SEA_ORM__URL: "postgres://master:password@postgres/tdavis_dev"
    depends_on:
      - postgres
  
  postgres:
    image: postgres:13
    environment:
      POSTGRES_USER: master
      POSTGRES_PASSWORD: password
      POSTGRES_DB: tdavis_dev
    volumes:
      - postgres_data:/var/lib/postgresql/data
```

To enable the app service, uncomment the `app` section in `docker-compose.yml`.

## Volume Mounts

For development, you can mount the source code:

```bash
docker run -p 8000:8000 \
  -v $(pwd)/static:/app/static \
  -v $(pwd)/templates:/app/templates \
  -v $(pwd)/data:/app/data \
  rocket-blog
```

## Security Considerations

- The application runs as a non-root user (`app`) inside the container
- Static files are served directly by the application
- Database credentials should be provided via environment variables or Docker secrets
- For production, use a reverse proxy (nginx/traefik) in front of the application

## Performance Optimization

For production builds:

```bash
# Build with optimized Rust flags
docker build --build-arg RUSTFLAGS="-C target-cpu=native" -t rocket-blog .
```

## Multi-architecture Builds

To build for multiple architectures:

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t rocket-blog .
```

## Deployment

### Development
```bash
docker-compose up --build
```

### Production
```bash
# Build and tag for production
docker build -t rocket-blog:latest .
docker tag rocket-blog:latest your-registry/rocket-blog:latest
docker push your-registry/rocket-blog:latest

# Deploy with production configuration
docker run -d \
  --name rocket-blog \
  -p 8000:8000 \
  -e ROCKET_DATABASES__SEA_ORM__URL="postgres://user:pass@prod-db/rocket_blog" \
  -e ROCKET_SECRET_KEY="your-secret-key" \
  --restart unless-stopped \
  your-registry/rocket-blog:latest
```