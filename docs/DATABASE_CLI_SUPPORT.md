# Database Auto-Fallback and CLI Support

This update adds intelligent database switching with auto-fallback capabilities to the rocket_blog application.

## New Features

### 1. CLI Arguments

The application now supports command-line arguments for database selection:

```bash
# Use PostgreSQL (default)
cargo run --bin blog

# Use SQLite database file
cargo run --bin blog -- --database sqlite

# Use in-memory SQLite (data lost on restart)
cargo run --bin blog -- --database memory

# Enable auto-fallback (tries PostgreSQL, falls back to memory if unavailable)
cargo run --bin blog -- --auto-fallback

# View help
cargo run --bin blog -- --help
```

### 2. Auto-Fallback Logic

When auto-fallback is enabled (either via `--auto-fallback` flag or by default when no specific database is chosen):

1. **Primary Attempt**: Tries to connect to PostgreSQL using the configured URL
2. **Fallback**: If PostgreSQL fails, automatically switches to in-memory SQLite
3. **Warning**: Shows clear warning when using fallback database

### 3. Database Detection

The application now:
- Tests database connectivity before starting the server
- Provides clear feedback about which database is being used
- Shows warnings for non-persistent databases (in-memory SQLite)
- Gracefully handles connection failures

## Usage Examples

### Development Mode (No PostgreSQL Required)
```bash
# Auto-fallback enabled by default - will use memory if PostgreSQL unavailable
cargo run --bin blog
```

### Explicit Database Selection
```bash
# Force PostgreSQL (will fail if not available)
cargo run --bin blog -- --database postgres

# Use persistent SQLite file
cargo run --bin blog -- --database sqlite

# Use temporary in-memory database
cargo run --bin blog -- --database memory
```

### Environment Variable Override
```bash
# Set database URL via environment variable
export ROCKET_DATABASES__SEA_ORM__URL="sqlite:my_blog.db?mode=rwc"
cargo run --bin blog

# Or use DATABASE_URL
export DATABASE_URL="postgres://user:pass@localhost/mydb"
cargo run --bin blog
```

## Technical Implementation

### New Components

1. **`src/database.rs`**: Database configuration and auto-fallback logic
2. **`create_base_rocket_with_database()`**: New function supporting dynamic database configuration
3. **CLI argument parsing**: Using clap for command-line argument handling

### Database Types Supported

- **PostgreSQL**: Full production-ready database with all features
- **SQLite**: Persistent file-based database for development/small deployments
- **SQLite Memory**: Temporary database for testing (data lost on restart)

### Backward Compatibility

- Existing `create_base_rocket()` function unchanged
- Existing `Rocket.toml` configuration still works
- No breaking changes to existing deployments

## Benefits for Developers

1. **Easy Setup**: No need to install PostgreSQL for local development
2. **Quick Testing**: Memory database for rapid iteration
3. **Flexible Deployment**: Choose database based on environment needs
4. **Clear Feedback**: Know exactly which database is being used
5. **Fail-Safe**: Auto-fallback prevents complete failure if PostgreSQL unavailable

## Warning Messages

When using fallback or memory databases, the application shows clear warnings:

```
⚠️  Using in-memory database - data will not persist between restarts!
```

This helps developers understand the implications of their database choice.