# Rocket Blog

A modern, fast, and feature-rich blog application built with **Rust** and the **Rocket** web framework. This blog platform provides a clean interface for content management with markdown support, user authentication, commenting system, and a powerful tagging system.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Rust Version](https://img.shields.io/badge/rust-1.70+-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## ✨ Features

### Current Features
- 📝 **Blog Management** - Create, edit, delete, and publish blog posts with markdown support
- 🔐 **Authentication System** - Secure login/logout with admin privileges
- 💬 **Comment System** - Enable readers to comment on blog posts
- 🎬 **Media Support** - Video streaming with range requests for optimal performance
- 📄 **Pagination** - Efficient navigation through blog posts
- 📑 **Draft System** - Save posts as drafts before publishing
- 🏷️ **Tag System** - Organize posts with colorful, filterable tags
- 📱 **Responsive Design** - Bootstrap-based UI that works on all devices
- 📧 **RSS Feed** - Subscribe to blog updates via RSS at `/feed/rss`

### Planned Features ([See Roadmap](docs/FEATURE_SUGGESTIONS.md))
- 🔍 **Search Functionality** - Full-text search across posts
- 📊 **Analytics Dashboard** - Track views and engagement
- 👤 **User Profiles** - Multi-author support with user profiles
- ❤️ **Like/Reaction System** - Reader engagement features
- 🖼️ **Image Upload** - Built-in image management

## 🚀 Quick Start

### Prerequisites
- **Rust** 1.70 or higher ([Install Rust](https://rustup.rs/))
- **PostgreSQL** 13+ ([Install PostgreSQL](https://www.postgresql.org/download/))
- **Just** command runner ([Install Just](https://github.com/casey/just)) - Optional but recommended

### 1. Clone and Setup
```bash
git clone https://github.com/xiro-codes/rocket_blog.git
cd rocket_blog
```

### 2. Database Setup
```bash
# Start PostgreSQL with Docker (recommended)
docker-compose up postgres -d

# OR set up PostgreSQL manually and create database:
# createdb tdavis_dev
# psql tdavis_dev -c "CREATE USER master WITH PASSWORD 'password';"
# psql tdavis_dev -c "GRANT ALL PRIVILEGES ON DATABASE tdavis_dev TO master;"
```

### 3. Configure Environment
```bash
# Copy and edit configuration (optional - defaults work with Docker)
cp Rocket.toml.example Rocket.toml
# Edit database URL if needed
```

### 4. Run Migrations
```bash
# Using just (recommended)
just migrate

# OR using cargo directly
cargo run -p migrations
```

### 5. Start the Application
```bash
# Development mode
cargo run

# The blog will be available at http://localhost:8000
```

### 6. Initial Setup
1. Visit `http://localhost:8000/blog`
2. Create your first admin account through the seeding process
3. Start creating blog posts!

## 📖 Usage

### Writing Blog Posts
1. **Navigate** to `/blog` to see all posts
2. **Create** new posts with the "New Post" button (requires admin login)
3. **Write** in Markdown - full markdown syntax is supported
4. **Add Tags** to organize your content (comma-separated)
5. **Save as Draft** or **Publish** immediately
6. **Edit** posts anytime with the edit button

### User Authentication
- **Login**: Use the login form at `/auth`
- **Admin Features**: Create, edit, and delete posts
- **Logout**: Use the logout link when signed in

### Comments
- **Readers** can comment on published posts
- **Moderation**: Admin users can manage comments
- **Nested Discussions**: Support for reply threads

## 🛠️ Development

### Project Structure
```
rocket_blog/
├── src/                    # Main application code
│   ├── controllers/        # Route handlers
│   ├── services/          # Business logic
│   ├── middleware/        # Request processing
│   └── main.rs           # Application entry point
├── models/               # Database models (SeaORM)
├── migrations/          # Database migrations
├── templates/          # Tera HTML templates
├── static/            # CSS, JS, images
├── docs/             # Documentation
└── scripts/         # Development scripts
```

### Available Commands
```bash
# Development workflow
just migrate           # Run database migrations
just gen-models       # Generate ORM models from database
cargo run            # Start development server
cargo test           # Run tests
cargo check         # Check code without building

# Production build
cargo build --release
```

### Development Setup
1. **Install Dependencies**: Rust, PostgreSQL, Just
2. **Database**: Run `docker-compose up postgres -d`
3. **Migrations**: Run `just migrate`
4. **Development**: Run `cargo run` for auto-reload
5. **Testing**: Run `cargo test`

### Adding Features
- **New Models**: Add migrations in `migrations/`
- **New Routes**: Add controllers in `src/controllers/`
- **New Templates**: Add `.html.tera` files in `templates/`
- **New Services**: Add business logic in `src/services/`

See [Feature Integration Guide](docs/TAG_INTEGRATION_EXAMPLE.md) for detailed examples.

## 🏗️ Architecture

### Technology Stack
- **Backend**: Rust with Rocket web framework
- **Database**: PostgreSQL with SeaORM for type-safe queries
- **Frontend**: Server-side rendered HTML with Tera templates
- **Styling**: Bootstrap 5 for responsive design
- **Authentication**: Token-based with secure cookies

### Key Components
- **Controllers**: Handle HTTP requests and responses
- **Services**: Contain business logic and database operations
- **Models**: Type-safe database entities with SeaORM
- **Middleware**: Handle authentication, seeding, and request processing
- **Templates**: Server-side rendering with Tera template engine

### Security Features
- 🔐 Password hashing with secure algorithms
- 🍪 Secure cookie-based authentication
- 🛡️ CSRF protection ready
- 🔒 SQL injection protection via SeaORM
- 🚫 Input validation and sanitization

## 📦 Deployment

### Docker Deployment (Recommended)
```bash
# Build and run with Docker Compose
docker-compose up --build

# Or build manually
docker build -t rocket-blog .
docker run -p 8000:8000 rocket-blog
```

### Manual Deployment
```bash
# Build release binary
cargo build --release

# Set up production database and environment
export ROCKET_DATABASES__SEA_ORM__URL="postgres://user:pass@host/db"
export ROCKET_DATA_PATH="/var/lib/rocket-blog"

# Run migrations
cargo run -p migrations

# Start application
./target/release/app
```

### Environment Variables
- `ROCKET_DATABASES__SEA_ORM__URL`: Database connection string
- `ROCKET_DATA_PATH`: Directory for uploaded files
- `ROCKET_SECRET_KEY`: Secret key for sessions (generate with `openssl rand -base64 32`)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](docs/CONTRIBUTING.md) for details.

### Quick Contribution Steps
1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes with tests
4. **Test** your changes (`cargo test`)
5. **Commit** your changes (`git commit -am 'Add amazing feature'`)
6. **Push** to the branch (`git push origin feature/amazing-feature`)
7. **Open** a Pull Request

### Development Guidelines
- Write tests for new features
- Follow Rust coding standards
- Update documentation for user-facing changes
- Use conventional commit messages

## 📚 Documentation

- [**Feature Roadmap**](docs/FEATURE_SUGGESTIONS.md) - Planned features and implementation timeline
- [**Implementation Guide**](docs/IMPLEMENTATION_SUMMARY.md) - Technical architecture details
- [**Tag Integration Example**](docs/TAG_INTEGRATION_EXAMPLE.md) - Step-by-step feature integration
- [**Database Migrations**](migrations/README.md) - Managing database changes
- [**Development Scripts**](scripts/README.md) - Automation and tooling

## 🐛 Issues and Support

- **Bug Reports**: [Create an issue](https://github.com/xiro-codes/rocket_blog/issues)
- **Feature Requests**: [Request a feature](https://github.com/xiro-codes/rocket_blog/issues)
- **Questions**: [Start a discussion](https://github.com/xiro-codes/rocket_blog/discussions)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Rocket Framework** - For the excellent Rust web framework
- **SeaORM** - For type-safe database operations
- **Bootstrap** - For responsive UI components
- **Tera** - For powerful templating capabilities

---

**Happy Blogging!** 🎉

For more information, visit the [full documentation](docs/) or check out the [live demo](https://your-demo-url.com).