# Mock Database Fairing for Testing Rocket Applications

This module provides a test fairing that creates mock database connections for testing Rocket applications without requiring a real database.

## Overview

The `MockDatabaseFairing` allows you to:
- Create mock database connections with predefined test data
- Test Rocket routes and controllers with controlled database responses
- Simulate various database scenarios (empty results, errors, etc.)
- Test database operations like create, update, and delete

## Usage Examples

### Basic Usage

```rust
use crate::tests::fairings::mock_database::MockDatabaseFairing;

// Create a basic mock fairing with test data
let fairing = MockDatabaseFairing::with_test_post();
let mock_db = fairing.create_mock_connection();

// Use with your service
let blog_service = BlogService::new();
let result = blog_service.find_by_id(&mock_db, post_id).await;
```

### Builder Pattern

```rust
let fairing = MockDatabaseFairing::new()
    .with_account(test_account)
    .with_post(test_post)
    .with_exec_result(MockExecResult {
        last_insert_id: 1,
        rows_affected: 1,
    });

let mock_db = fairing.create_mock_connection();
```

### Testing Empty Database

```rust
// Test when no data is found
let mock_db = MockDatabaseFairing::create_empty_mock_connection();
let result = blog_service.find_by_id(&mock_db, post_id).await;
assert!(result.unwrap().is_none());
```

### Testing Rocket Routes

```rust
#[tokio::test]
async fn test_rocket_route_with_mock_db() {
    let fairing = MockDatabaseFairing::with_test_post_and_account();
    let rocket = rocket::build()
        .attach(fairing)
        .mount("/", routes![your_route]);

    // Use rocket testing client to test routes
}
```

## Available Methods

### Creation Methods
- `MockDatabaseFairing::new()` - Create empty fairing
- `MockDatabaseFairing::with_test_post()` - Create with a single test post
- `MockDatabaseFairing::with_test_post_and_account()` - Create with post and associated account
- `MockDatabaseFairing::create_empty_mock_connection()` - Static method for empty database

### Builder Methods
- `.with_post(post)` - Add a post to the mock data
- `.with_account(account)` - Add an account to the mock data
- `.with_exec_result(result)` - Add execution result for mutations

### Connection Methods
- `.create_mock_connection()` - Create the mock database connection
- `.create_joined_mock_connection()` - Create connection optimized for joined queries

## Testing Different Scenarios

### Single Post Query
```rust
let fairing = MockDatabaseFairing::with_test_post();
let mock_db = fairing.create_mock_connection();
// Tests find_by_id, find_by_seq_id
```

### Joined Queries (Post with Account)
```rust
let fairing = MockDatabaseFairing::with_test_post_and_account();
let mock_db = fairing.create_mock_connection();
// Tests find_by_seq_id_with_account
```

### Database Operations (Create/Update/Delete)
```rust
let fairing = MockDatabaseFairing::new()
    .with_post(draft_post)
    .with_exec_result(MockExecResult { 
        last_insert_id: 0, 
        rows_affected: 1 
    })
    .with_post(published_post); // Result after update

let mock_db = fairing.create_mock_connection();
// Tests publish_by_seq_id, create, update operations
```

## Integration with Existing Tests

This fairing integrates seamlessly with the existing SeaORM mock testing infrastructure and follows the same patterns used in `seaorm_mock_tests.rs`.

## Rocket Fairing Integration

The `MockDatabaseFairing` implements the Rocket `Fairing` trait and can be attached to a Rocket instance:

```rust
let rocket = rocket::build()
    .attach(MockDatabaseFairing::with_test_post())
    .mount("/", routes![your_routes]);
```

However, for most testing scenarios, you'll use the mock connection directly with your services rather than through the Rocket instance.