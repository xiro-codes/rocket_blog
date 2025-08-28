# Multi-App Workspace Documentation

## Overview

This project has been restructured to support multiple applications using shared code, database migrations, and models. The new structure allows for easy development of multiple applications that share common functionality and database schema.

## Project Structure

```
rocket_blog/
├── Cargo.toml                    # Workspace root configuration
├── justfile                      # Build commands for all apps
├── apps/                         # Applications directory
│   ├── blog/                     # Original blog application
│   │   ├── Cargo.toml
│   │   ├── Rocket.toml
│   │   ├── src/
│   │   └── templates/
│   └── hello-world/              # New hello world selection app
│       ├── Cargo.toml
│       ├── Rocket.toml
│       ├── src/
│       └── templates/
├── shared/                       # Shared components
│   ├── common/                   # Common utilities, config, database
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── models/                   # Database models
│   │   ├── Cargo.toml
│   │   └── src/
│   └── migrations/               # Database migrations
│       ├── Cargo.toml
│       └── src/
└── static/                       # Shared static assets
```

## Applications

### 1. Blog App (`apps/blog/`)
- **Purpose**: The original Rocket Blog application with full features
- **Features**: Authentication, blog posts, comments, tags, reactions, AI integration
- **Port**: 8000 (default)
- **Database**: Uses shared database schema

### 2. Hello World Selection App (`apps/hello-world/`)
- **Purpose**: Demonstration app showing shared structure usage
- **Features**: 
  - Simple landing page
  - Hello world page
  - Interactive selection system with 3 options
  - JSON API endpoint for selections
- **Port**: 8000 (default, run separately from blog)
- **Database**: Can use shared database schema (currently runs without DB for demo)

## Shared Components

### Common Library (`shared/common/`)
- **Database**: Shared database connection and migration runner
- **Config**: Application configuration structures
- **Utils**: Logging and utility functions

### Models (`shared/models/`)
- **Purpose**: Shared database entity models
- **Generated**: Using SeaORM CLI from database schema
- **DTOs**: Custom data transfer objects and form structures

### Migrations (`shared/migrations/`)
- **Purpose**: Database schema migrations
- **Tool**: SeaORM migrations
- **Shared**: All apps use the same database schema

## Build Commands

The `justfile` has been updated to support multi-app development:

### Database Commands (Shared)
```bash
just migrate              # Run migrations
just force-migrate        # Fresh migration (drops all data)
just new-migration NAME   # Create new migration
just gen-models          # Generate SeaORM models
```

### Build Commands
```bash
just build-all           # Build all applications (release)
just build-all-dev       # Build all applications (debug)
just build-blog          # Build only blog app
just build-hello-world   # Build only hello-world app
```

### Run Commands
```bash
just run-blog           # Run blog application
just run-hello-world    # Run hello-world application
```

### Test Commands
```bash
just test-all           # Test all applications
just test-blog          # Test blog application
just test-hello-world   # Test hello-world application
```

### Legacy Commands
For backward compatibility, these commands still work:
```bash
just build              # Same as build-blog
just run                # Same as run-blog
just test               # Same as test-all
```

## Development Workflow

### Adding a New Application

1. **Create app directory**:
   ```bash
   mkdir -p apps/new-app/src
   ```

2. **Create Cargo.toml**:
   ```toml
   [package]
   name = "new-app"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   common = { path = "../../shared/common" }
   models = { path = "../../shared/models" }
   migrations = { path = "../../shared/migrations" }
   rocket = { version = "0.5", features = ["http2", "json", "uuid", "secrets"] }
   # ... other dependencies
   ```

3. **Update workspace** in root `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       "shared/models", 
       "shared/migrations", 
       "shared/common",
       "apps/blog",
       "apps/hello-world",
       "apps/new-app"  # Add here
   ]
   ```

4. **Create main.rs** following the hello-world pattern
5. **Add justfile commands** for the new app

### Using Shared Components

In your application's `main.rs`:
```rust
use common::database::{run_migrations, Db};
use common::utils::setup_logger;

#[launch]
async fn rocket() -> _ {
    let _ = setup_logger();
    
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        // ... your routes
}
```

## Database Setup

Both applications can share the same database:

1. **Set up PostgreSQL**:
   ```sql
   CREATE DATABASE rocket_blog;
   CREATE USER admin WITH PASSWORD 'admin';
   GRANT ALL PRIVILEGES ON DATABASE rocket_blog TO admin;
   ```

2. **Configure Rocket.toml** in each app:
   ```toml
   [app.sea_orm]
   url = "postgres://admin:admin@localhost:5432/rocket_blog"
   max_connections = 100
   min_connections = 5
   ```

3. **Run migrations**:
   ```bash
   just migrate
   ```

## Screenshots

The hello-world application includes:
- Landing page with navigation
- Hello world page
- Interactive selections page with 3 options
- JSON API endpoint

All pages use responsive Bootstrap design and work without requiring a database connection for demonstration purposes.

## Benefits

1. **Code Reuse**: Shared database models, utilities, and configurations
2. **Consistency**: All apps use the same database schema and patterns
3. **Maintainability**: Changes to shared components benefit all apps
4. **Scalability**: Easy to add new applications to the workspace
5. **Development Speed**: New apps can be created quickly using shared components

## Next Steps

- Add more example applications
- Create shared middleware components
- Add shared authentication system
- Create shared UI components library
- Add integration tests for multi-app scenarios