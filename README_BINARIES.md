# Running the Separate Binaries

The application has been split into two separate binaries:

## Blog Binary (`blog`)

Contains the main blog functionality including:
- Blog posts and content management
- User authentication and settings  
- Comments and reactions
- SEO optimization
- AI services integration
- Background job processing

**To run:**
```bash
cargo run --bin blog
# or for release build:
cargo run --release --bin blog
```

## Work Time Tracker Binary (`worktime`)

Contains the Progressive Web App for work time tracking including:
- Time tracking with role-based wages
- Configurable notifications
- Pay period management
- PWA capabilities with offline support
- Independent authentication and operation

**To run:**
```bash
cargo run --bin worktime
# or for release build:
cargo run --release --bin worktime
```

## Building Both Binaries

To build both binaries at once:
```bash
cargo build --release --bin blog --bin worktime
```

To check both binaries compile:
```bash
cargo check --bin blog --bin worktime
```

## Binary Locations

After building, the binaries will be located at:
- `target/release/blog` (or `target/debug/blog` for debug builds)
- `target/release/worktime` (or `target/debug/worktime` for debug builds)

## Shared Components

Both binaries share:
- Database connection and migrations
- Authentication services and guards
- Models and database schemas
- Common configuration
- Base services and types

## Default Ports

Both applications use the same configuration system, so they would try to bind to the same port if run simultaneously. To run both at the same time, you'll need to configure different ports in your `Rocket.toml` or environment variables.

Example `Rocket.toml` for running on different ports:
```toml
[default]
port = 8000

[worktime]
port = 8001
```

Then run with profiles:
```bash
ROCKET_PROFILE=default cargo run --bin blog      # runs on port 8000
ROCKET_PROFILE=worktime cargo run --bin worktime # runs on port 8001
```

## Docker Deployment

The dual-binary architecture supports containerized deployment with several Docker configurations:

### Production Deployment
```bash
# Build and run both services with nginx proxy
cd scripts/docker/
docker-compose up -d

# Blog accessible at: http://localhost/
# Work time tracker accessible at: http://localhost/worktime/
```

### Development Deployment
```bash
# Build and run both services for development
cd scripts/docker/
docker-compose -f docker-compose.dev.yml up -d

# Blog accessible at: http://localhost/
# Work time tracker accessible at: http://localhost/worktime/
```

### Individual Service Deployment
```bash
# Build the Docker image
docker build -f scripts/docker/Dockerfile -t rocket-blog .

# Run blog service only
docker run -d -p 8000:8000 --name blog-service rocket-blog ./blog

# Run work time tracker service only  
docker run -d -p 8001:8001 --name worktime-service rocket-blog ./worktime
```

### Container-Only Features

All Docker configurations are designed to be completely container-isolated:
- ✅ No host PC port exposures (except through nginx)
- ✅ All services communicate within Docker networks
- ✅ Independent scaling for each binary
- ✅ Shared database and file storage
- ✅ Automatic SSL certificate management (production)
- ✅ Container-native logging and monitoring

### Available Docker Files

- `Dockerfile` - Production multi-stage build with both binaries
- `Dockerfile.dev` - Development build with faster compile times
- `Dockerfile.test` - Test environment for both binaries
- `Dockerfile.coverage` - Code coverage analysis with Tarpaulin
- `docker-compose.yml` - Production deployment with nginx and SSL
- `docker-compose.dev.yml` - Development deployment with nginx
- `docker-compose.dev.live.yml` - Container-only live development