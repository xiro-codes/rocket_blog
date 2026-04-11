//! HTTP route controllers for handling web requests.
//!
//! This module contains all the HTTP route handlers organized by functionality.
//! Each controller handles a specific domain of the application such as
//! authentication, blog posts, comments, etc.
//!
//! ## Controller Architecture
//!
//! Controllers follow a consistent pattern:
//! - Each controller implements the `MountableController` trait
//! - Controllers use dependency injection through the service registry
//! - Route handlers are organized by HTTP method and functionality
//! - Controllers return appropriate HTTP responses and error handling
//!
//! ## Available Controllers
//!
//! - [`IndexController`] - Home page and root routes
//! - [`AuthController`] - User authentication and session management
//! - [`BlogController`] - Blog post creation, editing, and viewing
//! - [`CommentController`] - Comment creation and moderation
//! - [`FeedController`] - RSS feed generation
//! - [`SeoController`] - SEO and meta tag management
//! - [`WorkTimeController`] - Work time tracking functionality

mod base;
pub use base::{ControllerBase, ControllerHelpers, AdminController, MountableController};

mod index;
/// Main application index and home page controller
pub use index::Controller as IndexController;

mod auth;
/// Authentication and user session management controller
pub use auth::Controller as AuthController;

mod blog;
/// Blog post management and viewing controller
pub use blog::Controller as BlogController;

mod comment;
/// Comment system and moderation controller
pub use comment::Controller as CommentController;

mod feed;
/// RSS feed generation and syndication controller
pub use feed::Controller as FeedController;

mod seo;
/// SEO optimization and meta tag management controller
pub use seo::Controller as SeoController;

mod worktime;
/// Work time tracking and management controller
pub use worktime::Controller as WorkTimeController;

mod worktime_api;
/// Work time tracking JSON API controller
pub use worktime_api::Controller as WorkTimeApiController;

/// Work time authentication module
pub mod worktime_auth;
