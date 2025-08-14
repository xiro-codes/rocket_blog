//! Comment service for managing blog post comments.
//!
//! This service handles comment creation, retrieval, and management for blog posts.
//! Comments are linked to specific posts and ordered chronologically for display.

use std::fmt::format;

use chrono::Local;
use models::comment;
use models::prelude::Comment;
use sea_orm::*;
use uuid::Uuid;

/// Comment service for managing blog post comments.
///
/// This service provides functionality for:
/// - Creating new comments on blog posts
/// - Retrieving individual comments by ID
/// - Fetching all comments for a specific blog post
/// - Ordering comments by publication date
pub struct Service;

impl Service {
    /// Creates a new CommentService instance.
    ///
    /// # Returns
    ///
    /// A new Service instance ready for comment operations.
    pub fn new() -> Self {
        Self
    }

    /// Creates a new comment on a blog post.
    ///
    /// This method creates a new comment entry in the database with the current
    /// timestamp and links it to the specified blog post.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `post_id` - UUID of the blog post to comment on
    /// * `data` - Form data containing the comment text
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Comment created successfully
    /// * `Err(DbErr)` - Database error during creation
    pub async fn create(&self, db: &DbConn, post_id: Uuid, data: comment::FormDTO) -> Result<(), DbErr> {
        let comment = comment::ActiveModel {
            id: Set(Uuid::new_v4()),
            text: Set(data.text),
            date_published: Set(Local::now().naive_local()),
            post_id: Set(post_id),
        }.insert(db).await?;
        Ok(())
    }

    /// Finds a specific comment by its UUID.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `id` - UUID of the comment to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(comment::Model)` - The found comment
    /// * `Err(DbErr)` - Database error or comment not found
    pub async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<comment::Model, DbErr> {
        Comment::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Comment")))
    }

    /// Retrieves all comments for a specific blog post.
    ///
    /// Comments are returned ordered by publication date in descending order
    /// (newest comments first) to provide a chronological discussion flow.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `post_id` - UUID of the blog post
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<comment::Model>)` - List of comments for the post
    /// * `Err(DbErr)` - Database error
    pub async fn find_many_by_post_id(&self, db: &DbConn, post_id: Uuid) -> Result<Vec<comment::Model>, DbErr> {
        Comment::find()
            .filter(comment::Column::PostId.eq(post_id))
            .order_by_desc(comment::Column::DatePublished)
            .all(db)
            .await
    }
}
