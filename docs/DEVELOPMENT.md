# Development Docker Setup

This directory contains optimized Docker configurations for development workflows.

## Quick Start

### Option 1: Standard Development Environment

For development with debug builds (faster compilation):

```bash
# Start development environment (using just)
just docker-dev

# OR using the script directly
./scripts/docker-deploy.sh dev

# OR directly with docker-compose
docker-compose -f docker-compose.dev.yml up --build
```

**Features:**
- Debug builds (faster compilation than release builds)
- App exposed on http://localhost:8000
- PostgreSQL database on localhost:5432
- pgAdmin on http://localhost:5050
- Optimized for development with proper logging

### Option 2: Live Development with Template/Static File Reloading

For live development with automatic template and static file reloading:

```bash
# Start development with live file reloading (using just)
just docker-dev-live

# OR using the script directly
./scripts/docker-deploy.sh dev-live

# OR directly with docker-compose
docker-compose -f docker-compose.dev.live.yml up --build
```

**Features:**
- All features from standard development environment
- Template and static file changes are immediately visible (no rebuild needed)
- Ideal for frontend development and template modifications

## Development Files

### Docker Configuration Files

- **`docker-compose.dev.yml`** - Standard development environment
  - Uses `Dockerfile.dev` for faster debug builds
  - Good for testing built application
  
- **`docker-compose.dev.live.yml`** - Live development environment  
  - Mounts templates and static files for live editing
  - Immediate updates for frontend changes (no rebuild needed)
  - Best for template and design development

- **`Dockerfile`** - Production-ready Dockerfile used for all builds
  - Multi-stage build for optimized production images
  - Works consistently across all platforms including NixOS
  - Used by both development and production docker-compose setups

### Helper Scripts

- **`scripts/docker-deploy.sh`** - Management script with commands:
  - `dev` - Start standard development environment
  - `dev-live` - Start live development environment
  - `prod` - Start production environment
  - `stop` - Stop all services
  - `clean` - Remove all containers and volumes

## Development Workflow

### For Quick Testing

1. Start the standard development environment:
   ```bash
   ./scripts/docker-deploy.sh dev
   ```

2. Make code changes and rebuild when needed:
   ```bash
   docker-compose -f docker-compose.dev.yml up --build
   ```

### For Active Development

1. Start the live development environment:
   ```bash
   ./scripts/docker-deploy.sh dev-live
   ```

2. Edit your Rust code - changes will automatically trigger rebuilds

3. View live logs to see compilation and runtime output:
   ```bash
   ./scripts/docker-deploy.sh logs app
   ```

## Environment Variables

The development environment includes these optimized settings:

- `RUST_LOG=debug` - Verbose logging for development
- `ROCKET_ENV=development` - Rocket development mode
- `ROCKET_DATABASES__SEA_ORM__URL` - PostgreSQL connection string

## Performance Notes

### Debug vs Release Builds

- **Debug builds** (development): ~30 seconds compile time
- **Release builds** (production): ~2-3 minutes compile time

For development, debug builds provide much faster iteration while maintaining full functionality.

### Live Reloading Performance

- File watching uses `cargo-watch` with efficient change detection
- Only affected modules are recompiled when possible
- Template and static file changes are immediate (no rebuild needed)

## Troubleshooting

### Port Conflicts

If ports 8000, 5432, or 5050 are already in use:

```bash
# Check what's using the ports
lsof -i :8000
lsof -i :5432  
lsof -i :5050

# Stop existing services
./scripts/docker-deploy.sh stop
```

### Build Issues

If you encounter build errors:

1. Clean and rebuild:
   ```bash
   ./scripts/docker-deploy.sh clean
   ./scripts/docker-deploy.sh dev
   ```

2. Check logs for specific errors:
   ```bash
   ./scripts/docker-deploy.sh logs app
   ```

### Live Reloading Not Working

1. Ensure you're using the live development setup:
   ```bash
   ./scripts/docker-deploy.sh dev-live
   ```

2. Check that cargo-watch is running:
   ```bash
   docker-compose -f docker-compose.dev.live.yml logs app
   ```

## Comparison with Production

| Feature | Development | Production |
|---------|-------------|------------|
| Build Type | Debug | Release |
| Compile Time | ~30s | ~2-3min |
| SSL/HTTPS | No | Yes |
| Nginx | No | Yes |
| Hot Reload | Yes (live) | No |
| Database Access | Direct | Through app |
| Logging | Verbose | Production |
| Performance | Lower | Optimized |

The development setup prioritizes fast iteration and debugging capabilities, while production focuses on performance and security.