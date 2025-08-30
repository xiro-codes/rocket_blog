# Database Support Guide

This project now supports both PostgreSQL and SQLite databases. You can choose which database to use by modifying the connection string in your configuration.

## Supported Databases

### PostgreSQL (Recommended for Production)
- Full feature support including advanced full-text search with tsvector
- Timezone-aware timestamps
- JSON column support
- Better performance for complex queries

### SQLite (Great for Development/Testing)
- Simple setup, no external database server required
- Cross-platform compatibility
- Good performance for smaller applications
- Some features use simplified implementations (e.g., text search instead of tsvector)

## Configuration

### Using PostgreSQL

In your `Rocket.toml` file:

```toml
[default.databases.sea_orm]
url = "postgres://master:password@localhost/tdavis_dev"
```

Or via environment variable:
```bash
export ROCKET_DATABASES__SEA_ORM__URL="postgres://master:password@localhost/tdavis_dev"
```

### Using SQLite

In your `Rocket.toml` file:

```toml
[default.databases.sea_orm]
url = "sqlite:blog.db?mode=rwc"
```

Or via environment variable:
```bash
export ROCKET_DATABASES__SEA_ORM__URL="sqlite:blog.db?mode=rwc"
```

For in-memory database (useful for testing):
```toml
[default.databases.sea_orm]
url = "sqlite::memory:"
```

## Quick Start

### PostgreSQL Setup

1. Install PostgreSQL on your system
2. Create a database:
   ```sql
   CREATE DATABASE tdavis_dev;
   CREATE USER master WITH PASSWORD 'password';
   GRANT ALL PRIVILEGES ON DATABASE tdavis_dev TO master;
   ```
3. Update your `Rocket.toml` with the PostgreSQL connection string
4. Run migrations: `cargo run --bin blog` (migrations run automatically on startup)

### SQLite Setup

1. Simply update your `Rocket.toml` with a SQLite connection string
2. Run the application: `cargo run --bin blog`
3. The SQLite database file will be created automatically, and migrations will run

## Feature Differences

| Feature | PostgreSQL | SQLite |
|---------|------------|--------|
| Full-text Search | tsvector + GIN index + triggers | Simple text column with LIKE queries |
| Timestamps | With timezone support | Without timezone (timezone handled in application) |
| JSON Columns | Native JSON type | Text column with JSON stored as string |
| UUIDs | Native UUID type | Text representation |
| Performance | Optimized for concurrent access | Good for single-user/development |

## Migration Compatibility

All migrations are designed to work with both databases:

- **Database Detection**: Migrations automatically detect the database backend and apply appropriate schema changes
- **PostgreSQL-specific Features**: Advanced features like tsvector and timezone-aware timestamps are only applied when using PostgreSQL
- **SQLite Compatibility**: Simplified but functional equivalents are used for SQLite

## Switching Between Databases

You can switch between databases by:

1. Updating the connection string in `Rocket.toml`
2. Running the application (migrations will automatically set up the new database)

**Note**: Data is not automatically migrated between different database types. If you need to migrate data, you'll need to export/import manually.

## Performance Recommendations

### Development
- Use SQLite for quick local development
- No need to set up a separate database server
- Fast startup and easy to reset

### Production
- Use PostgreSQL for production deployments
- Better concurrent access handling
- Advanced features like full-text search perform better
- Better backup and recovery options

## Environment Variables

You can also control database selection via environment variables:

```bash
# PostgreSQL
export ROCKET_DATABASES__SEA_ORM__URL="postgres://user:password@localhost/db_name"

# SQLite file
export ROCKET_DATABASES__SEA_ORM__URL="sqlite:./data/blog.db?mode=rwc"

# SQLite in-memory (testing)
export ROCKET_DATABASES__SEA_ORM__URL="sqlite::memory:"
```

## Testing

The project includes tests that work with both database backends. The tests will attempt to use SQLite by default for easier CI/CD setup, but you can test with PostgreSQL by setting up a test database and updating the test configuration.

## Troubleshooting

### SQLite Issues
- **Permission errors**: Make sure the directory where the SQLite file is created has write permissions
- **Lock errors**: SQLite doesn't handle high concurrency well; consider PostgreSQL for high-traffic applications

### PostgreSQL Issues
- **Connection refused**: Make sure PostgreSQL server is running
- **Authentication failed**: Check username, password, and host in connection string
- **Database does not exist**: Create the database manually first

### Migration Issues
- **Feature not supported**: Some PostgreSQL-specific features gracefully degrade on SQLite
- If migrations fail, check the logs for specific database compatibility issues

## Examples

### Development with SQLite
```bash
# Set SQLite as database
export ROCKET_DATABASES__SEA_ORM__URL="sqlite:dev.db?mode=rwc"
cargo run --bin blog
```

### Production with PostgreSQL
```bash
# Set PostgreSQL as database
export ROCKET_DATABASES__SEA_ORM__URL="postgres://user:password@prod-db:5432/blog_prod"
cargo run --release --bin blog
```

This dual-database support allows you to use the most appropriate database for your specific use case and environment.