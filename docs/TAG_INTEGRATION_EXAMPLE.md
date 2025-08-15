# Example Integration Guide: Adding Tags UI

This document demonstrates how to integrate the tag system UI with the existing rocket_blog application. The backend foundation is already complete.

## Quick Integration Steps

### 1. Add Tag Input to Blog Forms

**File**: `templates/blog/create.html.tera`

Add after the existing form fields:

```html
<div class="mb-3">
    <label for="tags" class="form-label">Tags (comma-separated)</label>
    <input type="text" class="form-control" name="tags" id="tags" 
           placeholder="e.g. rust, web development, tutorial">
    <div class="form-text">Separate multiple tags with commas</div>
</div>
```

**File**: `templates/blog/edit.html.tera`

Add the same field with current tags pre-filled:

```html
<div class="mb-3">
    <label for="tags" class="form-label">Tags</label>
    <input type="text" class="form-control" name="tags" id="tags" 
           value="{% for tag in post.tags %}{{ tag.name }}{% if not loop.last %}, {% endif %}{% endfor %}">
</div>
```

### 2. Update Blog Controller to Handle Tags

**File**: `src/controllers/blog.rs`

Add tag handling to the create function:

```rust
use crate::services::TagService;

#[post("/create", format = "multipart/form-data", data = "<form_data>")]
async fn create(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    tag_service: &State<TagService>,
    auth_service: &State<AuthService>,
    jar: &CookieJar<'_>,
    form_data: Form<FormDTO<'_>>,
) -> Result<Flash<Redirect>, Status> {
    if let Some(token) = ControllerBase::check_auth(jar)? {
        let db = conn.into_inner();
        let token = Uuid::parse_str(&token).unwrap();
        if let Some(account) = auth_service.check_token(db, token).await {
            if !account.admin {
                return Err(Status::Unauthorized);
            }

            let form = form_data.into_inner();
            let post = service
                .create(db, account.id, &mut form)
                .await
                .unwrap();

            // Handle tags if provided
            if let Some(tags_str) = form.tags {
                for tag_name in tags_str.split(',') {
                    let tag_name = tag_name.trim();
                    if !tag_name.is_empty() {
                        let tag = tag_service.find_or_create_tag(db, tag_name).await.unwrap();
                        let _ = tag_service.add_tag_to_post(db, post.id, tag.id).await;
                    }
                }
            }

            return Ok(ControllerBase::success_redirect(
                format!("/blog/{}", post.seq_id),
                "Post successfully created with tags"
            ));
        }
    }
    Err(Status::Unauthorized)
}
```

### 3. Display Tags in Post Views

**File**: `templates/blog/detail.html.tera`

Add after the post title:

```html
{% if post.tags %}
<div class="mb-3">
    {% for tag in post.tags %}
    <span class="badge bg-primary me-1" style="background-color: {{ tag.color }}!important;">
        {{ tag.name }}
    </span>
    {% endfor %}
</div>
{% endif %}
```

**File**: `templates/blog/list.html.tera`

Add tags to each post in the list:

```html
{% for post in posts %}
<li class="list-group-item">
    <a class="link-offset-2 link-offset-3-hover link-underline link-underline-opacity-0 link-underline-opacity-75-hover"
       href="/blog/{{post.seq_id}}">{{post.title}}</a>
    {% if post.tags %}
    <div class="mt-1">
        {% for tag in post.tags %}
        <small><span class="badge bg-secondary me-1">{{ tag.name }}</span></small>
        {% endfor %}
    </div>
    {% endif %}
</li>
{% endfor %}
```

### 4. Add Tag Filtering

**File**: `src/controllers/blog.rs`

Add a new route for filtering by tag:

```rust
#[get("/tag/<slug>?<page>&<page_size>")]
async fn posts_by_tag(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    tag_service: &State<TagService>,
    slug: String,
    page: Option<u64>,
    page_size: Option<u64>,
    jar: &CookieJar<'_>,
) -> Result<Template, Status> {
    let token = ControllerBase::check_auth(jar).unwrap_or_default();
    let db = conn.into_inner();
    
    // Find tag by slug
    let tag = tag::Entity::find()
        .filter(tag::Column::Slug.eq(slug))
        .one(db)
        .await
        .map_err(|_| Status::NotFound)?
        .ok_or(Status::NotFound)?;
    
    // Get posts with this tag (implementation needed in BlogService)
    let (posts, page, page_size, num_pages) = service
        .paginate_posts_by_tag(db, tag.id, page, page_size)
        .await
        .unwrap();

    Ok(Template::render(
        "blog/list",
        context! {
            posts,
            page,
            page_size,
            num_pages,
            token,
            tag_filter: tag.name,
            title: format!("Posts tagged with '{}'", tag.name)
        },
    ))
}
```

### 5. Add Tag Cloud Widget

**File**: `templates/blog/list.html.tera`

Add a tag cloud sidebar:

```html
<div class="row">
    <div class="col-md-8">
        <!-- Existing post list -->
    </div>
    <div class="col-md-4">
        <div class="card">
            <div class="card-header">
                <h5>Tags</h5>
            </div>
            <div class="card-body">
                {% for tag in all_tags %}
                <a href="/blog/tag/{{ tag.slug }}" 
                   class="badge text-decoration-none me-1 mb-1"
                   style="background-color: {{ tag.color }};">
                    {{ tag.name }}
                </a>
                {% endfor %}
            </div>
        </div>
    </div>
</div>
```

## Benefits Achieved

1. **Content Organization**: Posts can be categorized and filtered
2. **Better Navigation**: Users can find related content easily
3. **SEO Improvement**: Tag pages create more entry points
4. **Visual Appeal**: Colorful tag badges enhance the UI
5. **Scalable**: Easy to add more tag-related features later

## Additional Features to Consider

- **Tag Management Interface**: Admin page to manage tags
- **Tag Statistics**: Show post counts for each tag
- **Tag Suggestions**: Auto-complete when typing tags
- **Related Posts**: Show posts with similar tags
- **Tag RSS Feeds**: Individual RSS feeds per tag

This demonstrates how the solid foundation enables rapid feature development with minimal code changes.