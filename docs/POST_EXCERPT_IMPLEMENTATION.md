# Post Excerpt/Summary System Implementation

## Overview

The Post Excerpt/Summary System adds the ability to display short previews of blog posts in list views, improving content discoverability and user engagement. This feature was implemented following the same architectural patterns used for the tag system.

## Features

### Excerpt Field
- **Optional Field**: Posts can have custom excerpts or auto-generated summaries
- **Auto-Generation**: If no excerpt is provided, the system automatically generates one from the post content
- **Character Limit**: Auto-generated excerpts are limited to 200 characters and break at word boundaries
- **Clean Text**: Auto-generated excerpts strip markdown formatting and headers for readability

### UI Improvements
- **Enhanced List View**: Post listings now show title and excerpt in card format instead of simple list items
- **Form Integration**: Create and edit forms include excerpt input field with helpful placeholder text
- **Read More Button**: List view includes "Read More" buttons for better navigation

## Implementation Details

### Database Changes

#### Migration: `m20241202_000001_add_excerpt.rs`
```rust
// Adds excerpt column to post table
ALTER TABLE post ADD COLUMN excerpt VARCHAR NULL;
```

The migration:
- Adds a nullable `excerpt` column to the `post` table
- Follows the same naming convention as the tags migration
- Includes proper up/down migration methods

### Model Updates

#### Post Model (`models/src/post.rs`)
```rust
pub struct Model {
    // ... existing fields
    pub excerpt: Option<String>,  // New excerpt field
    // ... rest of fields
}
```

#### PostTitleResult DTO (`models/src/dto.rs`)
```rust
pub struct PostTitleResult {
    pub id: Uuid,
    pub seq_id: i32,
    pub title: String,
    pub excerpt: Option<String>,  // New field for list views
}
```

### Form Integration

#### FormDTO (`src/dto/mod.rs`)
```rust
pub struct FormDTO<'r> {
    pub title: String,
    pub text: String,
    pub excerpt: Option<String>,  // New excerpt field
    pub file: TempFile<'r>,
    pub tags: Option<String>,
}
```

### Service Layer

#### Blog Service (`src/services/blog.rs`)

**Excerpt Generation Logic:**
```rust
fn generate_excerpt(text: &str, provided_excerpt: Option<String>) -> Option<String> {
    // Use provided excerpt if available and not empty
    if let Some(excerpt) = provided_excerpt {
        if !excerpt.trim().is_empty() {
            return Some(excerpt.trim().to_string());
        }
    }
    
    // Auto-generate from content:
    // 1. Clean markdown formatting and HTML
    // 2. Skip headers and empty lines
    // 3. Take first 200 characters
    // 4. Break at word boundary
    // 5. Add ellipsis if truncated
}
```

**Updated Methods:**
- `create()`: Generates excerpt when creating posts
- `update_by_id()`: Updates excerpt when editing posts
- `update_by_seq_id()`: Updates excerpt when editing posts
- `paginate_with_title()`: Includes excerpt in list queries
- `paginate_posts_by_tag()`: Includes excerpt in tag-filtered queries
- `find_many_with_title()`: Includes excerpt in bulk queries

### Template Updates

#### Create Form (`templates/blog/create.html.tera`)
```html
<div>
    <label for="excerpt" class="form-label">Excerpt</label>
    <textarea name="excerpt" id="excerpt" rows="3" class="form-control" 
              placeholder="Brief summary of the post (optional - will be auto-generated if left empty)"></textarea>
    <div class="form-text">Optional short summary that will appear in post listings.</div>
</div>
```

#### Edit Form (`templates/blog/edit.html.tera`)
```html
<div>
    <label for="excerpt" class="form-label">Excerpt</label>
    <textarea name="excerpt" id="excerpt" rows="3" class="form-control">
        {% if post.excerpt %}{{post.excerpt}}{% endif %}
    </textarea>
</div>
```

#### List View (`templates/blog/list.html.tera`)
```html
{% for post in posts %}
<div class="card mb-3">
    <div class="card-body">
        <h5 class="card-title">
            <a href="/blog/{{post.seq_id}}">{{post.title}}</a>
        </h5>
        {% if post.excerpt %}
        <p class="card-text text-muted">{{post.excerpt}}</p>
        {% endif %}
        <a href="/blog/{{post.seq_id}}" class="btn btn-outline-primary btn-sm">Read More</a>
    </div>
</div>
{% endfor %}
```

