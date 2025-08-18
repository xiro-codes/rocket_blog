use rocket::{
    form::Form,
    http::{CookieJar, Status},
    response::{Flash, Redirect},
    Route, State,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    controllers::base::ControllerBase,
    pool::Db,
    services::{AuthService, CoordinatorService},
};

pub struct Controller {
    base: ControllerBase,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            base: ControllerBase::new("/admin".to_string()),
        }
    }
}

#[derive(FromForm, Serialize, Deserialize)]
struct SettingsFormDTO {
    openai_api_key: String,
    openai_model: String,
    openai_max_tokens: String,
    openai_temperature: String,
}

#[derive(FromForm)]
struct GeneratePostFormDTO {
    topic: String,
}

#[get("/settings")]
async fn settings_view(
    conn: Connection<'_, Db>,
    coordinator: &State<CoordinatorService>,
    auth_service: &State<AuthService>,
    jar: &CookieJar<'_>,
) -> Result<Template, Status> {
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
            
            let settings = coordinator.get_all_settings(db).await
                .map_err(|_| Status::InternalServerError)?;
            
            // Convert settings to a map for easier template access
            let mut settings_map = std::collections::HashMap::new();
            for setting in settings {
                settings_map.insert(setting.key, setting.value.unwrap_or_default());
            }
            
            Ok(Template::render(
                "admin/settings",
                context! {
                    settings: settings_map,
                }
            ))
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
    }
}

#[post("/settings", data = "<data>")]
async fn update_settings(
    conn: Connection<'_, Db>,
    coordinator: &State<CoordinatorService>,
    auth_service: &State<AuthService>,
    jar: &CookieJar<'_>,
    data: Form<SettingsFormDTO>,
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
            
            let form = data.into_inner();
            let updates = vec![
                ("openai_api_key".to_string(), form.openai_api_key),
                ("openai_model".to_string(), form.openai_model),
                ("openai_max_tokens".to_string(), form.openai_max_tokens),
                ("openai_temperature".to_string(), form.openai_temperature),
            ];
            
            match coordinator.update_settings(db, updates).await {
                Ok(_) => Ok(ControllerBase::success_redirect("/admin/settings", "Settings updated successfully!")),
                Err(e) => {
                    log::error!("Failed to update settings: {}", e);
                    Ok(ControllerBase::danger_redirect("/admin/settings", "Failed to update settings"))
                }
            }
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
    }
}

#[get("/generate")]
async fn generate_post_view(
    conn: Connection<'_, Db>,
    coordinator: &State<CoordinatorService>,
    auth_service: &State<AuthService>,
    jar: &CookieJar<'_>,
) -> Result<Template, Status> {
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
            
            Ok(Template::render(
                "admin/generate_post",
                context! {}
            ))
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
    }
}

#[post("/generate", data = "<data>")]
async fn generate_post(
    conn: Connection<'_, Db>,
    coordinator: &State<CoordinatorService>,
    auth_service: &State<AuthService>,
    jar: &CookieJar<'_>,
    data: Form<GeneratePostFormDTO>,
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
            
            let form = data.into_inner();
            
            if form.topic.trim().is_empty() {
                return Ok(ControllerBase::danger_redirect("/admin/generate", "Please provide a topic"));
            }
            
            match coordinator.generate_blog_post(db, &form.topic).await {
                Ok(generated_post) => {
                    // Store the generated post in session/flash for the blog creation form
                    // For now, redirect to create post with success message
                    Ok(ControllerBase::success_redirect(
                        &format!("/blog/create?generated_title={}&generated_content={}&generated_excerpt={}", 
                            urlencoding::encode(&generated_post.title),
                            urlencoding::encode(&generated_post.content),
                            urlencoding::encode(&generated_post.excerpt)
                        ), 
                        "Post generated successfully! Review and publish below."
                    ))
                }
                Err(e) => {
                    log::error!("Failed to generate post: {}", e);
                    Ok(ControllerBase::danger_redirect("/admin/generate", &format!("Failed to generate post: {}", e)))
                }
            }
        } else {
            Err(Status::Unauthorized)
        }
    } else {
        Err(Status::Unauthorized)
    }
}

fn routes() -> Vec<Route> {
    routes![settings_view, update_settings, generate_post_view, generate_post]
}

crate::impl_controller_routes!(Controller, "Admin Controller", routes());