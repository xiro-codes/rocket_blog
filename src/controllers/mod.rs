mod index;
pub use index::Controller as IndexController;
mod auth;
pub use auth::Controller as AuthController;
mod blog;
pub use blog::Controller as BlogController;
mod comment;
pub use comment::Controller as CommentController;

