//! Data Transfer Objects (DTOs) for API communication.
//!
//! This module contains form and API data structures used for communication
//! between the frontend and backend. DTOs provide validation and type safety
//! for incoming requests and outgoing responses.
//!
//! ## Structure
//!
//! DTOs are organized by domain:
//! - [`post`] - Blog post creation and editing forms
//! - Additional DTO modules for other features as needed
//!
//! ## Usage
//!
//! DTOs are typically used with Rocket's form handling:
//!
//! ```ignore
//! use rocket::post;
//! use app::dto::post::FormDTO;
//!
//! #[post("/posts", data = "<form>")]
//! async fn create_post(form: FormDTO<'_>) -> Result<String, Status> {
//!     // Handle form data...
//!     Ok("Post created".to_string())
//! }
//! ```

/// Blog post form data transfer objects
pub mod post {
    use rocket::{fs::TempFile, FromForm};

    /// Form data structure for creating and editing blog posts.
    ///
    /// This DTO handles all the form data needed for blog post operations,
    /// including content, metadata, file uploads, and AI generation options.
    ///
    /// # Fields
    ///
    /// * `title` - Post title (required)
    /// * `text` - Post content in Markdown format
    /// * `excerpt` - Optional custom excerpt/summary
    /// * `file` - Optional file upload (images, videos, etc.)
    /// * `youtube_url` - Optional YouTube URL for content embedding
    /// * `tags` - Comma-separated list of tags
    /// * `action` - Post action: "draft" or "publish"
    /// * `ai_generate` - AI generation type: "content", "excerpt", "tags"
    /// * `ai_prompt` - Additional prompt for AI content generation
    #[derive(FromForm, Debug)]
    pub struct FormDTO<'r> {
        pub title: String,
        pub text: String,
        pub excerpt: Option<String>,
        pub file: Option<TempFile<'r>>,
        pub youtube_url: Option<String>, // YouTube URL for downloading
        pub tags: Option<String>,
        pub action: Option<String>, // "draft" or "publish"
        pub ai_generate: Option<String>, // "content", "excerpt", "tags"
        pub ai_prompt: Option<String>, // Additional prompt for AI generation
    }
}
