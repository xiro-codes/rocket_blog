use models::comment;
use rocket::{
    fairing::{Fairing, Kind, Info, Result as FairingResult},
    form::Form,
    http::Status,
    response::{Redirect, Flash},
    Build, Rocket, Route, State,
};
use sea_orm_rocket::Connection;
use uuid::Uuid;

use crate::{
    pool::Db, 
    services::{CommentService, BlogService},
    generic::controller,
};

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

fn routes() -> Vec<Route> {
    routes![create]
}

// Use the macro to generate the controller boilerplate
controller!(Controller, CommentService, "Comment Controller", routes());
