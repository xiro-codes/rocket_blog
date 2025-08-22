mod base;
pub use base::ControllerBase;

mod index;
pub use index::Controller as IndexController;
mod auth;
pub use auth::Controller as AuthController;
mod blog;
pub use blog::Controller as BlogController;
mod comment;
pub use comment::Controller as CommentController;
mod feed;
pub use feed::Controller as FeedController;
mod settings;
pub use settings::Controller as SettingsController;
mod seo;
pub use seo::Controller as SeoController;
