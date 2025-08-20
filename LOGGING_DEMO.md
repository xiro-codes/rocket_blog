# Enhanced Logging in Rocket Blog

This document demonstrates the comprehensive logging improvements added to the Rocket Blog application for development purposes.

## Logging Configuration

The application now features enhanced logging that:
- Outputs to both console (development mode) and file (`output.log`)
- Uses appropriate log levels (Debug in development, Info in production)
- Filters noisy dependencies (rocket, sqlx, etc.)
- Provides structured, contextual information

## Example Log Output

Here's what you'll see when running the application in development mode:

### Application Startup
```
[2024-01-15 10:30:45 INFO app] Starting Rocket Blog application...
[2024-01-15 10:30:45 DEBUG app] Development mode: true
[2024-01-15 10:30:45 DEBUG app] Seeding enabled: true
[2024-01-15 10:30:45 DEBUG app] Log level: Debug
[2024-01-15 10:30:45 INFO app] Building Rocket instance and attaching services...
[2024-01-15 10:30:45 INFO app] Starting database migrations...
[2024-01-15 10:30:46 INFO app] Database migrations completed successfully
[2024-01-15 10:30:46 INFO app] Registering application services...
[2024-01-15 10:30:46 DEBUG app] Creating AI provider service with OpenAI and Ollama providers
[2024-01-15 10:30:46 DEBUG app] Attaching services: Auth, Blog, Comment, OpenAI, Ollama, AIProvider, Reaction, Settings, Tag, Coordinator
[2024-01-15 10:30:46 INFO app] Attaching database seeding middleware
[2024-01-15 10:30:46 INFO app] Seeding middleware: checking if database seeding is needed
[2024-01-15 10:30:46 INFO app] Database is empty, creating seed data with 50 posts...
[2024-01-15 10:30:46 DEBUG app] Creating admin account
[2024-01-15 10:30:46 DEBUG app] Admin account created: admin (12345678-1234-5678-9abc-123456789012)
[2024-01-15 10:30:46 DEBUG app] Creating sample tags
[2024-01-15 10:30:46 DEBUG app] Created 7 tags
[2024-01-15 10:30:46 DEBUG app] No sample video found, posts will be text-only
[2024-01-15 10:30:46 DEBUG app] Creating 49 sample posts with comments and tags
[2024-01-15 10:30:47 DEBUG app] Inserting 49 posts into database
[2024-01-15 10:30:47 DEBUG app] Inserting 245 comments into database
[2024-01-15 10:30:47 DEBUG app] Inserting 98 post-tag relationships into database
[2024-01-15 10:30:47 INFO app] Database seeding completed successfully
[2024-01-15 10:30:47 INFO app] Registering application controllers...
[2024-01-15 10:30:47 DEBUG app] Attaching controllers: Index (/), Auth (/auth), Blog (/blog), Comment (/comment), Feed (/feed), Settings (/settings)
[2024-01-15 10:30:47 INFO app] Attaching controllers and static file server
```

### HTTP Request Handling
```
[2024-01-15 10:31:15 INFO app] Home page accessed - redirecting to blog
[2024-01-15 10:31:15 INFO app] Blog list view requested - Page: 1, Size: 10, Client IP: 127.0.0.1
[2024-01-15 10:31:15 DEBUG app] Anonymous user viewing blog list
[2024-01-15 10:31:15 DEBUG app] Fetching blog list data via coordinator service
[2024-01-15 10:31:15 DEBUG app] Coordinator: getting blog list data - page=1, size=10, client_ip=127.0.0.1
[2024-01-15 10:31:15 DEBUG app] Coordinator: checking if accounts exist
[2024-01-15 10:31:15 DEBUG app] Found existing account(s) in database
[2024-01-15 10:31:15 DEBUG app] Coordinator: no token provided, treating as non-admin
[2024-01-15 10:31:15 DEBUG app] Coordinator: user is not admin, excluding draft posts
[2024-01-15 10:31:15 DEBUG app] Coordinator: fetching paginated posts
[2024-01-15 10:31:15 DEBUG app] Paginating blog posts: page=1, page_size=10, include_drafts=false
[2024-01-15 10:31:15 DEBUG app] Filtering out draft posts
[2024-01-15 10:31:15 DEBUG app] Fetching page 1 of 5 pages
[2024-01-15 10:31:15 DEBUG app] Successfully fetched 10 posts for page 1
[2024-01-15 10:31:15 DEBUG app] Coordinator: fetching all tags
[2024-01-15 10:31:15 DEBUG app] Coordinator: fetching reaction summaries for 10 posts
[2024-01-15 10:31:15 INFO app] Coordinator: blog list data prepared - 10 posts, 7 tags, 10 reaction summaries
[2024-01-15 10:31:15 DEBUG app] Blog list data fetched successfully - 10 posts, 5 pages
```

