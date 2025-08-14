//! Blog service for managing blog posts and related operations.
//!
//! This service provides comprehensive blog post management including creation,
//! updating, deletion, retrieval, and pagination functionality. It handles
//! Markdown processing, file uploads, and database operations for blog posts.

use crate::dto::post::FormDTO;
use chrono::Local;
use models::prelude::{Account, Post};
use models::{account, post};
use sea_orm::ColumnTrait;
use sea_orm::*;
use uuid::Uuid;

/// Blog service for managing blog posts and content.
///
/// This service handles all blog post-related operations including:
/// - Creating new posts with Markdown processing
/// - Updating existing posts by ID or sequence ID
/// - Soft deletion (marking as draft) of posts
/// - Retrieving posts with optional account information
/// - Pagination for post listings
/// - File handling for post attachments
pub struct Service;

/// Default number of posts per page for pagination
const DEFAULT_PAGE_SIZE: u64 = 39;
/// Base directory for storing uploaded files
const DATA_PATH: &str = "/home/tod/.local/share/blog";

impl Service {
    /// Creates a new BlogService instance.
    ///
    /// # Returns
    ///
    /// A new Service instance ready for blog operations.
    pub fn new() -> Self {
        Self
    }

    /// Creates a new blog post with Markdown processing and file upload.
    ///
    /// This method:
    /// 1. Converts the post text from Markdown to HTML
    /// 2. Handles file upload and storage
    /// 3. Creates a new post entry in the database
    /// 4. Sets the post as draft by default
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `id` - User account ID creating the post
    /// * `data` - Form data containing post content and file upload
    ///
    /// # Returns
    ///
    /// * `Ok(post::Model)` - The created blog post
    /// * `Err(DbErr)` - Database or file operation error
    pub async fn create(
        &self,
        db: &DbConn,
        id: Uuid,
        data: &mut FormDTO<'_>,
    ) -> Result<post::Model, DbErr> {
        let text = markdown::to_html(data.text.as_str());
        let fid = Uuid::new_v4().to_string();
        let path = format!("{DATA_PATH}/{}_{}.webm", fid, data.file.name().unwrap());

        data.file
            .copy_to(path.clone())
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        post::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title.to_owned()),
            text: Set(text),
            path: Set(Some(path)),
            draft: Set(Some(true)),
            date_published: Set(Local::now().naive_local()),
            account_id: Set(id),
            ..Default::default()
        }
        .insert(db)
        .await
    }

    /// Updates an existing blog post by its UUID.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `id` - UUID of the post to update
    /// * `data` - Form data with updated content
    ///
    /// # Returns
    ///
    /// * `Ok(post::Model)` - The updated blog post
    /// * `Err(DbErr)` - Database error or post not found
    pub async fn update_by_id(
        &self,
        db: &DbConn,
        id: Uuid,
        data: FormDTO<'_>,
    ) -> Result<post::Model, DbErr> {
        let mut p: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", id)))
            .map(Into::into)?;
        p.title = Set(data.title.to_owned());
        p.text = Set(data.text.to_owned());
        p.update(db).await
    }

    /// Updates an existing blog post by its sequence ID.
    ///
    /// Sequence IDs are auto-incrementing integers used for user-friendly URLs.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `id` - Sequence ID of the post to update
    /// * `data` - Form data with updated content
    ///
    /// # Returns
    ///
    /// * `Ok(post::Model)` - The updated blog post
    /// * `Err(DbErr)` - Database error or post not found
    pub async fn update_by_seq_id(
        &self,
        db: &DbConn,
        id: i32,
        data: FormDTO<'_>,
    ) -> Result<post::Model, DbErr> {
        let mut p: post::ActiveModel = Post::find()
            .filter(post::Column::SeqId.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", id)))
            .map(Into::into)?;
        p.title = Set(data.title.to_owned());
        p.text = Set(data.text.to_owned());
        p.update(db).await
    }

    /// Permanently deletes a blog post by its UUID.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection  
    /// * `id` - UUID of the post to delete
    ///
    /// # Returns
    ///
    /// * `Ok(DeleteResult)` - Deletion result
    /// * `Err(DbErr)` - Database error
    ///
    /// # Note
    ///
    /// This method is currently not implemented and will panic.
    pub async fn delete_by_id(&self, db: &DbConn, id: Uuid) -> Result<DeleteResult, DbErr> {
        todo!()
    }

    /// Soft deletes a blog post by marking it as draft.
    ///
    /// This method doesn't permanently delete the post but marks it as a draft,
    /// effectively hiding it from public view while preserving the content.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `id` - Sequence ID of the post to soft delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(DbErr)` - Database error or post not found
    pub async fn delete_by_seq_id(&self, db: &DbConn, id: i32) -> Result<(), DbErr> {
        let mut p = self.find_by_seq_id(db, id).await?.into_active_model();
        p.draft = Set(Some(true));
        p.save(db).await.map(|_|())
    }

    /// Finds a blog post by its UUID.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `id` - UUID of the post to find
    ///
    /// # Returns
    ///
    /// * `Ok(Some(post::Model))` - The found blog post
    /// * `Ok(None)` - Post not found
    /// * `Err(DbErr)` - Database error
    pub async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<Option<post::Model>, DbErr> {
        Post::find_by_id(id).one(db).await
    }

    /// Finds a blog post by its sequence ID.
    ///
    /// Sequence IDs are user-friendly auto-incrementing integers used in URLs.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `id` - Sequence ID of the post to find
    ///
    /// # Returns
    ///
    /// * `Ok(post::Model)` - The found blog post
    /// * `Err(DbErr)` - Database error or post not found
    pub async fn find_by_seq_id(
        &self,
        db: &DbConn,
        id: i32,
    ) -> Result<post::Model , DbErr> {
        Post::find()
            .filter(post::Column::SeqId.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", id)))
            .map(Into::into)
    }

    /// Finds a blog post by sequence ID along with its author information.
    ///
    /// This method joins the post with the account table to retrieve both
    /// the post content and the author's account details.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `id` - Sequence ID of the post
    ///
    /// # Returns
    ///
    /// * `Ok((post::Model, Option<account::Model>))` - Post with optional author info
    /// * `Err(DbErr)` - Database error or post not found
    pub async fn find_by_seq_id_with_account(
        &self,
        db: &DbConn,
        id: i32,
    ) -> Result<(post::Model, Option<account::Model>), DbErr> {
        Post::find()
            .filter(post::Column::SeqId.eq(id))
            .find_also_related(Account)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", id)))
            .map(Into::into)
    }

    /// Retrieves all posts with only title information for efficient listing.
    ///
    /// This method returns a lightweight result containing only the essential
    /// fields needed for post listings (ID, title, sequence ID).
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<post::TitleResult>)` - List of posts with title information
    /// * `Err(DbErr)` - Database error
    pub async fn find_many_with_title(&self, db: &DbConn) -> Result<Vec<post::TitleResult>, DbErr> {
        Post::find()
            .select_only()
            .column(post::Column::Id)
            .column(post::Column::Title)
            .column(post::Column::SeqId)
            .into_partial_model()
            .all(db)
            .await
    }

    /// Gets the minimum and maximum sequence IDs in the database.
    ///
    /// This is useful for navigation and understanding the range of available posts.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    ///
    /// # Returns
    ///
    /// * `Ok(Some((min_id, max_id)))` - The range of sequence IDs
    /// * `Ok(None)` - No posts in database
    /// * `Err(DbErr)` - Database error
    pub async fn find_mm_seq_id(&self, db: &DbConn) -> Result<Option<(i32, i32)>, DbErr> {
        Post::find()
            .select_only()
            .column_as(post::Column::SeqId.min(), "min_post")
            .column_as(post::Column::SeqId.max(), "max_post")
            .into_tuple::<(i32, i32)>()
            .one(db)
            .await
    }

    /// Paginates blog posts with title information, excluding drafts.
    ///
    /// Returns a paginated list of published posts (non-draft) ordered by
    /// publication date in descending order (newest first).
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `page` - Page number (1-based, defaults to 1)
    /// * `page_size` - Number of posts per page (defaults to DEFAULT_PAGE_SIZE)
    ///
    /// # Returns
    ///
    /// * `Ok((posts, page, page_size, num_pages))` - Paginated results with metadata
    /// * `Err(DbErr)` - Database error or invalid page parameters
    ///
    /// # Errors
    ///
    /// Returns an error if page number or page size is zero.
    pub async fn paginate_with_title(
        &self,
        db: &DbConn,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<(Vec<post::TitleResult>, u64, u64, u64), DbErr> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page == 0 {
            return Err(DbErr::Custom("Page number cannot be zero".to_owned()));
        }
        if page_size == 0 {
            return Err(DbErr::Custom("Page size cannot be zero".to_owned()));
        }
        let paginator = Post::find()
            .select_only()
            .column(post::Column::Id)
            .column(post::Column::Title)
            .column(post::Column::SeqId)
            .filter(post::Column::Draft.eq(false))
            .order_by_desc(post::Column::DatePublished)
            .into_partial_model()
            .paginate(db, page_size);
        let num_pages = paginator.num_pages().await?;
        paginator
            .fetch_page(page - 1)
            .await
            .map(|p| (p, page, page_size, num_pages))
    }
}
