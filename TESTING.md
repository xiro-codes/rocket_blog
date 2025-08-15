# Testing Guide for Rocket Blog

## Overview

This document describes the test suite for the rocket_blog application. The tests are organized into unit tests and integration tests covering the main components of the application.

## Test Structure

### Unit Tests (in `tests/` directory)

1. **`base_service_tests.rs`** - Tests for the BaseService utility functions
   - UUID generation
   - Error handling utilities
   - Service instantiation

2. **`auth_service_tests.rs`** - Tests for authentication service logic
   - Service creation and state management
   - Type validation for tokens and account IDs

3. **`blog_service_tests.rs`** - Tests for blog service components
   - Service instantiation
   - Constants validation

4. **`tag_service_tests.rs`** - Tests for tag management service
   - Tag name slugification
   - Color handling
   - Service creation

5. **`middleware_tests.rs`** - Tests for custom middleware
   - Seeding middleware configuration
   - Constants and path validation

6. **`main_app_tests.rs`** - Tests for main application setup
   - Configuration handling
   - Error catcher behavior
   - Logging filter logic

7. **`route_tests.rs`** - Tests for HTTP routes and controllers
   - Index route redirections
   - Controller instantiation

8. **`integration_tests.rs`** - Full application integration tests
   - Application building and startup
   - Route behavior verification
   - Component integration

## Running Tests

### Prerequisites

1. Rust toolchain installed
2. Project dependencies resolved

### Running All Tests

```bash
cargo test
```

### Running Specific Test Files

```bash
# Run only unit tests
cargo test --tests

# Run a specific test file
cargo test --test base_service_tests

# Run tests with output
cargo test -- --nocapture

# Run tests with verbose output
cargo test --verbose
```

### Running Tests in Development

```bash
# Watch mode for continuous testing during development
cargo watch -x test

# Run tests with specific filter
cargo test auth_service
```

## Test Categories

### Unit Tests
- **Focus**: Individual functions and methods
- **Dependencies**: Minimal, mostly pure functions
- **Speed**: Fast execution
- **Coverage**: Core business logic, utilities, data transformations

### Integration Tests  
- **Focus**: Component interactions, HTTP endpoints
- **Dependencies**: May require database, external services
- **Speed**: Slower execution
- **Coverage**: End-to-end workflows, API contracts

## Test Data and Fixtures

### Constants Used in Tests
- Default colors for tags: `#007bff`
- Sample paths: `static/sample_video.webm`
- Data directories: `/home/tod/.local/share/blog`

### Mock Data Patterns
- UUID generation for IDs
- Slugified tag names (`"Hello World"` → `"hello-world"`)
- Bcrypt password hashing
- Lorem ipsum text generation

## Testing Best Practices

### Current Implementation
- **Isolation**: Each test is independent
- **Naming**: Descriptive test names following `test_<functionality>_<expected_behavior>`
- **Assertions**: Clear success/failure criteria
- **Documentation**: Extensive comments explaining test purpose

### Areas for Future Enhancement

1. **Database Testing**
   - Set up test database fixtures
   - Test database migrations
   - Test CRUD operations end-to-end

2. **Authentication Testing**
   - Mock HTTP requests with authentication
   - Test token validation flows
   - Test authorization middleware

3. **File Handling Testing**
   - Mock file upload/download operations
   - Test video file processing
   - Test file path validation

4. **Template Testing**
   - Test template rendering
   - Test template data binding
   - Test error page rendering

## Continuous Integration

### Recommended CI Pipeline
```yaml
- name: Run Tests
  run: |
    cargo test --verbose
    cargo test --doc
```

### Test Coverage
To generate test coverage reports:
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Debugging Tests

### Running Individual Tests
```bash
# Run a single test with full output
cargo test test_generate_id_creates_valid_uuid -- --exact --nocapture
```

### Test Debugging Tips
- Use `println!` or `dbg!` macros for debug output
- Add `--nocapture` flag to see print statements
- Use `--test-threads=1` for sequential execution
- Check test names with `cargo test -- --list`

## Current Limitations

1. **Database Operations**: Most database-dependent tests are marked as needing integration setup
2. **External Dependencies**: Some tests require network access or external services
3. **File System**: Tests requiring file I/O may need temporary directories
4. **Async Testing**: Some async functionality requires tokio test runtime

## Future Testing Enhancements

1. **Mock Database**: Implement in-memory database for faster testing
2. **Test Fixtures**: Create reusable test data and scenarios
3. **Performance Tests**: Add benchmarking for critical paths
4. **Security Tests**: Add tests for authentication and authorization flows
5. **Error Handling**: Comprehensive error scenario testing