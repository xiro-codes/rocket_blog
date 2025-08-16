# Rocket Blog API Documentation

This document provides comprehensive information about the Rocket Blog API endpoints, authentication, and usage examples.

## 🔗 Base URL

```
Local Development: http://localhost:8000
Production: https://your-domain.com
```

## 🔐 Authentication

The Rocket Blog uses cookie-based authentication with secure tokens.

### Login
```http
POST /auth/
Content-Type: application/x-www-form-urlencoded

username=admin&password=your_password
```

**Response:**
- **Success**: Redirects to `/blog` with authentication cookie set
- **Failure**: Redirects to `/blog` with error message

### Logout
```http
GET /auth/logout
```

**Response:** Redirects to `/blog` with authentication cookie cleared

## 📝 Blog Endpoints

### List All Blog Posts

```http
GET /blog/?page=1&page_size=10
```

**Parameters:**
- `page` (optional): Page number (default: 1)
- `page_size` (optional): Posts per page (default: 10)

**Response:** HTML page with paginated blog posts

**Example:**
```bash
curl "http://localhost:8000/blog/?page=1&page_size=5"
```

### Get Single Blog Post

```http
GET /blog/{seq_id}
```

**Parameters:**
- `seq_id`: Sequential ID of the blog post

**Response:** HTML page with blog post details and comments

**Example:**
```bash
curl "http://localhost:8000/blog/1"
```

### Create New Blog Post

```http
POST /blog/
Content-Type: application/x-www-form-urlencoded
Authentication: Required (Admin)

title=My+Blog+Post&content=This+is+**markdown**+content&tags=rust,web,blog
```

**Parameters:**
- `title`: Post title (required)
- `content`: Markdown content (required)
- `tags`: Comma-separated tags (optional)

**Response:** 
- **Success**: Redirects to `/blog/{seq_id}`
- **Failure**: Returns error page

**Example:**
```bash
curl -X POST "http://localhost:8000/blog/" \
  -H "Cookie: token=your_auth_token" \
  -d "title=Hello World&content=This is my first post&tags=welcome,first"
```

### Update Blog Post

```http
POST /blog/{seq_id}
Content-Type: application/x-www-form-urlencoded
Authentication: Required (Admin)

title=Updated+Title&content=Updated+**markdown**+content&tags=updated,tags
```

**Parameters:**
- `seq_id`: Sequential ID of the blog post
- `title`: Updated post title (required)
- `content`: Updated markdown content (required)
- `tags`: Updated comma-separated tags (optional)

**Response:**
- **Success**: Redirects to `/blog/{seq_id}`
- **Failure**: Returns error page

### Delete Blog Post

```http
DELETE /blog/{seq_id}
Authentication: Required (Admin)
```

**Response:**
- **Success**: Redirects to `/blog/`
- **Failure**: Returns error page

### Get Edit Form

```http
GET /blog/{seq_id}/edit
Authentication: Required (Admin)
```

**Response:** HTML page with pre-filled edit form

### Get Create Form

```http
GET /blog/create
Authentication: Required (Admin)
```

**Response:** HTML page with blog creation form

### Stream Blog Media

```http
GET /blog/{seq_id}/stream
Range: bytes=0-1024 (optional)
```

**Parameters:**
- `seq_id`: Sequential ID of the blog post
- `Range`: HTTP Range header for partial content requests

**Response:** 
- **Success**: Video/media file with proper range support
- **Not Found**: 404 if media file doesn't exist

**Example:**
```bash
# Stream entire file
curl "http://localhost:8000/blog/1/stream"

# Request specific byte range
curl -H "Range: bytes=0-1023" "http://localhost:8000/blog/1/stream"
```

## 💬 Comment Endpoints

### Create Comment

```http
POST /comment/
Content-Type: application/x-www-form-urlencoded

author=John+Doe&content=Great+post!&post_id=1
```

**Parameters:**
- `author`: Comment author name (required)
- `content`: Comment content (required)
- `post_id`: Sequential ID of the blog post (required)

**Response:**
- **Success**: Redirects to `/blog/{post_id}#comments`
- **Failure**: Returns error page

**Example:**
```bash
curl -X POST "http://localhost:8000/comment/" \
  -d "author=Jane Smith&content=Thanks for sharing!&post_id=1"
```

## 🏷️ Tag Endpoints

The tag system is implemented on the backend but doesn't have dedicated public endpoints yet. Tags are managed through blog post creation and editing.

### Current Tag Functionality:
- **Create tags**: Automatically created when used in blog posts
- **Filter by tags**: Available in the UI (implementation ready)
- **Tag management**: Admin interface (planned feature)

### Planned Tag Endpoints:
```http
GET /tags/                    # List all tags
GET /tags/{name}             # Posts with specific tag  
GET /api/tags/{name}/posts   # JSON API for tag filtering
```

## 📧 RSS Feed Endpoints

### Get RSS Feed
```http
GET /feed/rss
```

Returns an RSS 2.0 XML feed of the 20 most recent published blog posts.

**Response:**
- **Content-Type**: `application/rss+xml`
- **Format**: RSS 2.0 XML

