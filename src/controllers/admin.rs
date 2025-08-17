use models::{dto::{SettingsFormDTO, GeneratePostFormDTO}};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::{Status},
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    controllers::base::ControllerBase,
    pool::Db,
    services::{AuthService, SettingsService, BlogService},
};

/// Admin Controller for admin-only features
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

#[get("/settings")]
async fn settings_view(
    conn: Connection<'_, Db>,
    auth_service: &State<AuthService>,
    settings_service: &State<SettingsService>,
    jar: &rocket::http::CookieJar<'_>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    // Check admin authentication
    if let Some(token) = ControllerBase::check_auth(jar)? {
        if let Ok(token_uuid) = uuid::Uuid::parse_str(&token) {
            if let Some(account) = auth_service.check_token(db, token_uuid).await {
                if !account.admin {
                    return Err(Status::Unauthorized);
                }
            } else {
                return Err(Status::Unauthorized);
            }
        } else {
            return Err(Status::Unauthorized);
        }
    } else {
        return Err(Status::Unauthorized);
    }
    
    let settings = settings_service.get_settings(db).await.map_err(|_| Status::InternalServerError)?;
    
    Ok(Template::render(
        "admin/settings",
        context! {
            settings: settings,
        }
    ))
}

#[post("/settings", data = "<data>")]
async fn update_settings(
    conn: Connection<'_, Db>,
    auth_service: &State<AuthService>,
    settings_service: &State<SettingsService>,
    jar: &rocket::http::CookieJar<'_>,
    data: Form<SettingsFormDTO>,
) -> Result<Flash<Redirect>, Status> {
    let db = conn.into_inner();
    
    // Check admin authentication
    if let Some(token) = ControllerBase::check_auth(jar)? {
        if let Ok(token_uuid) = uuid::Uuid::parse_str(&token) {
            if let Some(account) = auth_service.check_token(db, token_uuid).await {
                if !account.admin {
                    return Err(Status::Unauthorized);
                }
            } else {
                return Err(Status::Unauthorized);
            }
        } else {
            return Err(Status::Unauthorized);
        }
    } else {
        return Err(Status::Unauthorized);
    }

    match settings_service.create_or_update_settings(db, data.into_inner()).await {
        Ok(_) => Ok(ControllerBase::success_redirect("/admin/settings", "Settings updated successfully!")),
        Err(_) => Ok(ControllerBase::danger_redirect("/admin/settings", "Failed to update settings")),
    }
}

#[get("/generate")]
async fn generate_view(
    conn: Connection<'_, Db>,
    auth_service: &State<AuthService>,
    jar: &rocket::http::CookieJar<'_>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    // Check admin authentication
    if let Some(token) = ControllerBase::check_auth(jar)? {
        if let Ok(token_uuid) = uuid::Uuid::parse_str(&token) {
            if let Some(account) = auth_service.check_token(db, token_uuid).await {
                if !account.admin {
                    return Err(Status::Unauthorized);
                }
            } else {
                return Err(Status::Unauthorized);
            }
        } else {
            return Err(Status::Unauthorized);
        }
    } else {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render(
        "admin/generate",
        context! {}
    ))
}

#[post("/generate", data = "<data>")]
async fn generate_post(
    conn: Connection<'_, Db>,
    auth_service: &State<AuthService>,
    settings_service: &State<SettingsService>,
    jar: &rocket::http::CookieJar<'_>,
    data: Form<GeneratePostFormDTO>,
) -> Result<Flash<Redirect>, Status> {
    let db = conn.into_inner();
    
    // Check admin authentication
    if let Some(token) = ControllerBase::check_auth(jar)? {
        if let Ok(token_uuid) = uuid::Uuid::parse_str(&token) {
            if let Some(account) = auth_service.check_token(db, token_uuid).await {
                if !account.admin {
                    return Err(Status::Unauthorized);
                }
            } else {
                return Err(Status::Unauthorized);
            }
        } else {
            return Err(Status::Unauthorized);
        }
    } else {
        return Err(Status::Unauthorized);
    }
    
    // Get OpenAI configuration
    let config = settings_service.get_openai_config(db).await
        .map_err(|_| Status::InternalServerError)?;
    
    let (api_key, base_prompt) = match config {
        Some(config) => config,
        None => return Ok(ControllerBase::danger_redirect("/admin/generate", "OpenAI API key and base prompt not configured. Please configure them in settings first.")),
    };

    let form_data = data.into_inner();
    
    // Generate post using OpenAI
    match generate_post_content(&api_key, &base_prompt, &form_data.topic, form_data.style.as_deref()).await {
        Ok((title, content)) => {
            // Redirect to blog create page with generated content as query parameters
            let redirect_url = format!("/blog/create?generated_title={}&generated_content={}", 
                urlencoding::encode(&title), 
                urlencoding::encode(&content));
            Ok(ControllerBase::success_redirect(redirect_url, "Post generated successfully! Review and publish."))
        },
        Err(e) => {
            log::error!("Failed to generate post: {}", e);
            Ok(ControllerBase::danger_redirect("/admin/generate", "Failed to generate post. Please check your OpenAI configuration."))
        }
    }
}

async fn generate_post_content(
    api_key: &str,
    base_prompt: &str,
    topic: &str,
    style: Option<&str>,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let mut prompt = format!("{}\n\nTopic: {}", base_prompt, topic);
    if let Some(style) = style {
        prompt.push_str(&format!("\nStyle: {}", style));
    }
    prompt.push_str("\n\nPlease generate a blog post with a compelling title and well-structured content in markdown format. Return the response as JSON with 'title' and 'content' fields.");

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 2000,
        "temperature": 0.7
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("OpenAI API error: {}", error_text).into());
    }

    let response_json: Value = response.json().await?;
    
    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid response format")?;

    // Try to parse as JSON first, fallback to extracting title from content
    if let Ok(parsed) = serde_json::from_str::<Value>(content) {
        let title = parsed["title"].as_str().unwrap_or("Generated Post").to_string();
        let content = parsed["content"].as_str().unwrap_or(content).to_string();
        Ok((title, content))
    } else {
        // Fallback: extract title from first line or use default
        let lines: Vec<&str> = content.lines().collect();
        let title = if !lines.is_empty() && lines[0].starts_with('#') {
            lines[0].trim_start_matches('#').trim().to_string()
        } else {
            "Generated Post".to_string()
        };
        Ok((title, content.to_string()))
    }
}

fn routes() -> Vec<Route> {
    routes![settings_view, update_settings, generate_view, generate_post]
}

crate::impl_controller_fairing!(Controller, SettingsService, "Admin Controller", routes());