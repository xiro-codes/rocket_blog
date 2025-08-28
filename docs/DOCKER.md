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
just docker-dev
# OR: docker-compose -f scripts/docker/docker-compose.dev.yml up --build
# Access: http://localhost:8000
# pgAdmin: http://localhost:5050
```

**Note**: Development mode bypasses nginx and exposes the app directly, avoiding any SSL/redirect issues.

### Production (with SSL)
```bash
# First time setup: generate SSL certificates
just docker-setup-ssl
# OR: ./scripts/setup-ssl.sh

# Start the full production stack
just docker-prod
# OR: docker-compose -f scripts/docker/docker-compose.yml up --build -d
# Access: https://blog.tdavis.dev
```

**Note**: If you experience redirect loops, the nginx configuration will automatically detect missing SSL certificates and serve HTTP-only until SSL is properly configured.

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
just docker-prod
# OR: docker-compose up -d --build

# Check service status
just docker-status
# OR: docker-compose -f scripts/docker/docker-compose.yml ps

# View logs
just docker-logs nginx
just docker-logs app
# OR: docker-compose -f scripts/docker/docker-compose.yml logs nginx && docker-compose -f scripts/docker/docker-compose.yml logs app
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

For local development without SSL complexity, there are two optimized options:

### Standard Development Environment

```bash
# Use development compose file with debug builds
just docker-dev
# OR: ./scripts/docker-deploy.sh dev
# OR: docker-compose -f scripts/docker/docker-compose.dev.yml up --build

# Access points:
# - App: http://localhost:8000
# - pgAdmin: http://localhost:5050
# - Database: localhost:5432
```

### Live Development with Hot Reloading

```bash
# Use live development with automatic rebuilds
just docker-dev-live
# OR: ./scripts/docker-deploy.sh dev-live
# OR: docker-compose -f scripts/docker/docker-compose.dev.live.yml up --build

# Access points:
# - App: http://localhost:8000 (auto-reloads on code changes)
# - pgAdmin: http://localhost:5050
# - Database: localhost:5432
```

The development setups provide:
- **Debug builds** for faster compilation (30s vs 3min for release)
- **Live code reloading** with cargo-watch (live mode only)
- **Direct database access** for external tools
- **No SSL/nginx complexity** for simpler development
- **Verbose logging** optimized for debugging

For detailed development workflow documentation, see [Development Guide](DEVELOPMENT.md).

## SSL Certificate Management

### Manual Certificate Operations

```bash
# Force certificate renewal
docker-compose -f scripts/docker/docker-compose.yml exec nginx certbot renew --force-renewal

# Check certificate status
docker-compose -f scripts/docker/docker-compose.yml exec nginx certbot certificates

# Test nginx configuration
docker-compose -f scripts/docker/docker-compose.yml exec nginx nginx -t

# Reload nginx after config changes
docker-compose -f scripts/docker/docker-compose.yml exec nginx nginx -s reload
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
docker-compose -f scripts/docker/docker-compose.yml logs nginx
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

### Option 2: Using the Existing scripts/docker/.Dockerfile (Runtime-only)

If you can build locally but want to run in Docker:

```bash
# Build the application locally first
cargo build --release

# Build using the runtime-only Dockerfile
docker build -f scripts/docker/.Dockerfile -t rocket-blog-runtime .

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

### Redirect Loop Issues

If you encounter "The page isn't redirecting properly" errors:

#### Solution 1: Use Development Mode
```bash
# Bypass nginx entirely for local testing
./scripts/docker-deploy.sh dev
# Access: http://localhost:8000
```

#### Solution 2: Check SSL Certificate Status
```bash
# Check if SSL certificates exist
docker compose exec nginx ls -la /etc/letsencrypt/live/blog.tdavis.dev/

# View nginx logs for SSL-related errors
docker compose logs nginx

# Force regenerate SSL certificates
./scripts/setup-ssl.sh
```

#### Solution 3: Verify nginx Configuration
```bash
# Check which nginx config is being used
docker compose exec nginx nginx -T

# Restart nginx to trigger SSL detection
docker compose restart nginx
```

**Note**: The nginx configuration automatically detects missing SSL certificates and serves HTTP-only until SSL is properly configured, preventing redirect loops.

#### Solution 4: Pre-built Binary Approach

If SSL issues persist, you can build on a different machine and copy the binary:

1. Build on a machine without SSL issues:
   ```bash
   cargo build --release
   ```

2. Use the runtime-only Dockerfile:
   ```bash
   docker build -f scripts/docker/.Dockerfile -t rocket-blog .
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

## Container Networking

All Docker Compose configurations use a custom bridge network with static IP addresses to ensure stable connections between services:

### Network Configuration
- **Network**: `app-network` with subnet `172.20.0.0/16`
- **IP Assignments**:
  - PostgreSQL: `172.20.0.10`
  - pgAdmin: `172.20.0.20`
  - App: `172.20.0.30`
  - nginx: `172.20.0.40` (production only)
  - certbot: `172.20.0.50` (production only)

### Benefits
- **Stable pgAdmin connections**: PostgreSQL always has the same IP address (`172.20.0.10`)
- **Reliable service discovery**: No IP changes between container restarts
- **Consistent configuration**: Same network setup across development and production

