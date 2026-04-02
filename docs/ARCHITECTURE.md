# Architecture Overview

This document provides a comprehensive overview of the Rocket Blog application architecture, design decisions, and system components.

## 🏗️ System Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Browser   │    │ NixOS Nginx     │    │  Rocket Blog    │
│                 │◄──►│  Reverse Proxy  │◄──►│   Application   │
│   (Frontend)    │    │                 │    │   (Backend)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                                                        ▼
                                               ┌─────────────────┐
                                               │   PostgreSQL    │
                                               │    Database     │
                                               └─────────────────┘
```

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Backend Framework** | Rust + Rocket | High-performance web server |
| **Database** | PostgreSQL + SeaORM | Reliable data persistence with type safety |
| **Frontend** | Server-side rendered HTML + Bootstrap | Fast, SEO-friendly user interface |
| **Templating** | MiniJinja | Dynamic HTML generation |
| **Authentication** | Cookie-based tokens | Secure user sessions |
| **Build System** | Cargo + Just | Rust build tooling |

## 📁 Project Structure

```
rocket_blog/
├── src/                          # Main application source code
│   ├── controllers/              # HTTP request handlers
│   │   ├── auth.rs              # Authentication endpoints
│   │   ├── blog.rs              # Blog CRUD operations
│   │   ├── comment.rs           # Comment management
│   │   ├── index.rs             # Homepage routes
│   │   └── base.rs              # Shared controller utilities
│   ├── services/                 # Business logic layer
│   │   ├── auth.rs              # Authentication service
│   │   ├── blog.rs              # Blog operations
│   │   ├── comment.rs           # Comment operations
│   │   └── tag.rs               # Tag management
│   ├── middleware/               # Request processing
│   │   ├── seeding.rs           # Development data seeding
│   │   └── mod.rs               # Middleware exports
│   ├── dto/                     # Data transfer objects
│   ├── types/                   # Custom type definitions
│   ├── config.rs                # Application configuration
│   ├── pool.rs                  # Database connection pool
│   └── main.rs                  # Application entry point
├── models/                       # Database entity definitions
│   └── src/
│       ├── account.rs           # User account model
│       ├── post.rs              # Blog post model
│       ├── comment.rs           # Comment model
│       ├── tag.rs               # Tag model
│       ├── post_tag.rs          # Post-Tag relationship
│       └── dto.rs               # Form data structures
├── migrations/                   # Database schema evolution
├── templates/                    # HTML templates
│   ├── base.html.minijinja          # Base template layout
│   ├── blog/                   # Blog-related templates
│   └── error/                  # Error page templates
├── static/                      # Static assets (CSS, JS, images)
├── scripts/                     # Development automation scripts
└── docs/                        # Documentation
```

## 🔄 Request Flow

### Typical HTTP Request Flow

```
1. Browser Request
   │
   ▼
2. Nginx (Optional)
   ├── Static files → Direct serve
   └── Dynamic requests
       │
       ▼
3. Rocket Application
   ├── Authentication Middleware
   ├── Route Matching
   └── Controller Handler
       │
       ▼
4. Service Layer
   ├── Business Logic
   └── Database Operations
       │
       ▼
5. SeaORM + PostgreSQL
   │
   ▼
6. Response Generation
   ├── Template Rendering (MiniJinja)
   └── JSON/Redirect Response
       │
       ▼
7. Browser Response
```

### Example: Creating a Blog Post

```rust
// 1. HTTP Request: POST /blog/
// 2. Controller (blog.rs)
#[post("/", data = "<form>")]
async fn create_post(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    form: Form<FormDTO>
) -> Result<Redirect, Status> {
    // 3. Service Layer (services/blog.rs)
    let post = service.create_post(db, form.into_inner()).await?;
    
    // 4. Database Operation (via SeaORM)
    // INSERT INTO posts (title, content, ...) VALUES (...)
    
    // 5. Response
    Ok(Redirect::to(format!("/blog/{}", post.seq_id)))
}
```

## 🏛️ Architectural Patterns

### Model-View-Controller (MVC)

- **Models** (`models/`): Database entities and data structures
- **Views** (`templates/`): HTML templates for user interface
- **Controllers** (`src/controllers/`): Handle HTTP requests and responses

### Service Layer Pattern

Business logic is separated into service classes:

```rust
// Service interface
impl BlogService {
    async fn create_post(&self, db: &DatabaseConnection, form: FormDTO) -> Result<post::Model> {
        // Complex business logic here
        // Validation, data transformation, etc.
    }
    
