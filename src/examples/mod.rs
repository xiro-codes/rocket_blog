//! # CodeIgniter3-Style Query Builder Examples
//!
//! This module contains comprehensive examples and demonstrations of the
//! CodeIgniter3-style query builder proc macro implementation. These examples
//! showcase practical usage patterns, integration strategies, and best practices
//! for using the generated query builders in real applications.
//!
//! ## Module Organization
//!
//! - [`macro_demo`] - Basic macro usage and generated code examples
//! - [`query_builder_example`] - Practical query patterns and use cases
//! - [`typed_builder_demo`] - Type safety demonstrations and comparisons
//!
//! ## Getting Started
//!
//! To use the query builder in your own code:
//!
//! 1. Add the derive macro to your Sea-ORM entity:
//! ```rust
//! #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, QueryBuilder)]
//! #[sea_orm(table_name = "your_table")]
//! pub struct Model {
//!     // your fields...
//! }
//! ```
//!
//! 2. Use the generated query builder:
//! ```rust
//! let results = YourEntity::query()
//!     .where_eq(Column::Status, "active")
//!     .order_desc(Column::CreatedAt)
//!     .limit(10)
//!     .get(&db)
//!     .await?;
//! ```
//!
//! ## Benefits
//!
//! - **Familiar API**: CodeIgniter 3 developers feel at home
//! - **Type Safety**: Compile-time validation of all queries
//! - **Zero Cost**: No runtime overhead compared to native Sea-ORM
//! - **Integration**: Works alongside existing Sea-ORM code
//! - **Maintainability**: Changes to entities automatically update builders

pub mod macro_demo;
pub mod query_builder_example;
pub mod typed_builder_demo;