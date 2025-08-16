# Text Search Implementation

This document demonstrates the text search functionality implemented for the Rocket Blog.

## Features Implemented

### 1. Search Service Method
- **Location**: `src/services/blog.rs`
- **Method**: `search_posts()`
- **Functionality**: Searches both post titles and content using PostgreSQL LIKE queries
- **Draft Support**: Respects draft status based on user authentication

### 2. Search Controller Endpoint
- **Location**: `src/controllers/blog.rs`
- **Route**: `GET /blog/search?q=<query>&page=<page>&page_size=<page_size>`
- **Functionality**: Handles search requests and renders search results

### 3. Search Form in Navigation
- **Location**: `templates/base.html.tera`
- **Functionality**: Global search form in the navigation header
- **Styling**: Matches the terminal theme with green text on dark background

### 4. Search Results Template
- **Location**: `templates/blog/search.html.tera`
- **Functionality**: Displays search results with pagination support
- **Features**:
  - Shows search query and result count
  - Handles empty results gracefully
  - Includes tag cloud in sidebar
  - Pagination for large result sets

## Usage Examples

### Basic Search
```
GET /blog/search?q=rust
```
Searches for posts containing "rust" in title or content.

### Paginated Search
```
GET /blog/search?q=programming&page=2&page_size=10
```
Searches for "programming" and shows page 2 with 10 results per page.

### Empty Search
```
GET /blog/search
```
Shows search page without results, prompting user to enter search terms.

## Search Behavior

### For Regular Users (Not Admin)
- Only searches published posts (draft=false)
- Cannot see draft posts in search results

### For Admin Users
- Searches both published and draft posts
- Draft posts are marked with [DRAFT] badge in results

### Search Algorithm
- Uses PostgreSQL LIKE queries with wildcards (`%query%`)
- Searches both post title and text content
- Results ordered by date published (newest first)
- Case-insensitive search

## Code Structure

### Service Layer
```rust
pub async fn search_posts(
    &self,
    db: &DbConn,
    query_text: &str,
    page: Option<u64>,
    page_size: Option<u64>,
    include_drafts: bool,
) -> Result<(Vec<PostTitleResult>, u64, u64, u64), DbErr>
```

### Controller Layer
```rust
#[get("/search?<q>&<page>&<page_size>")]
async fn search(/* parameters */) -> Result<Template, Status>
```

### Template Integration
The search form is available globally in the navigation header and submits to `/blog/search` using GET method for SEO-friendly URLs.

## Testing

- All existing tests pass
- New unit test added for search method existence
- Code compiles successfully
- Implementation follows existing architectural patterns

## UI Integration

The search feature integrates seamlessly with the existing terminal theme:
- Green text (#00ff41) on dark background
- Terminal-style buttons with brackets (e.g., [SEARCH])
- Consistent styling with other blog pages
- Responsive design with Bootstrap classes