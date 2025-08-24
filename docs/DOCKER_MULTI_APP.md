# Docker Multi-App Setup

This document describes how to run the multi-app workspace using Docker Compose.

## Overview

The workspace now supports running multiple applications simultaneously:
- **Blog Application**: Runs on port 8000 (main blog functionality)
- **Hello World Application**: Runs on port 8001 (demo/selection interface)

Both applications share the same database and infrastructure but run as separate services.

## Quick Start

### Development Mode

Run both applications in development mode:
```bash
just docker-dev-multi
```

This will:
- Build both applications in debug mode
- Start PostgreSQL database
- Start pgAdmin on port 5050
- Expose blog app on http://localhost:8000
- Expose hello-world app on http://localhost:8001

### Development with Live Reload

Run with volume mounts for live template/static file editing:
```bash
just docker-dev-live-multi
```

### Production Mode

Run both applications in production mode with nginx:
```bash
just docker-prod-multi
```

This will:
- Build both applications in release mode
- Start nginx reverse proxy on ports 80/443
- Route traffic to both applications:
  - Main site: https://yourdomain.com → Blog App
  - Hello World: https://yourdomain.com/hello-world → Hello World App

## Service Architecture

### Services

1. **postgres**: PostgreSQL database (shared by both apps)
2. **blog-app**: Blog application container
3. **hello-world-app**: Hello World application container  
4. **nginx**: Reverse proxy (production only)
5. **pgadmin**: Database administration interface

### Network Configuration

- Network: `app-network` (172.20.0.0/16)
- postgres: 172.20.0.10:5432
- pgadmin: 172.20.0.20:80
- blog-app: 172.20.0.30:8000
- hello-world-app: 172.20.0.31:8001
- nginx: 172.20.0.40:80/443

### Data Persistence

- `postgres_data`: Database files
- `blog_data`: Blog application data and uploads
- `hello_world_data`: Hello World application data
- `letsencrypt_data`: SSL certificates (production)
- `nginx_logs`: Nginx access/error logs (production)

## Nginx Routing

In production, nginx routes requests as follows:

- `GET /` → Blog Application (port 8000)
- `GET /hello-world/*` → Hello World Application (port 8001)
- Static files served by each application

## Individual Application Management

### Build Individual Apps
```bash
# Build blog application only
docker build -f scripts/docker/Dockerfile.blog -t blog-app .

# Build hello-world application only  
docker build -f scripts/docker/Dockerfile.hello-world -t hello-world-app .
```

### Run Individual Apps
```bash
# Run blog app only
docker run -p 8000:8000 --env ROCKET_DATABASES__SEA_ORM__URL="postgres://user:pass@host/db" blog-app

# Run hello-world app only
docker run -p 8001:8001 hello-world-app
```

## Environment Variables

Both applications support these environment variables:

- `ROCKET_DATABASES__SEA_ORM__URL`: Database connection string
- `RUST_LOG`: Logging level (debug, info, warn, error)
- `ROCKET_ENV`: Environment (development, production)

## Troubleshooting

### Port Conflicts
If ports 8000 or 8001 are in use, modify the port mappings in the docker-compose files.

### Build Issues
If you encounter SSL or network issues during build:
1. Try the development Dockerfiles which include error handling
2. Use `--build-arg BUILDKIT_INLINE_CACHE=1` for better caching

### Database Connection Issues
Ensure the postgres service is healthy before applications start:
```bash
docker-compose logs postgres
```

### View Application Logs
```bash
# View all logs
docker-compose logs

# View specific application logs
docker-compose logs blog-app
docker-compose logs hello-world-app
```

## Legacy Support

The original single-app Docker commands still work and default to the blog application:
- `just docker-dev` → Single blog app in development mode
- `just docker-prod` → Single blog app in production mode

Use the new `-multi` commands to run both applications simultaneously.