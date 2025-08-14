//! Data Transfer Objects (DTOs) for API boundaries.
//!
//! This module contains form data structures used for handling HTTP form submissions
//! and API requests. DTOs provide a clean boundary between external input and internal
//! data models.

/// Blog post related DTOs and form structures.
pub mod post {
    use rocket::FromForm;
    use rocket::fs::TempFile;

    /// Form DTO for creating or updating blog posts.
    ///
    /// This structure represents the form data submitted when creating or editing
    /// a blog post. It includes the post content and an optional file upload.
    /// 
    /// # Form Fields
    /// 
    /// * `title` - The title of the blog post
    /// * `text` - The main content of the post (supports Markdown)
    /// * `file` - Optional file attachment (e.g., images, videos)
    ///
    /// # Usage
    ///
    /// This DTO is used in blog post creation and editing endpoints to capture
    /// user input and validate form data before processing.
    #[derive(FromForm)]
    pub struct FormDTO<'r> {
        /// The title of the blog post
        pub title: String,
        /// The main content text (Markdown supported)
        pub text: String,
        /// Optional file upload for post attachment
        pub file: TempFile<'r>,
    }
}