### Authentication Flow
```
[2024-01-15 10:32:00 DEBUG app] Serving login page
[2024-01-15 10:32:00 DEBUG app] Found existing account(s) in database
[2024-01-15 10:32:00 DEBUG app] Login page served successfully
[2024-01-15 10:32:10 INFO app] Login attempt from controller for username: admin
[2024-01-15 10:32:10 DEBUG app] Authentication attempt for username: admin
[2024-01-15 10:32:10 DEBUG app] User found in database: admin
[2024-01-15 10:32:10 INFO app] User successfully authenticated: admin (ID: 12345678-1234-5678-9abc-123456789012)
[2024-01-15 10:32:10 DEBUG app] Generated token: 87654321-4321-8765-cba9-876543210987
[2024-01-15 10:32:10 INFO app] Login successful - redirecting to blog
```

### Blog Post Creation
```
[2024-01-15 10:33:00 INFO app] Creating new blog post: title='My First Post', author_id=12345678-1234-5678-9abc-123456789012, action=Some("publish")
[2024-01-15 10:33:00 DEBUG app] Converting markdown to HTML
[2024-01-15 10:33:00 DEBUG app] Generated post ID: abcdef12-3456-7890-abcd-ef1234567890
[2024-01-15 10:33:00 DEBUG app] No file to upload
[2024-01-15 10:33:00 DEBUG app] Publishing post immediately
[2024-01-15 10:33:00 DEBUG app] Inserting post into database
[2024-01-15 10:33:00 INFO app] Blog post created successfully: My First Post (abcdef12-3456-7890-abcd-ef1234567890)
```

### Error Scenarios
```
[2024-01-15 10:34:00 WARN app] Login attempt failed - user not found: baduser
[2024-01-15 10:34:00 WARN app] Login failed - redirecting back to login form
[2024-01-15 10:34:30 DEBUG app] AuthenticatedUser guard: checking authentication
[2024-01-15 10:34:30 DEBUG app] AuthenticatedUser guard: no token cookie found
[2024-01-15 10:34:30 DEBUG app] AuthenticatedUser guard: authentication failed
[2024-01-15 10:34:45 WARN app] Unhandled route accessed - redirecting to home page
```

## Benefits for Development

This enhanced logging provides:

1. **Request Tracing**: Track HTTP requests from start to finish
2. **Performance Insights**: See database query counts and timing
3. **Authentication Debugging**: Monitor login attempts and token validation
4. **Error Diagnosis**: Detailed error contexts for troubleshooting
5. **Feature Development**: Understanding data flow through services
6. **Database Operations**: Monitor seeding, migrations, and queries

## Using the Logs

- **Console Output**: Real-time feedback during development
- **File Logging**: Persistent logs in `output.log` for analysis
- **Log Levels**: Use `RUST_LOG=debug` for maximum detail
- **Filtering**: Noise from dependencies is automatically filtered

## Production Considerations

In production (release builds):
- Console logging is disabled
- Only INFO level and above are logged
- Focus on operational events rather than debug details
- File logging continues for audit trails