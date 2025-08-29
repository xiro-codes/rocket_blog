# CodeIgniter3-Style Query Builder for Sea-ORM

This document describes the new procedural macro that generates a CodeIgniter3-style query builder for Sea-ORM entities, providing a fluent, type-safe interface for building database queries.

## Overview

The `QueryBuilder` derive macro automatically generates a query builder for Sea-ORM entities that provides a familiar, fluent interface similar to CodeIgniter 3's Active Record pattern. This makes database queries more readable and easier to write while maintaining full type safety.

## Features

- **Fluent Interface**: Chain method calls to build complex queries
- **Type Safety**: Compile-time checks ensure valid column references and types
- **CodeIgniter3 Compatibility**: Similar API to CI3's Active Record pattern
- **Sea-ORM Integration**: Works seamlessly with existing Sea-ORM entities
- **Automatic Generation**: Minimal setup required with derive macro

## Usage

### 1. Add the Derive Macro

Add the `QueryBuilder` derive to your Sea-ORM model:

```rust
use query_builder_macro::QueryBuilder;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, QueryBuilder)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub text: String,
    pub excerpt: Option<String>,
    pub draft: Option<bool>,
    pub date_published: DateTime,
    pub account_id: Uuid,
}
```

### 2. Use the Query Builder

Once the derive macro is applied, you can use the generated query builder:

```rust
use models::post;
use sea_orm::{DbConn, Order};

// Basic query with WHERE condition
let posts = post::Entity::query()
    .where_eq(post::Column::Draft, false)
    .order_desc(post::Column::DatePublished)
    .limit(10)
    .get(&db)
    .await?;

// Search with LIKE pattern
let search_results = post::Entity::query()
    .like(post::Column::Title, "rust")
    .where_eq(post::Column::Draft, false)
    .order_asc(post::Column::Title)
    .get(&db)
    .await?;

// Get first result
let first_post = post::Entity::query()
    .where_eq(post::Column::AccountId, author_id)
    .where_eq(post::Column::Draft, false)
    .order_desc(post::Column::DatePublished)
    .first(&db)
    .await?;

// Count records
let draft_count = post::Entity::query()
    .where_eq(post::Column::Draft, true)
    .count(&db)
    .await?;
```

## Available Methods

### WHERE Conditions

- `where_eq(column, value)` - Equality condition
- `where_in(column, values)` - IN condition with multiple values
- `like(column, pattern)` - LIKE condition with automatic wildcards
- `where_not_null(column)` - NOT NULL condition
- `where_null(column)` - IS NULL condition
- `or_where_eq(column, value)` - OR condition (basic implementation)

### Ordering

- `order_by(column, direction)` - Order by column with specified direction
- `order_asc(column)` - Order by column ascending
- `order_desc(column)` - Order by column descending

### Limiting and Pagination

- `limit(count)` - Limit number of results
- `offset(count)` - Skip number of results

### Execution

- `get(db)` - Execute query and return all results
- `first(db)` - Execute query and return first result (Option)
- `count(db)` - Count matching records

## Integration with Existing Services

The query builder can be easily integrated into existing service methods:

```rust
impl BlogService {
    /// Find recent published posts using query builder
    pub async fn find_recent_published_posts_qb(
        &self,
        db: &DbConn,
        limit: Option<u64>,
    ) -> Result<Vec<models::post::Model>, DbErr> {
        let limit = limit.unwrap_or(10);
        
        models::post::Entity::query()
            .where_eq(models::post::Column::Draft, false)
            .order_desc(models::post::Column::DatePublished)
            .limit(limit)
            .get(db)
            .await
    }
    
    /// Find posts by author with optional draft inclusion
    pub async fn find_posts_by_author_qb(
        &self,
        db: &DbConn,
        author_id: Uuid,
        include_drafts: bool,
    ) -> Result<Vec<models::post::Model>, DbErr> {
        let mut query = models::post::Entity::query()
            .where_eq(models::post::Column::AccountId, author_id);

        if !include_drafts {
            query = query.where_eq(models::post::Column::Draft, false);
        }

        query
            .order_desc(models::post::Column::DatePublished)
            .get(db)
            .await
    }
}
```

## Examples

### Complex Query Example

```rust
// Find recent posts with excerpts, paginated
let posts = post::Entity::query()
    .where_eq(post::Column::Draft, false)
    .where_not_null(post::Column::Excerpt)
    .order_desc(post::Column::DatePublished)
    .limit(20)
    .offset(40)
    .get(&db)
    .await?;
```

### Search Example

```rust
// Search posts by title pattern
let search_results = post::Entity::query()
    .like(post::Column::Title, search_term)
    .where_eq(post::Column::Draft, false)
    .order_desc(post::Column::DatePublished)
    .limit(10)
    .get(&db)
    .await?;
```

### Counting Example

```rust
// Count published posts by author
let post_count = post::Entity::query()
    .where_eq(post::Column::AccountId, author_id)
    .where_eq(post::Column::Draft, false)
    .count(&db)
    .await?;
```

## Comparison with Traditional Sea-ORM

### Traditional Sea-ORM
```rust
let posts = Post::find()
    .filter(post::Column::Draft.eq(false))
    .order_by_desc(post::Column::DatePublished)
    .limit(10)
    .all(&db)
    .await?;
```

### Query Builder
```rust
let posts = post::Entity::query()
    .where_eq(post::Column::Draft, false)
    .order_desc(post::Column::DatePublished)
    .limit(10)
    .get(&db)
    .await?;
```

## Benefits

1. **Familiar API**: Developers coming from CodeIgniter 3 will find the interface familiar
2. **Readability**: Method chaining creates more readable queries
3. **Type Safety**: Full compile-time checking of column types and names
4. **Consistency**: Uniform API across all entities
5. **Maintainability**: Generated code reduces boilerplate and potential errors

## Current Limitations

1. **Select Fields**: Currently returns full models; column selection is commented out for simplicity
2. **Complex Joins**: JOIN operations are not yet implemented
3. **Advanced OR Conditions**: OR logic needs enhancement for complex scenarios
4. **Subqueries**: Not yet supported

## Future Enhancements

- Column selection support
- JOIN operations
- Advanced OR/AND condition grouping
- Subquery support
- Raw SQL injection points
- Custom expression support
- Aggregation functions

## Testing

The query builder includes comprehensive tests demonstrating various usage patterns:

```rust
#[tokio::test]
async fn test_query_builder_basic() {
    let posts = QueryBuilderExample::find_published_posts(&db).await;
    assert!(posts.is_ok());
}
```

See `src/examples/query_builder_example.rs` for complete test examples.

## Contributing

To extend the query builder:

1. Modify the proc macro in `query_builder_macro/src/lib.rs`
2. Add new method implementations to the generated builder
3. Update tests and documentation
4. Ensure backward compatibility

The macro uses `quote!` and `syn` for code generation and parsing, making it straightforward to add new functionality while maintaining type safety.