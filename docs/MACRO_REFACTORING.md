# Custom Macros and Traits Refactoring

This document outlines the custom macros and traits implemented to refactor the app crate, reducing boilerplate code and improving maintainability.

## Overview

The refactoring introduces several key patterns:
- **Service Traits**: Common interfaces for services
- **Controller Traits**: Common behaviors for controllers  
- **Service Macros**: Reduce boilerplate in service creation
- **Controller Macros**: Simplify controller implementations
- **Registry Macros**: Automatically generate service/controller registries
- **Helper Macros**: Common patterns for authentication and authorization

## Service System

### Service Traits

#### `ServiceHelpers`
Common utility methods for all services:
```rust
pub trait ServiceHelpers {
    fn generate_id() -> Uuid;
    fn handle_not_found<T>(result: Option<T>, entity_name: &str) -> Result<T, DbErr>;
    fn db_error_to_status(err: &DbErr) -> rocket::http::Status;
}
```

#### `ManagedService` 
Trait for services that can be managed by Rocket:
```rust
pub trait ManagedService: Send + Sync + 'static {
    fn new() -> Self;
}
```

#### `CrudService<T, CreateDto, UpdateDto>`
Generic trait for CRUD operations on entities:
```rust
pub trait CrudService<T, CreateDto, UpdateDto> {
    fn create(&self, db: &DbConn, data: CreateDto) -> impl Future<Output = Result<T::Model, DbErr>> + Send;
    fn find_by_id(&self, db: &DbConn, id: Uuid) -> impl Future<Output = Result<Option<T::Model>, DbErr>> + Send;
    fn update_by_id(&self, db: &DbConn, id: Uuid, data: UpdateDto) -> impl Future<Output = Result<T::Model, DbErr>> + Send;
    fn delete_by_id(&self, db: &DbConn, id: Uuid) -> impl Future<Output = Result<DeleteResult, DbErr>> + Send;
}
```

### Service Macros

#### `impl_service!(ServiceType)`
Implements common service patterns:
```rust
impl_service!(TagService);
// Generates ManagedService and ServiceHelpers implementations
```

#### `impl_service_with_base!(ServiceType)`
For services with a `base: BaseService` field:
```rust
pub struct TagService {
    base: BaseService,
}

impl_service_with_base!(TagService);
// Generates constructor and trait implementations
```

#### `impl_service_custom!(ServiceType)`
For services with custom constructors:
```rust
pub struct SettingsService {
    base: BaseService,
    encryption_key: [u8; 32],
}

impl SettingsService {
    pub fn new() -> Self {
        // Custom initialization logic
        Self {
            base: BaseService::new(),
            encryption_key: [/* custom key */],
        }
    }
}

impl_service_custom!(SettingsService);
```

#### `create_service_registry!(RegistryName, [ServiceList])`
Automatically generates service registries:
```rust
create_service_registry!(
    BlogServiceRegistry,
    [
        AuthService,
        BlogService,
        CommentService,
        TagService,
        SettingsService,
    ]
);

// Generates:
// - attach_all_services() method
// - fairing() method
```

## Controller System

### Controller Traits

#### `ControllerHelpers`
Common controller functionality:
```rust
pub trait ControllerHelpers {
    fn check_auth(jar: &CookieJar<'_>) -> Result<Option<String>, Status>;
    fn require_auth(jar: &CookieJar<'_>) -> Result<String, Status>;
    fn success_redirect<T: Into<String>, U: Into<String>>(to: T, message: U) -> Flash<Redirect>;
    fn danger_redirect<T: Into<String>, U: Into<String>>(to: T, message: U) -> Flash<Redirect>;
    fn extract_flash(flash: Option<FlashMessage<'_>>) -> Option<(String, String)>;
}
```

#### `AdminController`
For controllers requiring admin functionality:
```rust
pub trait AdminController: ControllerHelpers {
    async fn check_admin_auth(conn: Connection<'_, Db>, coordinator: &State<CoordinatorService>, jar: &CookieJar<'_>) -> Result<bool, Status>;
    async fn require_admin_auth(conn: Connection<'_, Db>, coordinator: &State<CoordinatorService>, jar: &CookieJar<'_>) -> Result<(), Status>;
}
```

