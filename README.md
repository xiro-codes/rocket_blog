# Rocket Blog

A modern, full-featured blog application built with Rust using the Rocket web framework and SeaORM for database operations.

## Features

- 📝 **Blog Management**: Create, read, update, and delete blog posts
- 👥 **User Authentication**: Secure login/logout system with password hashing  
- 💬 **Comment System**: Users can comment on blog posts
- 🔒 **Admin Panel**: Administrative features for managing content
- 📄 **Markdown Support**: Write posts using Markdown syntax
- 🎨 **Responsive Templates**: Clean, mobile-friendly UI using Tera templates
- 🗄️ **Database**: PostgreSQL with SeaORM for type-safe database operations
- 🚀 **Performance**: Built with Rust for maximum performance and safety

## Architecture

This application follows a clean architecture pattern with clear separation of concerns:

```
src/
├── main.rs              # Application entry point
├── controllers/         # HTTP request handlers  
│   ├── auth.rs         # Authentication endpoints
│   ├── blog.rs         # Blog post endpoints
│   ├── comment.rs      # Comment endpoints
│   └── index.rs        # Home page routing
├── services/           # Business logic layer
│   ├── auth.rs         # Authentication services
│   ├── blog.rs         # Blog post services
│   └── comment.rs      # Comment services
├── middleware/         # Custom middleware
├── dto/               # Data transfer objects
├── types/             # Custom type definitions
└── pool.rs            # Database connection pool

models/                 # Database models (separate crate)
├── src/
│   ├── account.rs     # User account model
│   ├── post.rs        # Blog post model  
│   └── comment.rs     # Comment model

migrations/            # Database migrations (separate crate)
└── src/
    └── m*.rs         # Migration files

templates/             # Tera HTML templates
static/               # CSS, JS, and other static assets
```

## Prerequisites

- Rust 1.70+ 
- PostgreSQL 12+
- [sea-orm-cli](https://github.com/SeaQL/sea-orm) for database operations

## Getting Started

### 1. Clone the Repository

```bash
git clone <repository-url>
cd rocket_blog
```

### 2. Setup Database

Create a PostgreSQL database and update your database URL in `Rocket.toml`:

```toml
[default.databases.sea_orm]
url = "postgres://username:password@localhost/rocket_blog"
```

### 3. Install Dependencies

```bash
cargo build
```

### 4. Run Migrations

```bash
# Using justfile (recommended)
just migrate

# Or directly with sea-orm-cli
sea-orm-cli migrate -d migrations
```

### 5. Generate Models (if needed)

```bash
# Using justfile  
just gen-models

# Or directly with sea-orm-cli
sea-orm-cli generate entity -l --model-extra-attributes 'serde(crate="rocket::serde")' --with-serde both --with-copy-enums -o ./models/src
```

### 6. Run the Application

```bash
cargo run
```

The application will start on `http://localhost:8000` by default.

## API Endpoints

### Authentication
- `POST /auth` - Login with credentials
- `GET /auth/logout` - Logout current user

### Blog Posts  
- `GET /blog` - List all blog posts (paginated)
- `GET /blog/<id>` - View a specific blog post
- `GET /blog/new` - Show new post form (authenticated)
- `POST /blog/new` - Create a new blog post (authenticated)
- `GET /blog/edit/<id>` - Show edit form (authenticated)
- `POST /blog/edit/<id>` - Update a blog post (authenticated)
- `POST /blog/delete/<id>` - Delete a blog post (authenticated)

### Comments
- `POST /comment` - Add a comment to a blog post
- `POST /comment/delete/<id>` - Delete a comment (authenticated)

### Static Assets
- `GET /static/*` - Serve static files (CSS, JS, images)

## Development

### Available Commands (using [just](https://github.com/casey/just))

```bash
# Database operations
just migrate                    # Run pending migrations
just force-migrate             # Fresh migration (drops all data)  
just new-migration <NAME>      # Create a new migration
just gen-models                # Regenerate model files from database

# Development
cargo run                      # Start the application
cargo test                     # Run tests
cargo fmt                      # Format code
cargo clippy                   # Run linter
```

### Code Organization

- **Controllers**: Handle HTTP requests/responses, parameter validation
- **Services**: Contain business logic, database operations  
- **Models**: Database entities with relationships
- **DTOs**: Data transfer objects for API boundaries
- **Middleware**: Cross-cutting concerns like authentication, logging

### Database Schema

The application uses three main entities:

- **Account**: User accounts with authentication
- **Post**: Blog posts with metadata and content
- **Comment**: Comments linked to posts and users

All entities use UUIDs as primary keys for better security and distribution.

## Configuration

Configuration is handled through `Rocket.toml`:

- Database connection settings
- Server port and address
- Template directories  
- Static file serving
- Secret keys for cookies/sessions

## Docker Support

A `docker-compose.yml` is included for easy development setup with PostgreSQL.

```bash
docker-compose up -d postgres  # Start database only
docker-compose up              # Start full application stack
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with appropriate tests
4. Run `cargo fmt` and `cargo clippy`
5. Commit your changes (`git commit -am 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Technology Stack

- **Backend**: [Rocket](https://rocket.rs/) - Fast, secure web framework
- **Database**: [SeaORM](https://www.sea-ql.org/SeaORM/) - Async ORM for Rust
- **Templates**: [Tera](https://tera.netlify.app/) - Template engine inspired by Jinja2
- **Database**: PostgreSQL - Robust, open-source relational database
- **Authentication**: Custom JWT-based authentication with secure password hashing
- **Frontend**: HTML5, CSS3, JavaScript (vanilla)