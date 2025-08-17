mod base;
pub use base::{BaseController, ControllerBase};

mod index;
pub use index::Controller as IndexController;
mod auth;
pub use auth::Controller as AuthController;
mod admin;
pub use admin::Controller as AdminController;
mod blog;
pub use blog::Controller as BlogController;
mod comment;
pub use comment::Controller as CommentController;
mod feed;
pub use feed::Controller as FeedController;
