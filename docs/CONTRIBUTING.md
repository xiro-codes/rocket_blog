# Contributing to Rocket Blog

Thank you for your interest in contributing to Rocket Blog! This guide will help you get started with contributing to this Rust-based blog application.

## 🚀 Quick Start for Contributors

### Prerequisites
- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **PostgreSQL** 13+ ([Install PostgreSQL](https://www.postgresql.org/download/))
- **Just** command runner ([Install Just](https://github.com/casey/just)) - Recommended
- **Git** for version control

### Development Environment Setup

1. **Fork and Clone**
   ```bash
   # Fork the repository on GitHub, then:
   git clone https://github.com/your-username/rocket_blog.git
   cd rocket_blog
   ```

2. **Database Setup**
   ```bash
   # Option 1: Docker Development Environment (Recommended)
   just docker-dev            # Standard development environment
   just docker-dev-live       # Development with live template/static reloading
   
   # Option 2: Docker Database Only
   docker-compose up postgres -d
   
   # Option 3: Local PostgreSQL
   createdb tdavis_dev
   psql tdavis_dev -c "CREATE USER master WITH PASSWORD 'password';"
   psql tdavis_dev -c "GRANT ALL PRIVILEGES ON DATABASE tdavis_dev TO master;"
   ```

3. **Install Dependencies and Setup**
   ```bash
   # Run database migrations
   just migrate
   # OR: cargo run -p migrations
   
   # Generate models (if you modify database schema)
   just gen-models
   
   # Run tests to ensure everything works
   just test
   # OR: cargo test
   
   # Start development server
   just run
   # OR: cargo run
   ```

4. **Verify Setup**
   - Visit `http://localhost:8000/blog`
   - You should see the blog interface
   - Try creating a test post (login as admin)

### Docker Development with Just

The project includes comprehensive Docker support accessible through just commands:

**Development Environments:**
```bash
# Standard development environment
just docker-dev              # App: http://localhost:8000, pgAdmin: http://localhost:5050

# Live development with template/static file reloading  
just docker-dev-live          # Perfect for frontend development and template editing

# Production environment (with SSL)
just docker-prod              # Requires SSL setup
```

**Docker Management:**
```bash
# Check status
just docker-status            # Show running services

# View logs
just docker-logs              # All services
just docker-logs app          # Specific service (app, postgres, pgadmin, nginx)

# Stop and cleanup
just docker-stop              # Stop all services
just docker-clean             # Stop and remove containers/volumes (WARNING: deletes data)

# SSL management (for production)
just docker-setup-ssl         # Initial SSL certificate setup
just docker-renew-ssl         # Force SSL certificate renewal

# Help
just docker-help              # Show Docker command help
```

**Benefits of Docker Development:**
- **Consistent environment** across different machines and operating systems
- **No local dependencies** - PostgreSQL, pgAdmin, and SSL certificates handled automatically
- **Live reloading** for templates and static files (with `docker-dev-live`)
- **Production parity** - same environment as production deployment
- **Quick teardown** and fresh starts for testing

## 📋 Development Workflow

### Typical Development Process

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # OR for bug fixes:
   git checkout -b fix/issue-description
   ```

2. **Make Changes**
   - Write your code following our guidelines (see below)
   - Add tests for new functionality
   - Update documentation if needed

3. **Test Your Changes**
   ```bash
   # Run all tests
   just test
   # OR: cargo test
   
   # Check code formatting
   just fmt-check
   # OR: cargo fmt --check
   
   # Run linter (if available)
   just clippy
   # OR: cargo clippy
   
   # Test the application manually
   just run
   # OR: cargo run
   
   # OR test in Docker environment
   just docker-dev              # Test with production-like build
   just docker-dev-live          # Test with live template reloading
   ```

4. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add amazing new feature"
   # Use conventional commits (see below)
   ```

5. **Submit Pull Request**
   ```bash
   git push origin feature/your-feature-name
   # Then create a PR on GitHub
   ```

## 🎯 Areas for Contribution

### 🟢 Good First Issues (Easy)
- **Documentation improvements** - Fix typos, add examples, improve clarity
- **Template enhancements** - Improve UI/UX with CSS/HTML changes
- **Test coverage** - Add unit tests for existing functions
- **Code cleanup** - Remove unused imports, improve error messages

### 🟡 Intermediate Features
- **New blog features** - Implement features from our [roadmap](FEATURE_SUGGESTIONS.md)
- **API endpoints** - Add REST API support for blog operations
- **UI improvements** - Add responsive design, better navigation
- **Performance optimizations** - Database query optimization, caching

### 🔴 Advanced Contributions
- **Architecture improvements** - Refactor for better maintainability
- **Security enhancements** - CSRF protection, rate limiting
- **Advanced features** - Multi-author support, analytics dashboard
- **Infrastructure** - CI/CD, deployment improvements

## 📝 Coding Guidelines

### Rust Style Guidelines

1. **Use `rustfmt`**
   ```bash
   just fmt
   # OR: cargo fmt
   ```

2. **Follow Clippy recommendations**
   ```bash
   just clippy
   # OR: cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Write documentation**
   ```rust
   /// Creates a new blog post with the given title and content.
   /// 
   /// # Arguments
   /// * `title` - The title of the blog post
   /// * `content` - The markdown content of the post
   /// 
   /// # Returns
   /// Result containing the created post or an error
   pub async fn create_post(title: &str, content: &str) -> Result<Post, Error> {
       // implementation
   }
   ```

### Project-Specific Guidelines

1. **Controller Structure**
   - Keep controllers thin - delegate business logic to services
   - Use consistent error handling patterns
   - Follow existing route naming conventions

2. **Service Layer**
   - Put business logic in services
   - Keep database operations in services
   - Use async/await consistently

3. **Database Migrations**
   - Always create reversible migrations
   - Test migrations both up and down
   - Document schema changes

4. **Templates**
   - Follow Bootstrap conventions for styling
   - Keep templates semantic and accessible
   - Use consistent naming for template variables

### Code Examples

**Controller Pattern:**
```rust
#[get("/posts/<id>")]
pub async fn get_post(
    conn: Connection<'_, Db>,
    id: i32,
    service: &State<BlogService>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    match service.get_post_by_id(db, id).await {
        Ok(post) => Ok(Template::render("blog/detail", context! { post })),
        Err(_) => Err(Status::NotFound),
    }
}
```

**Service Pattern:**
```rust
impl BlogService {
    pub async fn get_post_by_id(&self, db: &DatabaseConnection, id: i32) -> Result<post::Model, DbErr> {
        Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Post not found".to_string()))
    }
}
```

## 🧪 Testing Guidelines

### Writing Tests

1. **Unit Tests** - Test individual functions
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_create_slug() {
           assert_eq!(create_slug("Hello World"), "hello-world");
       }
   }
   ```

2. **Integration Tests** - Test controller endpoints
   ```rust
   #[rocket::async_test]
   async fn test_blog_list_endpoint() {
       let client = Client::tracked(rocket()).await.unwrap();
       let response = client.get("/blog").dispatch().await;
       assert_eq!(response.status(), Status::Ok);
   }
   ```

3. **Service Tests** - Test business logic
   ```rust
   #[tokio::test]
   async fn test_blog_service_create_post() {
       let service = BlogService::new();
       let result = service.create_post(&db, "Test", "Content").await;
       assert!(result.is_ok());
   }
   ```

### Running Tests
```bash
# Run all tests
just test
# OR: cargo test

# Run specific test
just test-name test_create_slug
# OR: cargo test test_create_slug

# Run tests with output
just test-verbose
# OR: cargo test -- --nocapture

# Run integration tests only
just test --test integration_tests
# OR: cargo test --test integration_tests
```

## 📚 Documentation Standards

### Code Documentation
- Document all public functions and structs
- Include examples in documentation when helpful
- Explain complex algorithms or business logic

### User Documentation
- Update README.md for user-facing changes
- Add examples for new features
- Keep documentation current with code changes

### Technical Documentation
- Document architectural decisions
- Update API documentation for endpoint changes
- Include migration guides for breaking changes

## 🔄 Pull Request Process

### Before Submitting
- [ ] Code follows style guidelines
- [ ] Tests pass (`just test`)
- [ ] Documentation is updated
- [ ] Commit messages follow conventions
- [ ] No merge conflicts with main branch

### PR Template
When creating a PR, please include:

**Description**
- What does this PR do?
- Why is this change needed?

**Changes Made**
- List key changes
- Mention any breaking changes

**Testing**
- How was this tested?
- Are there new tests?

**Documentation**
- Was documentation updated?
- Are there examples?

### Code Review Process
1. **Automated Checks** - CI runs tests and linting
2. **Peer Review** - Other contributors review code
3. **Maintainer Review** - Core maintainers approve changes
4. **Merge** - Changes are merged to main branch

## 📐 Commit Message Conventions

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `chore:` - Maintenance tasks

**Examples:**
```bash
git commit -m "feat(blog): add tag filtering to blog posts"
git commit -m "fix(auth): resolve login redirect issue"
git commit -m "docs: update installation instructions"
```

## 🏗️ Architecture Overview

### Key Components
- **Controllers** (`src/controllers/`) - Handle HTTP requests
- **Services** (`src/services/`) - Business logic and database operations
- **Models** (`models/src/`) - Database entity definitions
- **Migrations** (`migrations/`) - Database schema changes
- **Templates** (`templates/`) - HTML templates with Tera
- **Static Assets** (`static/`) - CSS, JavaScript, images

### Adding New Features

1. **Database Changes**
   ```bash
   # Create migration
   just new-migration add_feature_table
   
   # Edit migration file
   # Run migration
   just migrate
   
   # Regenerate models
   just gen-models
   ```

2. **Service Layer**
   ```rust
   // Add to src/services/
   pub struct FeatureService;
   
   impl FeatureService {
       pub async fn create_feature(&self, db: &DatabaseConnection) -> Result<(), DbErr> {
           // Implementation
       }
   }
   ```

3. **Controller Layer**
   ```rust
   // Add to src/controllers/
   #[get("/features")]
   pub async fn list_features(service: &State<FeatureService>) -> Template {
       // Implementation
   }
   ```

4. **Templates**
   ```html
   <!-- Add to templates/ -->
   {% extends "base" %}
   {% block content %}
   <h1>Features</h1>
   {% endblock %}
   ```

## 🆘 Getting Help

- **Discord/Chat** - [Join our community](https://discord.gg/your-invite)
- **Issues** - [Create an issue](https://github.com/xiro-codes/rocket_blog/issues)
- **Discussions** - [GitHub Discussions](https://github.com/xiro-codes/rocket_blog/discussions)
- **Email** - contributor-help@your-domain.com

## 🎉 Recognition

Contributors are recognized in:
- README.md contributors section
- Release notes for major contributions
- Special thanks for significant architectural improvements

Thank you for contributing to Rocket Blog! Your help makes this project better for everyone. 🚀