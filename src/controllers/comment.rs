use models::{account, comment, dto::CommentFormDTO};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::Status,
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
};
use sea_orm_rocket::Connection;
use uuid::Uuid;

use crate::{
    controllers::base::ControllerBase,
    pool::Db,
    services::{BlogService, CommentService},
};

pub struct Controller {
    base: ControllerBase,
}

impl Controller {
    pub fn new(path: String) -> Self {
        Self {
            base: ControllerBase::new(path),
        }
    }
}

#[post("/create/<post_id>", data = "<form_data>")]
async fn create(
    conn: Connection<'_, Db>,
    service: &State<CommentService>,
    blog_service: &State<BlogService>,
    post_id: Uuid,
    form_data: Form<CommentFormDTO>,
) -> Result<Flash<Redirect>, Status> {
    let db = conn.into_inner();
    let _ = service.create(db, post_id, form_data.into_inner()).await;
    match blog_service.find_by_id(db, post_id).await {
        Ok(Some(post)) => Ok(ControllerBase::success_redirect(
            format!("/blog/{}", post.seq_id),
            "Comment created",
        )),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

pub fn routes() -> Vec<Route> {
    routes![create]
}

crate::impl_controller_fairing!(Controller, CommentService, "Comment Controller", routes());
