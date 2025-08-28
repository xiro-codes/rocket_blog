# Shared Authentication Implementation

This document describes the implementation of shared authentication components that can be used across multiple applications in the rocket_blog workspace.

## Overview

The authentication system has been refactored to be shared between applications. This allows multiple apps in the workspace to use the same authentication logic, database schema, and user accounts.

## Components

### Shared Auth Service (`shared/common/src/auth/service.rs`)

The `AuthService` provides core authentication functionality:

- **Login**: User authentication with username/password
- **Token Management**: Session token creation and validation
- **Account Creation**: Both admin and regular user account creation
- **Admin Verification**: Check if tokens belong to admin users
- **Account Existence**: Check if any accounts exist in the system

### Shared Auth Controller (`shared/common/src/auth/controller.rs`)

The `AuthController` provides HTTP endpoints for authentication:

- `GET /auth/` - Login page
- `POST /auth/` - Login submission
- `GET /auth/logout` - User logout
- `GET /auth/register` - User registration page
- `POST /auth/register` - User registration submission
- `GET /auth/create-admin` - Admin account creation page (only when no accounts exist)
- `POST /auth/create-admin` - Admin account creation submission

### Configuration

The auth controller is configurable per application using `AuthControllerConfig`:

```rust
let auth_config = AuthControllerConfig::new(
    "/blog/".to_string(),  // redirect after login
    "/blog/".to_string(),  // redirect after logout
    "/auth/".to_string(),  // redirect after register
);
```

## Integration

### Blog Application

The blog application now uses the shared auth components:

1. **Service**: Uses `common::auth::AuthService` instead of local auth service
2. **Controller**: Uses `common::auth::AuthController` instead of local auth controller
3. **Configuration**: Redirects to `/blog/` after login/logout
4. **Templates**: Continues to use existing auth templates

### Hello World Application

The hello-world application has been enhanced with optional auth support:

1. **Database**: Now includes database connection for auth
2. **Service**: Manages shared `AuthService`
3. **Controller**: Mounts shared `AuthController` at `/auth`
4. **Configuration**: Redirects to `/` (hello-world home) after login/logout
5. **Templates**: Uses copied auth templates from blog app

## Benefits

1. **Code Reuse**: Authentication logic is written once and used by all apps
2. **Consistency**: All apps use the same auth behavior and database schema
3. **Shared Accounts**: Users can log into any app with the same credentials
4. **Maintainability**: Auth updates benefit all applications
5. **Scalability**: Easy to add auth to new applications

## Migration Impact

### For Existing Blog Users
- **No Breaking Changes**: Existing functionality remains unchanged
- **Same URLs**: Auth routes remain at `/auth/`
- **Same Database**: Uses same account table and schema

### For Developers
- **Simplified Auth**: New apps get auth support easily
- **Shared Infrastructure**: All apps use the same auth system
- **Configurable Redirects**: Each app can configure where users go after auth actions

## Usage in New Applications

To add auth support to a new application:

1. Add shared dependencies:
   ```toml
   common = { path = "../../shared/common" }
   models = { path = "../../shared/models" }
   ```

2. Configure the application:
   ```rust
   use common::auth::{AuthService, AuthController, AuthControllerConfig};
   
   let auth_config = AuthControllerConfig::new(
       "/".to_string(),       // redirect after login
       "/".to_string(),       // redirect after logout
       "/auth/".to_string(),  // redirect after register
   );
   
   rocket::build()
       .manage(AuthService::new())
       .manage(auth_config)
       .attach(AuthController::new("/auth".to_owned()))
   ```

3. Copy auth templates or create custom ones
4. Set up database connection for user account storage

## File Changes

### Added Files
- `shared/common/src/auth/mod.rs` - Auth module exports
- `shared/common/src/auth/service.rs` - Shared auth service
- `shared/common/src/auth/controller.rs` - Shared auth controller
- `shared/common/src/services/mod.rs` - Services module
- `shared/common/src/services/base.rs` - Shared base service

### Removed Files
- `apps/blog/src/services/auth.rs` - Local auth service (replaced with shared)
- `apps/blog/src/controllers/auth.rs` - Local auth controller (replaced with shared)
- `apps/blog/src/services/base.rs` - Local base service (replaced with shared)

### Modified Files
- `shared/common/Cargo.toml` - Added auth dependencies
- `shared/common/src/lib.rs` - Export auth and services modules
- `apps/blog/src/services/mod.rs` - Use shared auth and base services
- `apps/blog/src/controllers/mod.rs` - Use shared auth controller
- `apps/blog/src/registry.rs` - Configure shared auth components
- `apps/hello-world/src/main.rs` - Add auth support
- Template files copied to hello-world app

This implementation provides a solid foundation for shared authentication across the entire rocket_blog workspace while maintaining backward compatibility with existing functionality.