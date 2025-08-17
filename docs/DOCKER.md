# Docker Build Guide

This guide provides instructions for building and running the Rocket Blog application using Docker, particularly useful for NixOS users who may encounter build issues with native Rust compilation.

## Quick Start

### Option 1: Build from Source (Recommended)

```bash
# Build the Docker image
docker build -t rocket-blog .

# Run with docker-compose (includes database)
docker-compose up --build

# Or run standalone (requires external database)
docker run -p 8000:8000 \
  -e ROCKET_DATABASES__SEA_ORM__URL="postgres://user:pass@host/db" \
  rocket-blog
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