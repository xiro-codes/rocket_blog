//! # Rocket Blog
//!
//! A modern blog application built with Rust, featuring user authentication,
//! blog post management, and a commenting system. Built with the Rocket web framework
//! and SeaORM for database operations.
//!
//! ## Architecture
//!
//! The application follows a layered architecture:
//! - **Controllers**: Handle HTTP requests and responses
//! - **Services**: Business logic and data processing  
//! - **Models**: Database entities and relationships
//! - **Middleware**: Cross-cutting concerns like seeding and authentication
//!
//! ## Features
//!
//! - User authentication with secure password hashing
//! - Blog post CRUD operations with Markdown support
//! - Comment system for user engagement
//! - Admin functionality for content management
//! - Responsive web interface using Tera templates

#![allow(renamed_and_removed_lints)]
#[macro_use]
extern crate rocket;

mod controllers;
mod middleware;
mod dto;
mod types;
mod pool;
mod services;

use migrations::MigratorTrait;
use pool::Db;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::response::Redirect;
use rocket::{fairing, Build, Request, Rocket};
use rocket_dyn_templates::{context, Template};
use sea_orm::*;
use sea_orm_rocket::Database;

/// Runs database migrations on application startup.
///
/// This function is called as part of the application's fairing chain to ensure
/// the database schema is up-to-date before the application starts serving requests.
///
/// # Arguments
///
/// * `rocket` - The Rocket instance being built
///
/// # Returns
///
/// Returns a `fairing::Result` indicating success or failure of the migration process
async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migrations::Migrator::up(conn, None).await;
    Ok(rocket)
}

/// Default error catcher that redirects all unhandled errors to the home page.
///
/// This provides a user-friendly fallback for any routes that don't exist or 
/// result in errors, ensuring users always have a way to navigate back to the main site.
#[catch(default)]
pub fn catch_default() -> Redirect{
    Redirect::to("/")
}

/// Application entry point and Rocket instance configuration.
///
/// This function sets up the complete Rocket application with all necessary components:
///
/// - **Database**: SeaORM database connection pool
/// - **Templates**: Tera template engine for HTML rendering  
/// - **Migrations**: Automatic database schema migrations on startup
/// - **Seeding**: Development data seeding middleware
/// - **Controllers**: All HTTP route controllers for different features
/// - **Static Files**: CSS, JS, and asset serving
/// - **Error Handling**: Global error catchers
///
/// The application serves a blog platform with user authentication, post management,
/// and commenting functionality.
#[launch]
async fn rocket() -> _ {
    rocket::build()
        .register("/", catchers![catch_default])
        .attach(Db::init())
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .attach(middleware::Seeding::new(Some(0), 50))
        .attach(controllers::IndexController::new("/".to_owned()))
        .attach(controllers::AuthController::new("/auth".to_owned()))
        .attach(controllers::BlogController::new("/blog".to_owned()))
        .attach(controllers::CommentController::new("/comment".to_owned()))
        .mount("/static", FileServer::from("./static/"))
}