**Example Response:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
    <channel>
        <title>Rocket Blog</title>
        <link>http://localhost:8000</link>
        <description>A blog built with Rocket framework</description>
        <language>en-us</language>
        <lastBuildDate>Mon, 01 Jan 2024 12:00:00 GMT</lastBuildDate>
        <generator>Rocket Blog RSS Generator</generator>
        <item>
            <title>Sample Blog Post</title>
            <link>http://localhost:8000/blog/1</link>
            <description>This is the post excerpt...</description>
            <pubDate>Mon, 01 Jan 2024 10:00:00 GMT</pubDate>
            <guid>http://localhost:8000/blog/1</guid>
        </item>
    </channel>
</rss>
```

**RSS Discovery:**
The RSS feed is also discoverable via the HTML `<link>` tag in the page header:
```html
<link rel="alternate" type="application/rss+xml" title="Rocket Blog RSS Feed" href="/feed/rss" />
```

## 📊 Response Formats

### HTML Responses
Most endpoints return HTML pages using Tera templates:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Rocket Blog</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.2/dist/css/bootstrap.min.css" rel="stylesheet">
</head>
<body>
    <!-- Content -->
</body>
</html>
```

### Error Responses
Errors are handled through HTTP status codes and redirect responses:

- **404 Not Found**: Invalid blog post ID
- **401 Unauthorized**: Authentication required
- **403 Forbidden**: Insufficient permissions
- **500 Internal Server Error**: Server-side error

### Flash Messages
Success and error messages are displayed using Rocket's flash message system:

```html
<!-- Success message -->
<div class="alert alert-success">Post created successfully!</div>

<!-- Error message -->  
<div class="alert alert-danger">Authentication failed!</div>
```

## 🔧 Development API

For development and testing purposes, additional endpoints may be available:

### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "0.1.0"
}
```

## 📡 Future API Plans

### REST API (Planned)
A full REST API is planned with JSON responses:

```http
GET /api/v1/posts           # List posts (JSON)
POST /api/v1/posts          # Create post (JSON)
GET /api/v1/posts/{id}      # Get post (JSON)  
PUT /api/v1/posts/{id}      # Update post (JSON)
DELETE /api/v1/posts/{id}   # Delete post (JSON)

GET /api/v1/tags            # List tags (JSON)
GET /api/v1/tags/{name}/posts  # Posts by tag (JSON)

GET /api/v1/comments        # List comments (JSON)
POST /api/v1/comments       # Create comment (JSON)
```

### GraphQL API (Planned)
GraphQL endpoint for flexible querying:

```http
POST /graphql
Content-Type: application/json

{
  "query": "{ posts { id title tags { name } comments { author content } } }"
}
```

## 🛠️ Testing API Endpoints

### Using cURL

```bash
# List blog posts
curl "http://localhost:8000/blog/"

# Get specific post  
curl "http://localhost:8000/blog/1"

# Create post (with authentication)
curl -X POST "http://localhost:8000/blog/" \
  -H "Cookie: token=your_token" \
  -d "title=Test&content=Content"

# Add comment
curl -X POST "http://localhost:8000/comment/" \
  -d "author=Tester&content=Nice post&post_id=1"
```

### Using HTTPie

```bash
# List posts
http GET localhost:8000/blog/

# Create post
http POST localhost:8000/blog/ \
  Cookie:token=your_token \
  title="Test Post" \
  content="Test content"

# Add comment  
http POST localhost:8000/comment/ \
  author="John" \
  content="Great post!" \
  post_id:=1
```

### Using JavaScript Fetch

```javascript
// Create a new blog post
async function createPost(title, content, tags) {
  const response = await fetch('/blog/', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: new URLSearchParams({
      title: title,
      content: content,
      tags: tags
    }),
    credentials: 'same-origin' // Include cookies
  });
  
  if (response.ok) {
    console.log('Post created successfully');
  }
}

// Add a comment
async function addComment(author, content, postId) {
  const response = await fetch('/comment/', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: new URLSearchParams({
      author: author,
      content: content,
      post_id: postId
    })
  });
  
  if (response.ok) {
    console.log('Comment added successfully');
  }
}
```

## 🔒 Security Considerations

### Authentication
- Uses secure HTTP-only cookies
- Tokens are validated server-side
- Sessions expire automatically

### CSRF Protection
- **Current**: Not implemented (planned feature)
- **Planned**: CSRF tokens for form submissions

### Rate Limiting
- **Current**: Not implemented (planned feature)
- **Planned**: Rate limiting for API endpoints

### Input Validation
- Form data is validated server-side
- SQL injection protection via SeaORM
- XSS protection via template escaping

## 📝 API Changelog

### Version 0.1.0 (Current)
- Basic blog CRUD operations
- Comment system
- Tag support (backend only)
- Cookie-based authentication
- Media streaming with range requests

### Planned Version 0.2.0
- REST API endpoints with JSON responses
- Tag filtering endpoints
- CSRF protection
- Rate limiting
- Enhanced error handling

### Planned Version 0.3.0
- GraphQL API
- WebSocket support for real-time features
- API authentication tokens
- Advanced search endpoints

---

For more information about implementing new API endpoints, see the [Contributing Guide](CONTRIBUTING.md) and [Implementation Examples](TAG_INTEGRATION_EXAMPLE.md).