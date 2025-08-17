use crate::{
    config::AppConfig,
    controllers::base::ControllerBase,
    dto::post::FormDTO,
    pool::Db,
    services::{AuthService, BlogService, CommentService, ReactionService, TagService, CoordinatorService},
    types::{HttpRange, StreamedFile},
};
use models::{dto::SearchFormDTO, post_reaction::ReactionType, tag};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::{CookieJar, Status},
    request::{FlashMessage, FromRequest, Outcome, Request},
    response::{Flash, Redirect},
    serde::json::{json, Json, Value},
    Build, Rocket, Route, State,
};
use rocket_dyn_templates::{context, Template};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm_rocket::Connection;
use std::fs::File;
use uuid::Uuid;

/// Request guard to extract client IP address
pub struct ClientIp(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientIp {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Check X-Forwarded-For header (from reverse proxies)
        if let Some(forwarded) = req.headers().get_one("X-Forwarded-For") {
            // X-Forwarded-For can contain multiple IPs, the first is the original client
            if let Some(client_ip) = forwarded.split(',').next() {
                let client_ip = client_ip.trim();
                if !client_ip.is_empty() && client_ip != "unknown" {
                    return Outcome::Success(ClientIp(client_ip.to_string()));
                }
            }
        }
        
        // Check X-Real-IP header (from nginx/other proxies)
        if let Some(real_ip) = req.headers().get_one("X-Real-IP") {
            let real_ip = real_ip.trim();
            if !real_ip.is_empty() && real_ip != "unknown" {
                return Outcome::Success(ClientIp(real_ip.to_string()));
            }
        }
        
        // Check CF-Connecting-IP (from Cloudflare)
        if let Some(cf_ip) = req.headers().get_one("CF-Connecting-IP") {
            let cf_ip = cf_ip.trim();
            if !cf_ip.is_empty() {
                return Outcome::Success(ClientIp(cf_ip.to_string()));
            }
        }
        
        // Fall back to connection's remote address
        if let Some(remote_addr) = req.remote() {
            return Outcome::Success(ClientIp(remote_addr.ip().to_string()));
        }
        
        // No IP could be determined - this is an error condition
        Outcome::Error((Status::BadRequest, ()))
    }
}

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
    coordinator: &State<CoordinatorService>,
    flash: Option<FlashMessage<'_>>,
    client_ip: ClientIp,
) -> Result<Template, Status> {
    let token = ControllerBase::check_auth(jar).unwrap_or_default();
    let db = conn.into_inner();
    
    // Use coordinator service to get all data for the list view
    let list_data = coordinator.get_blog_list_data(
        db,
        page,
        page_size,
        token.as_deref(),
        &client_ip.0,
    ).await.map_err(|_| Status::InternalServerError)?;
    
    Ok(Template::render(
        "blog/list",
        context! {
            posts: list_data.posts,
            page: list_data.page,
            page_size: list_data.page_size,
            num_pages: list_data.num_pages,
            token,
            all_tags: list_data.all_tags,
            reaction_summaries: list_data.reaction_summaries,
            has_accounts: list_data.has_accounts,
            flash: ControllerBase::extract_flash(flash)
        },
    ))
}
#[get("/<id>")]
async fn detail_view(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    auth_service: &State<AuthService>,
    comment_service: &State<CommentService>,
    reaction_service: &State<ReactionService>,
    tag_service: &State<TagService>,
    flash: Option<FlashMessage<'_>>,
    jar: &CookieJar<'_>,
    id: i32,
    client_ip: ClientIp,
) -> Result<Template, Status> {
    let token = ControllerBase::check_auth(jar).unwrap_or_default();
    let db = conn.into_inner();
    debug!("{}", id);

    // Check if user is admin to allow draft access
    let is_admin = if let Some(token_str) = &token {
        if let Ok(token_uuid) = Uuid::parse_str(token_str) {
            if let Some(account) = auth_service.check_token(db, token_uuid).await {
                account.admin
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    // Get min/max post range - handle errors gracefully
    let (min_post, max_post) = match service.find_mm_seq_id(db).await {
        Ok(Some((min, max))) => (min, max),
        Ok(None) => return Err(Status::NotFound),
        Err(_) => return Err(Status::InternalServerError),
    };

    if id < min_post {
        debug!("Less Than");
    }

    // Get post with account - handle errors gracefully
    let (post, account) = match service.find_by_seq_id_with_account(db, id).await {
        Ok(result) => result,
        Err(_) => return Err(Status::NotFound),
    };

    // Get tags for the post - handle errors gracefully
    let tags = match tag_service.find_tags_by_post_id(db, post.id).await {
        Ok(tags) => tags,
        Err(_) => return Err(Status::InternalServerError),
    };

    debug!("{:?}", tags);

    // Check if post is draft and user is not admin
    if post.draft.unwrap_or(false) && !is_admin {
        return Err(Status::NotFound);
    }

    // Get comments for the post - handle errors gracefully
    let comments = match comment_service.find_many_by_post_id(db, post.id).await {
        Ok(comments) => comments,
        Err(_) => return Err(Status::InternalServerError),
    };

    // Get reaction data for the post
    let reaction_summary = reaction_service
        .get_post_reaction_summary(db, post.id, &client_ip.0)
        .await
        .unwrap_or_else(|_| {
            // Default empty reaction summary on error
            crate::services::PostReactionSummary {
                post_id: post.id,
                total_reactions: 0,
                user_reaction: None,
                reactions: vec![],
            }
        });

    // Prepare reaction types with emoji data for template
    let reaction_types_with_data: Vec<Value> = ReactionType::all()
        .into_iter()
        .map(|rt| {
            json!({
                "type": rt.as_str(),
                "emoji": rt.emoji(),
                "title": format!("{:?}", rt)
            })
        })
        .collect();

    Ok(Template::render(
        "blog/detail",
        context! {
            post,
            tags,
            comments,
            username: account.map(|a| a.username),
            token,
            min_post,
            max_post,
            reaction_summary,
            reaction_types: reaction_types_with_data,
            flash: ControllerBase::extract_flash(flash)
        },
    ))
}

#[get("/search?<query>&<page>&<page_size>")]
async fn search_get(
    conn: Connection<'_, Db>,
    coordinator: &State<CoordinatorService>,
    query: Option<String>,
    page: Option<u64>,
    page_size: Option<u64>,
    jar: &CookieJar<'_>,
    client_ip: ClientIp,
) -> Result<Template, Status> {
    let token = ControllerBase::check_auth(jar).unwrap_or_default();
    let db = conn.into_inner();
    
    let search_query = query.unwrap_or_default();
    
    // Use coordinator service to get search results
    let search_data = coordinator.search_blog_posts(
        db,
        &search_query,
        page,
        page_size,
        token.as_deref(),
        &client_ip.0,
    ).await.map_err(|_| Status::InternalServerError)?;

    Ok(Template::render(
        "blog/search",
        context! {
            results: search_data.results,
            search_query: search_query.clone(),
            page: search_data.page,
            page_size: search_data.page_size,
            num_pages: search_data.num_pages,
            token,
            all_tags: search_data.all_tags,
            reaction_summaries: search_data.reaction_summaries,
            title: if search_query.trim().is_empty() { 
                "Search Posts".to_string() 
            } else { 
                format!("Search results for '{}'", search_query) 
            }
        },
    ))
}

#[post("/search", data = "<search_form>")]
async fn search_post(
    search_form: Form<SearchFormDTO>,
) -> Redirect {
    let query = &search_form.query;
    if query.trim().is_empty() {
        Redirect::to(uri!("/blog/search"))
    } else {
        // Simple URL encoding for the query parameter
        let encoded = query.replace(' ', "%20").replace('&', "%26").replace('=', "%3D");
        Redirect::to(format!("/blog/search?query={}", encoded))
    }
}

#[get("/<id>/video")]
async fn video(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    range: Option<HttpRange>,
    id: i32,
) -> Result<StreamedFile, Status> {
    let db = conn.into_inner();
    let post = match service.find_by_seq_id(db, id).await {
        Ok(post) => post,
        Err(_) => return Err(Status::NotFound),
    };
    if let Some(path) = post.path.to_owned() {
        let file = File::open(&path).map_err(|e| {
            println!("Error opening file: {}", e);
            Status::NotFound
        })?;
        let size = file
            .metadata()
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

        Ok(StreamedFile::new(&path, final_range).map_err(|e| {
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
    tag_service: &State<TagService>,
    app_config: &State<AppConfig>,
    jar: &CookieJar<'_>,
    form_data: Form<FormDTO<'_>>,
) -> Result<Flash<Redirect>, Status> {
    if let Some(token) = ControllerBase::check_auth(jar)? {
        let db = conn.into_inner();
        let token = match Uuid::parse_str(&token) {
            Ok(uuid) => uuid,
            Err(_) => return Err(Status::BadRequest),
        };
        if let Some(account) = auth_service.check_token(db, token).await {
            if !account.admin {
                return Err(Status::Unauthorized);
            }
            let mut form = form_data.into_inner();
            let post = match service.create(db, app_config, account.id, &mut form).await {
                Ok(post) => post,
                Err(_) => return Err(Status::InternalServerError),
            };
            debug!("{:?}", form);
            if let Some(tags_str) = form.tags {
                for tag_name in tags_str.split(",") {
                    debug!("{:?}", tag_name);
                    let tag_name = tag_name.trim();
                    if !tag_name.is_empty() {
                        let tag = match tag_service.find_or_create_tag(db, tag_name).await {
                            Ok(tag) => tag,
                            Err(_) => return Err(Status::InternalServerError),
                        };
                        match tag_service.add_tag_to_post(db, post.id, tag.id).await {
                            Ok(_) => {}
                            Err(_) => return Err(Status::InternalServerError),
                        };
                    }
                }
            }
            return Ok(ControllerBase::success_redirect(
                format!("/blog/{}", post.seq_id),
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
    tag_service: &State<TagService>,
    id: i32,
) -> Result<Template, Status> {
    ControllerBase::require_auth(jar)?;
    let db = conn.into_inner();
    let post = match service.find_by_seq_id(db, id).await {
        Ok(post) => post,
        Err(_) => return Err(Status::NotFound),
    };
    
    // Get tags for the post
    let tags = match tag_service.find_tags_by_post_id(db, post.id).await {
        Ok(tags) => tags,
        Err(_) => return Err(Status::InternalServerError),
    };
    
    Ok(Template::render(
        "blog/edit",
        context! {
            post,
            tags,
            form_url: ""
        },
    ))
}
#[post("/<id>/edit", data = "<form_data>")]
async fn edit(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    tag_service: &State<TagService>,
    id: i32,
    form_data: Form<FormDTO<'_>>,
    jar: &CookieJar<'_>,
) -> Result<Flash<Redirect>, Status> {
    ControllerBase::require_auth(jar)?;
    let db = conn.into_inner();
    
    // First get the post to get its ID
    let post = match service.find_by_seq_id(db, id).await {
        Ok(post) => post,
        Err(_) => return Err(Status::NotFound),
    };
    
    let form = form_data.into_inner();
    let tags_str = form.tags.clone();
    
    // Update the post
    match service
        .update_by_seq_id(db, id, form)
        .await
    {
        Ok(_) => {
            // Handle tag updates
            if let Some(tags_str) = tags_str {
                // Remove all existing tags for this post
                match tag_service.find_tags_by_post_id(db, post.id).await {
                    Ok(existing_tags) => {
                        for existing_tag in existing_tags {
                            let _ = tag_service.remove_tag_from_post(db, post.id, existing_tag.id).await;
                        }
                    },
                    Err(_) => {}, // Continue even if we can't fetch existing tags
                }
                
                // Add new tags
                for tag_name in tags_str.split(',') {
                    let tag_name = tag_name.trim();
                    if !tag_name.is_empty() {
                        match tag_service.find_or_create_tag(db, tag_name).await {
                            Ok(tag) => {
                                let _ = tag_service.add_tag_to_post(db, post.id, tag.id).await;
                            },
                            Err(_) => {}, // Continue even if tag creation fails
                        }
                    }
                }
            }
            
            Ok(ControllerBase::success_redirect(
                format!("/blog/{id}"),
                "Updated Post",
            ))
        },
        Err(_) => Err(Status::NotFound),
    }
}

#[get("/tag/<slug>?<page>&<page_size>")]
async fn posts_by_tag(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    auth_service: &State<AuthService>,
    reaction_service: &State<ReactionService>,
    tag_service: &State<TagService>,
    slug: String,
    page: Option<u64>,
    page_size: Option<u64>,
    jar: &CookieJar<'_>,
    client_ip: ClientIp,
) -> Result<Template, Status> {
    let token = ControllerBase::check_auth(jar).unwrap_or_default();
    let db = conn.into_inner();
    
    // Check if user is admin to include drafts
    let is_admin = if let Some(token_str) = &token {
        if let Ok(token_uuid) = Uuid::parse_str(token_str) {
            if let Some(account) = auth_service.check_token(db, token_uuid).await {
                account.admin
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    
    // Find tag by slug
    let tag = tag::Entity::find()
        .filter(tag::Column::Slug.eq(slug))
        .one(db)
        .await
        .map_err(|_| Status::NotFound)?
        .ok_or(Status::NotFound)?;
    
    // Get posts with this tag
    let (posts, page, page_size, num_pages) = service
        .paginate_posts_by_tag_include_drafts(db, tag.id, page, page_size, is_admin)
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Get all tags for the tag cloud
    let all_tags = match tag_service.find_all_tags(db).await {
        Ok(tags) => tags,
        Err(_) => vec![], // Continue even if tag loading fails
    };

    // Get reaction summaries for all posts
    let post_ids: Vec<Uuid> = posts.iter().map(|p| p.id).collect();
    let reaction_summaries = reaction_service
        .get_posts_reaction_summaries(db, &post_ids, &client_ip.0)
        .await
        .unwrap_or_default();

    Ok(Template::render(
        "blog/list",
        context! {
            posts,
            page,
            page_size,
            num_pages,
            token,
            all_tags,
            reaction_summaries,
            tag_filter: tag.name.clone(),
            title: format!("Posts tagged with '{}'", tag.name)
        },
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

#[get("/<id>/publish")]
async fn publish(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
    auth_service: &State<AuthService>,
    id: i32,
    jar: &CookieJar<'_>,
) -> Result<Flash<Redirect>, Status> {
    if let Some(token) = ControllerBase::check_auth(jar)? {
        let db = conn.into_inner();
        let token = match Uuid::parse_str(&token) {
            Ok(uuid) => uuid,
            Err(_) => return Err(Status::BadRequest),
        };
        if let Some(account) = auth_service.check_token(db, token).await {
            if !account.admin {
                return Err(Status::Unauthorized);
            }
            match service.publish_by_seq_id(db, id).await {
                Ok(_) => {
                    return Ok(ControllerBase::success_redirect(
                        format!("/blog/{}", id),
                        "Post published successfully",
                    ));
                }
                Err(_) => {
                    return Ok(ControllerBase::danger_redirect(
                        format!("/blog/{}", id),
                        "Failed to publish post",
                    ));
                }
            }
        }
    }
    Err(Status::Unauthorized)
}

#[post("/<id>/react/<reaction_type>")]
async fn add_reaction(
    conn: Connection<'_, Db>,
    reaction_service: &State<ReactionService>,
    blog_service: &State<BlogService>,
    id: i32,
    reaction_type: String,
    client_ip: ClientIp,
) -> Result<Json<Value>, Status> {
    let db = conn.into_inner();
    
    // Get post by seq_id to get the actual UUID
    let post = match blog_service.find_by_seq_id(db, id).await {
        Ok(post) => post,
        Err(_) => return Err(Status::NotFound),
    };

    // Validate reaction type
    if ReactionType::from_str(&reaction_type).is_none() {
        return Err(Status::BadRequest);
    }
    
    match reaction_service
        .add_reaction(db, post.id, &reaction_type, &client_ip.0, None)
        .await
    {
        Ok(_) => {
            // Get updated reaction summary
            let summary = reaction_service
                .get_post_reaction_summary(db, post.id, &client_ip.0)
                .await
                .map_err(|_| Status::InternalServerError)?;
            
            Ok(Json(json!({
                "success": true,
                "summary": summary
            })))
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[delete("/<id>/react")]
async fn remove_reaction(
    conn: Connection<'_, Db>,
    reaction_service: &State<ReactionService>,
    blog_service: &State<BlogService>,
    id: i32,
    client_ip: ClientIp,
) -> Result<Json<Value>, Status> {
    let db = conn.into_inner();
    
    // Get post by seq_id to get the actual UUID
    let post = match blog_service.find_by_seq_id(db, id).await {
        Ok(post) => post,
        Err(_) => return Err(Status::NotFound),
    };

    match reaction_service
        .remove_reaction(db, post.id, &client_ip.0)
        .await
    {
        Ok(_) => {
            // Get updated reaction summary
            let summary = reaction_service
                .get_post_reaction_summary(db, post.id, &client_ip.0)
                .await
                .map_err(|_| Status::InternalServerError)?;
            
            Ok(Json(json!({
                "success": true,
                "summary": summary
            })))
        }
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/<id>/reactions")]
async fn get_reactions(
    conn: Connection<'_, Db>,
    reaction_service: &State<ReactionService>,
    blog_service: &State<BlogService>,
    id: i32,
    client_ip: ClientIp,
) -> Result<Json<Value>, Status> {
    let db = conn.into_inner();
    
    // Get post by seq_id to get the actual UUID
    let post = match blog_service.find_by_seq_id(db, id).await {
        Ok(post) => post,
        Err(_) => return Err(Status::NotFound),
    };
    
    let summary = reaction_service
        .get_post_reaction_summary(db, post.id, &client_ip.0)
        .await
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Json(json!(summary)))
}

fn routes() -> Vec<Route> {
    routes![
        list_view,
        detail_view,
        create_view,
        edit_view,
        posts_by_tag,
        search_get,
        search_post,
        video,
        create,
        edit,
        delete,
        publish,
        add_reaction,
        remove_reaction,
        get_reactions
    ]
}

crate::impl_controller_fairing!(Controller, BlogService, "Blog Controller", routes());
