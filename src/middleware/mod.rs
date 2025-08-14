//! Middleware components for cross-cutting concerns.
//!
//! This module contains Rocket fairings and middleware that handle
//! application-wide functionality such as database seeding and
//! other cross-cutting concerns.

mod seeding;
pub use seeding::Seeding;
