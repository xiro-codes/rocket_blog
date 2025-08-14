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

pub struct Controller {
    path: String,
}

impl Controller {
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
