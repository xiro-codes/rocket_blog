use crate::{
    controllers::base::ControllerBase,
    dto::post::FormDTO,
    services,
    types::{HttpRange, StreamedFile},
};
use models::post;
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::{CookieJar, Status},
    request::FlashMessage,
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use uuid::Uuid;
use std::fs::File;
use crate::services::AuthService;
use crate::services::BlogService;
use crate::{pool::Db, services::CommentService};

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

#[get("/?<page>&<page_size>")]
async fn list_view(
    conn: Connection<'_, Db>,
    page: Option<u64>,
    page_size: Option<u64>,
    jar: &CookieJar<'_>,
    service: &State<BlogService>,
    flash: Option<FlashMessage<'_>>,
) -> Template {
    let token = ControllerBase::check_auth(jar).unwrap_or_default();
    let db = conn.into_inner();
    let (posts, page, page_size, num_pages) = service
        .paginate_with_title(db, page, page_size)
        .await
        .unwrap();

    Template::render(
        "blog/list",
        context! {
            posts,
            page,
            page_size,
            num_pages,
            token,
            flash: ControllerBase::extract_flash(flash)
        },
    )
}
#[get("/<id>")]
async fn detail_view(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    comment_service: &State<CommentService>,
    flash: Option<FlashMessage<'_>>,
    jar: &CookieJar<'_>,
    id: i32,
) -> Result<Template,Status> {
    let token = ControllerBase::check_auth(jar).unwrap_or_default();
    let db = conn.into_inner();
    let (post, account) = service.find_by_seq_id_with_account(db, id).await.unwrap();

    if post.draft.unwrap() && token.is_none(){
        return Err(Status::NotFound);
    }
    let comments = comment_service
        .find_many_by_post_id(db, post.id)
        .await
        .unwrap();
    let (min_post, max_post) = service.find_mm_seq_id(db).await.unwrap().unwrap();
    Ok(Template::render(
        "blog/detail",
        context! {
            post,
            comments,
            username: account.map(|a| a.username),
            token,
            min_post,
            max_post,
            flash: ControllerBase::extract_flash(flash)
        },
    ))
}
#[get("/<id>/video")]
async fn video(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    range: Option<HttpRange>,
    id: i32,
) -> Result<StreamedFile, Status> {
    let db = conn.into_inner();
    let post  = service.find_by_seq_id(db, id).await.unwrap();
    if let Some(path) = post.path.to_owned() {
        let file = File::open(&path)
            .map_err(|e| {
                println!("Error opening file: {}", e);
                Status::NotFound
            })?;
        let size = file.metadata()
            .map_err(|e| {
                println!("Error getting metadata: {}", e);
                Status::InternalServerError
            })?
            .len();

        let final_range = if let Some(HttpRange(mut range)) = range {
            // Handle different range types with actual file size validation
            if range.start == u64::MAX {
                // Suffix-byte-range-spec: bytes=-suffix
                let suffix_length = range.end;
                range.start = size.saturating_sub(suffix_length);
                range.end = size;
            } else if range.end == u64::MAX {
                // Open-ended range: bytes=start-
                if range.start >= size {
                    return Err(Status::RangeNotSatisfiable);
                }
                range.end = size;
            } else {
                // Standard range: bytes=start-end
                if range.start >= size || range.start >= range.end {
                    return Err(Status::RangeNotSatisfiable);
                }
                // Clamp end to file size if it exceeds
                if range.end > size {
                    range.end = size;
                }
            }
            Some(range)
        } else {
            None
        };

        Ok(StreamedFile::new(&path, final_range).map_err(|e|{
            println!("Error creating StreamedFile: {}", e);
            Status::InternalServerError
        })?)
    } else {
        Err(Status::NotFound)
    }
}

#[get("/create")]
async fn create_view(
    flash: Option<FlashMessage<'_>>,
    jar: &CookieJar<'_>,
) -> Result<Template, Status> {
    ControllerBase::require_auth(jar)?;
    Ok(Template::render(
        "blog/create",
        context! {
            form_url: "create",
            flash: ControllerBase::extract_flash(flash)
        },
    ))
}

#[post("/create", format = "multipart/form-data", data = "<form_data>")]
async fn create(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    auth_service: &State<AuthService>,
    jar: &CookieJar<'_>,
    form_data: Form<FormDTO<'_>>,
) -> Result<Flash<Redirect>, Status> {
    if let Some(token) = ControllerBase::check_auth(jar)? {
        let db = conn.into_inner();
        let token = Uuid::parse_str(&token).unwrap();
        if let Some(account) = auth_service.check_token(db, token).await {
            if !account.admin {
                return Err(Status::Unauthorized);
            }

            let id = service
                .create(db, account.id, &mut form_data.into_inner())
                .await
                .unwrap()
            .seq_id;

            return Ok(ControllerBase::success_redirect(
                format!("/blog/{id}"),
                "Post successfully added"
            ));
        }
    }
    Err(Status::Unauthorized)
}

#[get("/<id>/edit")]
async fn edit_view(
    jar: &CookieJar<'_>,
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    id: i32,
) -> Result<Template, Status> {
    ControllerBase::require_auth(jar)?;
    let db = conn.into_inner();
    let post = service.find_by_seq_id(db, id).await.unwrap();
    Ok(Template::render(
        "blog/edit",
        context! {
            post,
            form_url: ""
        },
    ))
}
#[post("/<id>/edit", data = "<form_data>")]
async fn edit(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    id: i32,
    form_data: Form<FormDTO<'_>>,
    jar: &CookieJar<'_>,
) -> Result<Flash<Redirect>, Status> {
    ControllerBase::require_auth(jar)?;
    let db = conn.into_inner();
    let _ = service
        .update_by_seq_id(db, id, form_data.into_inner())
        .await
        .expect("Post does not exist");
    Ok(ControllerBase::success_redirect(
        format!("/blog/{id}"),
        "Updated Post"
    ))
}
#[get("/<id>/delete")]
async fn delete(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    id: i32,
    jar: &CookieJar<'_>,
) -> Result<Flash<Redirect>, Status> {
    ControllerBase::require_auth(jar)?;
    let db = conn.into_inner();
    let _ = service.delete_by_seq_id(db, id).await;
    Ok(ControllerBase::success_redirect("/blog", "Deleted Post"))
}

fn routes() -> Vec<Route> {
    routes![
        list_view,
        detail_view,
        create_view,
        edit_view,
        video,
        create,
        edit,
        delete
    ]
}

crate::impl_controller_fairing!(Controller, BlogService, "Blog Controller", routes());
