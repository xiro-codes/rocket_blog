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