#### `MountableController`
For controllers that can be mounted to Rocket:
```rust
pub trait MountableController {
    fn path(&self) -> &str;
    fn name(&self) -> &'static str;
}
```

### Controller Macros

#### `impl_controller!(ControllerType, "Name", routes![])`
Complete controller implementation:
```rust
impl_controller!(BlogController, "Blog Controller", routes![
    blog_index, blog_create, blog_edit
]);
// Generates Fairing implementation and trait implementations
```

#### `impl_admin_controller!(ControllerType, "Name", routes![])`
For admin controllers:
```rust
impl_admin_controller!(AdminController, "Admin Controller", routes![
    admin_dashboard, admin_users
]);
// Includes AdminController trait implementation
```

#### `create_controller_registry!(RegistryName, [(Controller, Path)])`
Automatically generates controller registries:
```rust
create_controller_registry!(
    BlogControllerRegistry,
    [
        (AuthController, "/auth"),
        (BlogController, "/blog"),
        (CommentController, "/comment"),
        (IndexController, "/"),
    ]
);
```

## Helper Macros

### Authentication Macros

#### `auth_required!(jar)`
Enforces authentication:
```rust
#[post("/protected")]
async fn protected_endpoint(jar: &CookieJar<'_>) -> Result<String, Status> {
    let token = auth_required!(jar);
    // Endpoint logic with authenticated user
    Ok("Protected content".to_string())
}
```

#### `admin_required!(conn, coordinator, jar)`
Enforces admin authentication:
```rust
#[post("/admin")]
async fn admin_endpoint(
    conn: Connection<'_, Db>,
    coordinator: &State<CoordinatorService>,
    jar: &CookieJar<'_>
) -> Result<String, Status> {
    admin_required!(conn, coordinator, jar);
    // Admin-only logic
    Ok("Admin content".to_string())
}
```

## Usage Examples

### Service Implementation
```rust
// Old way
pub struct MyService {
    base: BaseService,
}

impl MyService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }
}

// New way
pub struct MyService {
    base: BaseService,
}

impl_service_with_base!(MyService);
```

### Controller Implementation
```rust
// Old way
pub struct MyController {
    base: ControllerBase,
}

#[rocket::async_trait]
impl Fairing for MyController {
    fn info(&self) -> Info {
        Info {
            name: "My Controller",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket.mount(self.base.path(), routes![my_route]))
    }
}

// New way
pub struct MyController {
    base: ControllerBase,
}

impl_controller!(MyController, "My Controller", routes![my_route]);
```

### Registry Usage
```rust
// Old way - Manual registration
rocket
    .manage(AuthService::new())
    .manage(BlogService::new())
    .manage(CommentService::new())
    // ... many more services

// New way - Macro-generated registry
create_service_registry!(AppServiceRegistry, [
    AuthService,
    BlogService,
    CommentService,
]);

rocket = AppServiceRegistry::attach_all_services(rocket);
// or
rocket.attach(AppServiceRegistry::fairing());
```

## Benefits

1. **Reduced Boilerplate**: Eliminates repetitive code patterns
2. **Consistency**: Ensures all services/controllers follow the same patterns
3. **Maintainability**: Changes to patterns only need to be made in one place
4. **Type Safety**: Leverages Rust's type system for compile-time guarantees
5. **Composability**: Traits can be mixed and matched as needed
6. **Documentation**: Self-documenting through trait interfaces

## Migration Path

1. Existing code continues to work unchanged (backward compatibility)
2. New services/controllers can use the enhanced macros immediately
3. Existing services can be gradually migrated to use the new patterns
4. The `base` field usage can be gradually removed as functionality moves to traits

## Future Enhancements

- Proc macros for more complex code generation
- Derive macros for common service patterns
- Additional helper traits for specific domains
- Integration with error handling and logging patterns