### pgAdmin Configuration
When setting up database connections in pgAdmin, use:
- **Host**: `172.20.0.10` (PostgreSQL static IP)
- **Port**: `5432`
- **Database**: `tdavis_dev`
- **Username**: `master`
- **Password**: `password`

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

## Docker Volume Management and Backups

### Volume Overview

The Docker setup uses named volumes for persistent data storage:

**Production Volumes:**
- `postgres_data` - PostgreSQL database files
- `app_data` - Application uploaded files and data  
- `letsencrypt_data` - SSL certificates and Let's Encrypt data
- `certbot_webroot` - Certbot domain validation files
- `nginx_logs` - Nginx access and error logs

**Development Volumes:**
- `postgres_data` - PostgreSQL database files
- `app_data` - Application uploaded files and data

### Backup and Restore Operations

The project includes comprehensive Docker volume backup functionality:

```bash
# Backup volumes (auto-detects running environment)
./scripts/docker-deploy.sh backup

# Backup specific environment
./scripts/docker-deploy.sh backup prod
./scripts/docker-deploy.sh backup dev

# List available backups
./scripts/docker-deploy.sh backup-list

# Restore from latest backup
./scripts/docker-deploy.sh restore

# Restore specific environment
./scripts/docker-deploy.sh restore prod
./scripts/docker-deploy.sh restore dev

# Clean old backups (default: keep 7 days)
./scripts/docker-deploy.sh backup-clean

# Clean old backups (custom retention)
./scripts/docker-deploy.sh backup-clean 30
```

### Direct Backup Script Usage

For advanced usage, you can use the backup script directly:

```bash
# Direct script usage with more options
./scripts/docker-backup.sh backup [env]
./scripts/docker-backup.sh restore [env]
./scripts/docker-backup.sh restore-from /path/to/backup.tar.gz
./scripts/docker-backup.sh list
./scripts/docker-backup.sh clean [days]

# Custom backup directory
BACKUP_DIR=/custom/path ./scripts/docker-backup.sh backup
```

### Backup Contents

Each backup includes:
- **Volume data** - Complete filesystem contents of each Docker volume
- **Metadata** - Backup timestamp, environment, Docker version, volume list
- **Database dump** - Additional PostgreSQL dump for redundancy (when database is running)

Backups are stored as compressed tar.gz files in `./backups/` directory.

### Automated Backups with Systemd Timers

The production deployment automatically sets up systemd timers for daily backups at 2:00 AM.

```bash
# Production deployment automatically installs backup timers
just docker-prod

# Manually install systemd timers
just docker-backup-install-timers

# Check timer status
just docker-backup-timer-status
sudo systemctl status rocket-blog-backup.timer

# View backup service logs
sudo journalctl -u rocket-blog-backup.service

# List timer schedules
sudo systemctl list-timers rocket-blog-backup.timer

# Stop and disable automatic backups
just docker-backup-timer-stop
```

The systemd timer:
- Runs daily at 2:00 AM with a random delay up to 5 minutes
- Automatically backs up production volumes
- Cleans backups older than 7 days
- Logs all operations to the system journal
- Includes security restrictions for safer execution

### Volume Content Inspection

For easy inspection of volume contents, you can export Docker volumes to a readable folder structure:

```bash
# Export volumes for inspection (auto-detects running environment)
./scripts/docker-deploy.sh inspect

# Export specific environment volumes
./scripts/docker-deploy.sh inspect prod
./scripts/docker-deploy.sh inspect dev

# Or use the backup script directly
./scripts/docker-backup.sh inspect dev

# Using justfile commands
just docker-inspect
just docker-inspect dev
```

**What gets exported:**
- Each volume is extracted to its own subdirectory
- Files are preserved with original permissions and structure
- No compression - browse and edit files directly
- Includes inspection metadata with volume information
- Exports are timestamped for multiple snapshots

**Example usage:**
```bash
# Export development volumes
./scripts/docker-deploy.sh inspect dev

# Browse exported postgres data
ls -la ./backups/volume_inspections/volumes_dev_20241201_120000/postgres_data/

# View inspection summary
cat ./backups/volume_inspections/volumes_dev_20241201_120000/inspection_info.txt

# Edit a configuration file directly
nano ./backups/volume_inspections/volumes_dev_20241201_120000/nginx_logs/access.log
```

**Exports are stored in:** `./backups/volume_inspections/volumes_{env}_{timestamp}/`

### Volume Inspection and Troubleshooting

```bash
# List all volumes
docker volume ls

# Inspect specific volume
docker volume inspect rocket_blog_postgres_data

# View volume contents
docker run --rm -v rocket_blog_postgres_data:/data alpine ls -la /data

# Check volume disk usage
docker system df -v

# Remove unused volumes (careful!)
docker volume prune
```

### Data Recovery Scenarios

**Complete Environment Recovery:**
```bash
# 1. Stop services
./scripts/docker-deploy.sh stop

# 2. Restore volumes
./scripts/docker-deploy.sh restore prod

# 3. Start services
./scripts/docker-deploy.sh prod
```

**Selective Database Recovery:**
```bash
# If you have both volume backup and database dump
# 1. Restore just the database volume
./scripts/docker-backup.sh restore-from backup_file.tar.gz

# 2. Or import from SQL dump
docker-compose exec postgres psql -U master -d tdavis_dev < database_backup.sql
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