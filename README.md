# Rocket Blog

A modern, fast, and feature-rich blog application built with **Rust** and the **Rocket** web framework. This platform provides both a blog system and a Progressive Web App (PWA) work time tracker as separate binaries.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Rust Version](https://img.shields.io/badge/rust-1.70+-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## 🏗️ Architecture

This application consists of **two separate binaries**:

- **Blog Binary** (`blog`) - Complete blog platform with content management
- **Work Time Tracker Binary** (`worktime`) - PWA for time tracking with role-based wages

See [README_BINARIES.md](README_BINARIES.md) for detailed information about running each binary.

## ✨ Features

### Current Features
- 📝 **Blog Management** - Create, edit, delete, and publish blog posts with markdown support
- ✨ **WYSIWYG Markdown Editor** - Visual markdown editing with EasyMDE integration
  - Real-time preview and live editing capabilities
  - Syntax highlighting and formatting toolbar
  - Side-by-side and fullscreen editing modes
  - Custom terminal theme styling matching blog aesthetic
- 📄 **Post Excerpts/Summary System** - Enhanced content discovery and navigation
  - Optional custom excerpts or automatic generation from post content
  - Modern card-based list view with post previews
  - Auto-generated excerpts with intelligent content extraction (200 char limit)
  - "Read More" buttons for improved user experience
- 🔐 **Authentication System** - Secure login/logout with admin privileges
- 💬 **Comment System** - Enable readers to comment on blog posts with moderation
- 🎬 **Media Support** - Video streaming with range requests for optimal performance
- 📄 **Pagination** - Efficient navigation through blog posts
- 📑 **Draft System** - Save posts as drafts before publishing
- 🏷️ **Tag System** - Organize posts with colorful, filterable tags (backend complete)
- 📱 **Responsive Design** - Bootstrap-based UI that works on all devices
- 📧 **RSS Feed Generation** - Complete RSS feed implementation
  - RSS feed available at `/feed/rss` endpoint
  - Includes post excerpts for better feed content
  - XML-compliant RSS 2.0 format with proper metadata

### Planned Features ([See Roadmap](docs/FEATURE_SUGGESTIONS.md))
- 🔍 **Search Functionality** - Full-text search across posts with advanced filtering
- 🏷️ **Tag UI Integration** - Complete tag filtering and management interface
- 📊 **Analytics Dashboard** - Track views, engagement, and content performance
- 👤 **User Profiles** - Multi-author support with user profiles and bio
- ❤️ **Like/Reaction System** - Reader engagement features with emoji reactions
- 🖼️ **Image Upload** - Built-in image management with optimization
- 📡 **REST API** - JSON endpoints for all blog operations
- 🔔 **Real-time Notifications** - Live updates for comments and interactions
- 🌐 **Social Media Integration** - Share buttons and embedded content
- 📧 **Email Newsletter** - Subscriber management and automated campaigns
- 🌍 **Multi-language Support** - Internationalization (i18n) capabilities
- 📈 **SEO Optimization Tools** - Advanced SEO features and meta management

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
docker-compose -f scripts/docker/docker-compose.yml up postgres -d

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

### Browsing Blog Posts
1. **Main Blog Page** - Visit `/blog` to see all posts with enhanced card-based layout
2. **Post Discovery** - Browse posts with auto-generated or custom excerpts for quick content overview
3. **Tag Navigation** - Click on colorful tags to filter posts by topic (backend ready)
4. **RSS Subscription** - Subscribe to `/feed/rss` for updates with excerpt content and metadata
5. **Responsive Design** - Seamless experience across desktop, tablet, and mobile devices
6. **Modern UI** - Enhanced card layouts with improved readability and navigation

### Writing Blog Posts
1. **Navigate** to `/blog` to see all posts
2. **Create** new posts with the "New Post" button (requires admin login)
3. **Write** in Markdown with the visual WYSIWYG editor featuring:
   - Real-time preview and live editing
   - Syntax highlighting and formatting toolbar
   - Side-by-side and fullscreen editing modes
4. **Add Excerpts** - Write custom summaries or let the system auto-generate them
5. **Add Tags** to organize your content (comma-separated)
6. **Save as Draft** or **Publish** immediately
7. **Edit** posts anytime with the edit button

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
2. **Database**: Run `docker-compose -f scripts/docker/docker-compose.yml up postgres -d`
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
- **Numeric Types**: f64 for monetary calculations (simplified from rust_decimal)

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

For a complete Docker setup with build instructions, see [Docker Guide](docs/DOCKER.md).

**Development Options:**
```bash
# Standard development (debug builds, faster compilation)
./scripts/docker-deploy.sh dev

# Live development (template/static file auto-reload)
./scripts/docker-deploy.sh dev-live

# Or use docker-compose directly:
docker-compose -f scripts/docker/docker-compose.dev.yml up --build      # Standard
docker-compose -f scripts/docker/docker-compose.dev.live.yml up --build # Live template reload
```

**Production:**
```bash
# Production with nginx reverse proxy and SSL
./scripts/setup-ssl.sh  # First time only
./scripts/docker-deploy.sh prod
```

**Development Features:**
- Production builds compiled in clean containerized environment (cross-platform)
- Live template and static file reloading (dev-live mode)
- Direct database access for development tools
- Verbose logging for debugging
- No SSL complexity for faster iteration

**Production Features:**
- **Dual Binary Support**: Blog and work time tracker as separate services
- Nginx reverse proxy with SSL termination
- Automatic SSL certificate generation and renewal
- Production-optimized container images
- Secure defaults with proper headers
- Independent scaling of blog and worktime applications

**NixOS Users:** The Docker approach solves build issues on NixOS by building inside the container. See the [Docker Guide](docs/DOCKER.md) for troubleshooting SSL certificate issues and alternative build strategies.

**Troubleshooting Docker Builds:** If you encounter SSL certificate issues during Docker build, try:
```bash
# Use the fallback Dockerfile
docker build -f scripts/docker/Dockerfile.dev -t rocket-blog .

# Or build with host network
docker build --network=host -t rocket-blog .
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

### Core Documentation
- [**User Guide**](docs/USER_GUIDE.md) - Complete guide for content creators and blog users
- [**Architecture Overview**](docs/ARCHITECTURE.md) - Technical architecture and system design
- [**API Documentation**](docs/API.md) - Comprehensive API reference and examples

### Feature Guides
- [**Feature Roadmap**](docs/FEATURE_SUGGESTIONS.md) - Planned features and implementation timeline
- [**Implementation Guide**](docs/IMPLEMENTATION_SUMMARY.md) - Technical architecture details
- [**Tag Integration Example**](docs/TAG_INTEGRATION_EXAMPLE.md) - Step-by-step feature integration
- [**Post Excerpt Guide**](docs/POST_EXCERPT_IMPLEMENTATION.md) - Post summary system documentation
- [**WYSIWYG Editor Guide**](docs/WYSIWYG_EDITOR_IMPLEMENTATION.md) - Visual editor implementation details

### Development Documentation
- [**Contributing Guide**](docs/CONTRIBUTING.md) - How to contribute to the project
- [**Deployment Guide**](docs/DEPLOYMENT.md) - Production deployment instructions
- [**Docker Guide**](docs/DOCKER.md) - Container deployment and development
- [**Development Setup**](docs/DEVELOPMENT.md) - Optimized development workflows
- [**Database Migrations**](migrations/README.md) - Managing database changes
- [**Development Scripts**](scripts/README.md) - Automation and tooling

### Project Documentation
- [**Changelog**](docs/CHANGELOG.md) - Project history and release notes with detailed feature implementations
- [**Feature Roadmap**](docs/FEATURE_SUGGESTIONS.md) - Updated roadmap with completed and planned features
- [**Cleanup Summary**](docs/CLEANUP_SUMMARY.md) - Code cleanup and refactoring notes

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