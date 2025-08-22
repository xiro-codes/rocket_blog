use models::dto::CommentFormDTO;
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::Status,
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
    http::CookieJar,
};
use sea_orm_rocket::Connection;
use uuid::Uuid;

use crate::{
    controllers::base::ControllerBase,
    pool::Db,
    services::{AuthService, BlogService, CommentService},
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
    auth_service: &State<AuthService>,
    blog_service: &State<BlogService>,
    jar: &CookieJar<'_>,
    post_id: Uuid,
    form_data: Form<CommentFormDTO>,
) -> Result<Flash<Redirect>, Status> {
    let db = conn.into_inner();
    
    // Check if user is authenticated
    let user_id = if let Some(token_cookie) = jar.get_private("token") {
        let token_str = token_cookie.value();
        if let Ok(token) = token_str.parse::<Uuid>() {
            auth_service.check_token(db, token).await.map(|account| account.id)
        } else {
            None
        }
    } else {
        None
    };
    
    let _ = service.create(db, post_id, form_data.into_inner(), user_id).await;
    match blog_service.find_by_id(db, post_id).await {
        Ok(Some(post)) => Ok(ControllerBase::success_redirect(
            format!("/blog/{}", post.seq_id),
            "Comment created",
        )),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/reply/<post_id>/<parent_id>", data = "<form_data>")]
async fn reply(
    conn: Connection<'_, Db>,
    service: &State<CommentService>,
    auth_service: &State<AuthService>,
    blog_service: &State<BlogService>,
    jar: &CookieJar<'_>,
    post_id: Uuid,
    parent_id: Uuid,
    form_data: Form<CommentFormDTO>,
) -> Result<Flash<Redirect>, Status> {
    let db = conn.into_inner();
    
    // Check if user is authenticated
    let user_id = if let Some(token_cookie) = jar.get_private("token") {
        let token_str = token_cookie.value();
        if let Ok(token) = token_str.parse::<Uuid>() {
            auth_service.check_token(db, token).await.map(|account| account.id)
        } else {
            None
        }
    } else {
        None
    };
    
    // Create a comment with parent_id set
    let mut reply_data = form_data.into_inner();
    reply_data.parent_id = Some(parent_id);
    
    let _ = service.create(db, post_id, reply_data, user_id).await;
    match blog_service.find_by_id(db, post_id).await {
        Ok(Some(post)) => Ok(ControllerBase::success_redirect(
            format!("/blog/{}", post.seq_id),
            "Reply created",
        )),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

pub fn routes() -> Vec<Route> {
    routes![create, reply]
}

crate::impl_controller_routes!(Controller, "Comment Controller", routes());