    async fn list_posts(&self, db: &DatabaseConnection, page: u64) -> Result<Vec<post::Model>> {
        // Pagination logic
        // Filtering and sorting
    }
}
```

**Benefits:**
- ✅ Separation of concerns
- ✅ Testable business logic
- ✅ Reusable across controllers
- ✅ Transaction management

### Repository Pattern (via SeaORM)

Database access is abstracted through SeaORM's Active Record pattern:

```rust
// Clean database operations
let posts = Post::find()
    .filter(post::Column::Published.eq(true))
    .order_by_desc(post::Column::CreatedAt)
    .paginate(db, page_size)
    .fetch_page(page)
    .await?;
```

### Fairing System (Middleware)

Rocket's fairing system provides cross-cutting concerns:

```rust
// Application setup
rocket::build()
    .attach(Db::init())                    // Database connection
    .attach(Template::fairing())           // Template engine
    .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
    .attach(middleware::Seeding::new())    // Development data
    .manage(TagService::new())             // Dependency injection
```

## 🔌 Component Architecture

### Database Layer

**SeaORM Active Record Pattern:**
```rust
// Entity definition
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
    #[sea_orm(column_type = "Timestamp")]
    pub created_at: NaiveDateTime,
}
```

**Benefits:**
- 🔒 Compile-time type safety
- 🚀 Excellent performance
- 🔄 Automatic migrations
- 🛡️ SQL injection protection

### Authentication System

**Cookie-based Authentication:**
```rust
// Login flow
#[post("/", data = "<data>")]
async fn login(jar: &CookieJar<'_>, data: Form<AccountFormDTO>) -> Flash<Redirect> {
    if let Ok(token) = service.login(db, data.into_inner()).await {
        jar.add_private(Cookie::new("token", token.to_string()));
        // Secure, HTTP-only cookie automatically set
    }
}
```

**Security Features:**
- 🔐 Bcrypt password hashing
- 🍪 HTTP-only, secure cookies
- ⏰ Token expiration
- 🔒 Private cookie encryption

### Templating System

**MiniJinja Template Engine:**
```html
<!-- templates/blog/list.html.minijinja -->
{% extends "base" %}
{% block content %}
  {% for post in posts %}
    <article>
      <h2>{{ post.title }}</h2>
      <div>{{ post.content | markdown | safe }}</div>
      {% if post.tags %}
        {% for tag in post.tags %}
          <span class="badge" style="background-color: {{ tag.color }}">
            {{ tag.name }}
          </span>
        {% endfor %}
      {% endif %}
    </article>
  {% endfor %}
{% endblock %}
```

**Features:**
- 🎨 Template inheritance
- 🔧 Custom filters (markdown)
- 🔒 Automatic HTML escaping
- 📱 Responsive Bootstrap components

## 📊 Data Flow Architecture

### Database Schema

```sql
-- Core entities
accounts (id, username, password_hash, admin, created_at)
posts (id, seq_id, title, content, published, account_id, created_at)
comments (id, author, content, post_id, created_at)
tags (id, name, color, created_at)
post_tags (post_id, tag_id)  -- Many-to-many relationship
```

### Entity Relationships

```
Account (1) ──────── (N) Post
                        │
                        │ (1)
                        │
                        │ (N)
                    Comment
                        
Post (N) ──────── (N) Tag
     └─── post_tags ───┘
```

### Data Access Patterns

```rust
// Eager loading with relationships
let posts_with_tags = Post::find()
    .find_with_related(Tag)
    .all(db)
    .await?;

// Lazy loading
let post = Post::find_by_id(1).one(db).await?;
let tags = post.find_related(Tag).all(db).await?;

// Aggregation queries
let post_count = Post::find()
    .filter(post::Column::Published.eq(true))
    .count(db)
    .await?;
```

## 🚀 Performance Characteristics

### Request Processing Performance

| Operation | Typical Response Time | Database Queries |
|-----------|----------------------|------------------|
| List Posts | 50-100ms | 1-2 queries |
| View Post | 20-50ms | 2-3 queries |
| Create Post | 100-200ms | 3-5 queries |
| Add Comment | 50-100ms | 2 queries |

### Memory Usage

- **Base Application**: ~10-20MB
- **Per Request**: ~1-5MB (temporary)
- **Database Connections**: ~2-5MB per connection
- **Template Cache**: ~5-10MB

### Concurrency Model

**Async/Await Architecture:**
```rust
// Non-blocking I/O operations
#[rocket::async_trait]
impl Fairing for CustomFairing {
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        // Async initialization
        let db_connection = establish_connection().await?;
        Ok(rocket.manage(db_connection))
    }
}
```

**Benefits:**
- 🚀 High concurrency with low resource usage
- ⚡ Non-blocking database operations
- 📈 Excellent scalability characteristics

## 🔒 Security Architecture

### Authentication Flow

```
1. User Login Request
   ├── Password verification (bcrypt)
   ├── Token generation (UUID)
   ├── Secure cookie creation
   └── HTTP-only, secure flag set