## Usage

### For Content Creators

1. **Creating Posts with Custom Excerpt:**
   - Fill in the excerpt field when creating a new post
   - Keep it concise and engaging (recommended: 50-150 characters)
   - The excerpt should entice readers to read the full post

2. **Auto-Generated Excerpts:**
   - Leave the excerpt field empty for automatic generation
   - The system will use the first paragraph of your content
   - Clean, readable text without markdown formatting

3. **Editing Excerpts:**
   - Use the edit form to update or add excerpts to existing posts
   - Existing posts without excerpts will get auto-generated ones

### For Developers

#### Database Migration
```bash
# Apply the migration
cargo run --bin migration up

# Rollback if needed
cargo run --bin migration down
```

#### Excerpt Generation Algorithm
The auto-generation follows these steps:
1. Split content into lines and trim whitespace
2. Filter out empty lines and headers (lines starting with #)
3. Join remaining lines with spaces
4. Truncate to 200 characters
5. Find the last space within 200 characters to avoid breaking words
6. Add "..." if content was truncated

## Benefits

### User Experience
- **Better Content Discovery**: Users can quickly scan post previews
- **Improved Navigation**: "Read More" buttons provide clear CTAs
- **Visual Appeal**: Card-based layout is more modern and engaging
- **Mobile Friendly**: Cards stack well on mobile devices

### SEO Benefits
- **Meta Descriptions**: Excerpts can be used for meta descriptions
- **Search Snippets**: Provide better content for search result snippets
- **Social Sharing**: Excerpts improve social media share previews

### Content Management
- **Flexible Options**: Choose custom excerpts or auto-generation
- **Backward Compatible**: Existing posts work without modifications
- **Consistent Display**: All posts have previews, either custom or generated

## Technical Architecture

### Following Established Patterns

The excerpt system follows the same architectural patterns as the tag system:

1. **Database First**: Schema changes via migrations
2. **Model Layer**: Entity updates with proper types
3. **Service Layer**: Business logic for excerpt handling
4. **Controller Layer**: No changes needed (reuses existing endpoints)
5. **Template Layer**: Enhanced UI components

### Data Flow

1. **Creation**: User submits form → FormDTO captures excerpt → Service generates/processes → Model stores
2. **Display**: Query includes excerpt → PostTitleResult carries data → Template renders
3. **Update**: Edit form shows current excerpt → Service processes changes → Model updates

### Backward Compatibility

- Existing posts continue to work without excerpts
- Migration adds nullable column
- Templates handle missing excerpts gracefully
- Auto-generation provides fallback content

## Future Enhancements

### Potential Improvements
1. **Rich Text Excerpts**: Support for basic formatting in excerpts
2. **SEO Integration**: Automatic meta description generation
3. **Social Media**: Open Graph and Twitter Card integration
4. **Search Integration**: Use excerpts in search results
5. **API Endpoints**: Include excerpts in REST API responses

### Configuration Options
1. **Character Limits**: Configurable excerpt length limits
2. **Generation Rules**: Customizable auto-generation algorithms
3. **Template Variants**: Multiple list view styles
4. **Language Support**: Multi-language excerpt handling

## Testing

### Manual Testing Checklist
- [ ] Create new post with custom excerpt
- [ ] Create new post without excerpt (auto-generation)
- [ ] Edit existing post to add excerpt
- [ ] Edit existing post to modify excerpt
- [ ] Verify list view displays excerpts correctly
- [ ] Test mobile responsiveness
- [ ] Verify excerpt appears in tag-filtered views

### Edge Cases
- [ ] Very long excerpts (truncation)
- [ ] Empty excerpts (fallback to auto-generation)
- [ ] Posts with only headers (excerpt generation)
- [ ] Posts with markdown formatting (clean extraction)

## Conclusion

The Post Excerpt/Summary System enhances the Rocket Blog by providing better content previews and improved user experience. The implementation follows established patterns, maintains backward compatibility, and provides both manual and automatic excerpt generation options.

The feature significantly improves the blog's usability while maintaining the clean, minimal architecture of the existing codebase.