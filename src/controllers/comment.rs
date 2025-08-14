//! Comment controller for managing blog post comment endpoints.
//!
//! This controller handles HTTP requests related to comments on blog posts,
//! including creating new comments and managing comment-related operations.

use models::{account::FormDTO, comment};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::Status,
    response::{Redirect, Flash},
    Build, Rocket, Route, State,
};
use sea_orm_rocket::Connection;
use uuid::Uuid;

use crate::{pool::Db, services::{CommentService, BlogService}};

/// Comment controller for handling comment-related HTTP requests.
///
/// This controller manages comment operations including:
/// - Creating new comments on blog posts
/// - Validating comment form data
/// - Integration with blog post validation
/// - Providing user feedback through flash messages
pub struct Controller {
    /// The base path for comment routes (typically "/comment")
    path: String,
}

impl Controller {
    /// Creates a new CommentController instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The base path for comment routes
    ///
    /// # Returns
    ///
    /// A new Controller instance configured for comment endpoints.
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[post("/create/<post_id>", data = "<form_data>")]
async fn create(
    conn: Connection<'_, Db>,
    service: &State<CommentService>,
    blog_service: &State<BlogService>,
    post_id: Uuid,
    form_data: Form<comment::FormDTO>,
) -> Result<Flash<Redirect>, Status> {
    let db = conn.into_inner();
    let _ = service.create(db, post_id, form_data.into_inner()).await;
    let post = blog_service.find_by_id(db, post_id).await.unwrap().unwrap();
    Ok(Flash::success(
        Redirect::to(format!("/blog/{}", post.seq_id)),
        "Comment created",
    ))
}

pub fn routes() -> Vec<Route> {
    routes![create]
}

#[rocket::async_trait]
impl Fairing for Controller {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Comment Controller",
            kind: Kind::Ignite,
        }
    }
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket
            .manage(CommentService::new())
            .mount(self.path.to_owned(), routes()))
    }
}