2. Protected Request
   ├── Cookie extraction
   ├── Token validation
   ├── User permission check
   └── Request processing or rejection
```

### Security Layers

| Layer | Protection | Implementation |
|-------|------------|----------------|
| **Transport** | HTTPS/TLS | Nginx + Let's Encrypt |
| **Application** | Input validation | Form validation, SeaORM |
| **Authentication** | Secure sessions | Encrypted cookies, bcrypt |
| **Authorization** | Role-based access | Admin flags, route guards |
| **Database** | SQL injection | SeaORM parameterized queries |
| **XSS** | Output escaping | MiniJinja automatic escaping |

## 🔧 Configuration Management

### Environment-based Configuration

```toml
# Rocket.toml
[default]
address = "127.0.0.1"
port = 8000
data_path = "/home/user/.local/share/blog"

[default.databases.sea_orm]
url = "postgres://user:pass@localhost/blog_dev"

[release]
secret_key = "production_secret_key_here"
data_path = "/app/data"

[release.databases.sea_orm]
url = "postgres://user:pass@prod_host/blog_prod"
```

### Feature Flags

```rust
// Conditional compilation for features
#[cfg(feature = "development")]
.attach(middleware::Seeding::new(Some(0), 50))

#[cfg(debug_assertions)]
setup_development_logger().unwrap();
```

## 🧪 Testing Architecture

### Test Organization

```
src/tests/
├── integration_tests.rs     # Full application tests
├── controllers_tests.rs     # Controller unit tests
├── services_tests.rs        # Business logic tests
├── main_tests.rs           # Application setup tests
└── utils.rs                # Test utilities
```

### Testing Patterns

```rust
// Service layer testing
#[tokio::test]
async fn test_create_blog_post() {
    let db = setup_test_db().await;
    let service = BlogService::new();
    
    let form = FormDTO {
        title: "Test Post".to_string(),
        content: "Test content".to_string(),
    };
    
    let result = service.create_post(&db, form).await;
    assert!(result.is_ok());
}

// Integration testing
#[rocket::async_test]
async fn test_blog_endpoint() {
    let client = Client::tracked(rocket()).await.unwrap();
    let response = client.get("/blog").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
}
```

## 📈 Scalability Considerations

### Horizontal Scaling

```
┌─────────────────┐    ┌─────────────────┐
│ NixOS Nginx     │    │   App Instance  │
│                 │◄──►│       #1        │
│                 │    └─────────────────┘
│                 │    ┌─────────────────┐
│                 │◄──►│   App Instance  │
│                 │    │       #2        │
└─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   PostgreSQL    │
                       │    Database     │
                       └─────────────────┘
```

### Database Scaling

**Read Replicas:**
```rust
// Future: Read/write splitting
async fn get_posts(read_db: &DatabaseConnection) -> Result<Vec<Post>> {
    // Use read replica for queries
}

async fn create_post(write_db: &DatabaseConnection) -> Result<Post> {
    // Use primary database for writes
}
```

### Caching Strategy

**Planned Caching Layers:**
- **Application Cache**: In-memory post cache
- **Database Cache**: PostgreSQL query cache
- **CDN**: Static asset caching
- **Browser Cache**: Client-side caching

## 🔄 Future Architecture Evolution

### Planned Improvements

1. **Microservices Migration**
   - Extract authentication service
   - Separate media handling service
   - Independent scaling per service

2. **API Layer**
   - REST API endpoints
   - GraphQL interface
   - WebSocket real-time features

3. **Advanced Features**
   - Full-text search (Elasticsearch)
   - Real-time notifications
   - Advanced analytics

4. **Performance Optimizations**
   - Redis caching layer
   - CDN integration
   - Database sharding

### Migration Strategy

```
Phase 1: Monolith (Current)
   └── Single Rocket application

Phase 2: Modular Monolith
   ├── Clear service boundaries
   ├── API-first design
   └── Preparation for extraction

Phase 3: Selective Microservices
   ├── Auth service
   ├── Media service
   └── Core blog service

Phase 4: Full Microservices
   ├── Independent scaling
   ├── Technology diversity
   └── Advanced orchestration
```

---

This architecture provides a solid foundation for a modern, scalable blog application while maintaining simplicity and performance. The clean separation of concerns and type-safe design makes it easy to maintain and extend. 🏗️