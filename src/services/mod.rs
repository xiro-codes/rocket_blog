mod base;
pub use base::BaseService;

mod auth;
pub use auth::Service as AuthService;
mod blog;
pub use blog::Service as BlogService;
mod comment;
pub use comment::Service as CommentService;
