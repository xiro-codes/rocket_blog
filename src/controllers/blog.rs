//! Blog controller for managing blog post HTTP endpoints.
//!
//! This controller handles all blog-related HTTP requests including:
//! - Listing blog posts with pagination
//! - Viewing individual blog posts with comments
//! - Creating new blog posts (authenticated users only)
//! - Editing existing blog posts (authenticated users only)
//! - Deleting blog posts (authenticated users only)
//! - Serving media files associated with posts

use crate::{
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

/// Blog controller for handling blog post HTTP requests.
///
/// This controller manages the complete blog post lifecycle including:
/// - Public viewing of published posts
/// - Authenticated creation and editing of posts
/// - Pagination for post listings
/// - Integration with comment system
/// - Media file serving for post attachments
pub struct Controller {
    /// The base path for blog routes (typically "/blog")
    path: String,
}

impl Controller {
    /// Creates a new BlogController instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The base path for blog routes
    ///
    /// # Returns
    ///
    /// A new Controller instance configured for blog endpoints.
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

/// Displays a paginated list of blog posts.
///
/// This endpoint serves the main blog listing page with pagination support.
/// Only published posts (non-drafts) are shown to public users.
/// Authenticated users may see additional options or draft indicators.
///
/// # Query Parameters
///
/// * `page` - Page number (1-based, defaults to 1)
/// * `page_size` - Number of posts per page (defaults to service default)
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `page` - Optional page number
/// * `page_size` - Optional page size
/// * `jar` - Cookie jar for authentication status
/// * `service` - BlogService for post operations
/// * `flash` - Optional flash message for user feedback
///
/// # Returns
///
/// A rendered template with the blog post listing and pagination metadata.
#[get("/?<page>&<page_size>")]
async fn list_view(
    conn: Connection<'_, Db>,
    page: Option<u64>,
    page_size: Option<u64>,
    jar: &CookieJar<'_>,
    service: &State<BlogService>,
    flash: Option<FlashMessage<'_>>,
) -> Template {
    let token = jar.get_private("token").map(|c| c.value().to_owned());
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
            flash: flash.map(FlashMessage::into_inner)
        },
    )
}

/// Displays a detailed view of a single blog post with comments.
///
/// Shows the full content of a blog post along with all associated comments.
/// Draft posts are only visible to authenticated users. Public users will
/// receive a 404 error when trying to access draft posts.
///
/// # Path Parameters
///
/// * `id` - Sequence ID of the blog post to display
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `service` - BlogService for post retrieval
/// * `comment_service` - CommentService for loading comments
/// * `flash` - Optional flash message for user feedback
/// * `jar` - Cookie jar for authentication status
/// * `id` - Sequence ID of the post
///
/// # Returns
///
/// * `Ok(Template)` - Rendered post detail page with comments
/// * `Err(Status::NotFound)` - Post not found or draft post accessed by unauthenticated user
#[get("/<id>")]
async fn detail_view(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    comment_service: &State<CommentService>,
    flash: Option<FlashMessage<'_>>,
    jar: &CookieJar<'_>,
    id: i32,
) -> Result<Template,Status> {
    let token = jar.get_private("token").map(|c| c.value().to_owned());
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
            flash: flash.map(FlashMessage::into_inner)
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
    if let None = jar.get_private("token") {
        return Err(Status::Unauthorized);
    }
    Ok(Template::render(
        "blog/create",
        context! {
            form_url: "create",
            flash: flash.map(FlashMessage::into_inner)
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
    if let Some(token) = jar.get_private("token") {
        let db = conn.into_inner();
        let token = Uuid::parse_str(token.value()).unwrap();
        if let Some(account) = auth_service.check_token(db, token).await {
            if !account.admin {
                return Err(Status::Unauthorized);
            }

            let id = service
                .create(db, account.id, &mut form_data.into_inner())
                .await
                .unwrap()
            .seq_id;

            return Ok(Flash::success(
                Redirect::to(format!("/blog/{id}")),
                "Post successfully added",
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
    if let None = jar.get_private("token") {
        return Err(Status::Unauthorized);
    }
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
    if let None = jar.get_private("token") {
        return Err(Status::Unauthorized);
    }
    let db = conn.into_inner();
    let _ = service
        .update_by_seq_id(db, id, form_data.into_inner())
        .await
        .expect("Post does not exist");
    return Ok(Flash::success(
        Redirect::to(format!("/blog/{id}")),
        "Updated Post",
    ));
}
#[get("/<id>/delete")]
async fn delete(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    id: i32,
    jar: &CookieJar<'_>,
) -> Result<Flash<Redirect>, Status> {
    if let None = jar.get_private("token") {
        return Err(Status::Unauthorized);
    }
    let db = conn.into_inner();
    let _ = service.delete_by_seq_id(db, id).await;
    return Ok(Flash::success(
        Redirect::to(format!("/blog")),
        "Deleted Post",
    ));
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

#[rocket::async_trait]
impl Fairing for Controller {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Blog Controller",
            kind: Kind::Ignite,
        }
    }
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket
            .manage(BlogService::new())
            .mount(self.path.to_owned(), routes()))
    }
}
