use models::dto::{SettingsFormDTO, OllamaSettingsFormDTO};
use rocket::{
    form::Form,
    response::{Flash, Redirect},
    Route, State,
    http::Status,
    fairing::{self, Fairing, Kind},
    Build, Rocket,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;

use crate::{
    controllers::base::ControllerBase,
    guards::admin::Admin,
    pool::Db,
    services::SettingsService,
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

#[get("/")]
async fn settings_view(
    conn: Connection<'_, Db>,
    _admin: Admin,
    service: &State<SettingsService>,
) -> Result<Template, Status> {
    log::info!("Route accessed: GET /settings/ - Settings page requested");
    let db = conn.into_inner();
    
    // Get current settings (non-sensitive values)
    let openai_configured = service
        .get_openai_api_key(db)
        .await
        .unwrap_or(None)
        .is_some();
    
    let ollama_url = service
        .get_ollama_url(db)
        .await
        .unwrap_or(Some("http://localhost:11434".to_string()))
        .unwrap_or_default();
    
    let ollama_model = service
        .get_ollama_model(db)
        .await
        .unwrap_or(Some("llama2".to_string()))
        .unwrap_or_default();
    
    let ollama_enabled = service
        .get_ollama_enabled(db)
        .await
        .unwrap_or(false);
    
    Ok(Template::render(
        "settings/index",
        context! {
            openai_configured,
            ollama_url,
            ollama_model,
            ollama_enabled,
        }
    ))
}

#[post("/openai", data = "<data>")]
async fn save_openai_settings(
    conn: Connection<'_, Db>,
    _admin: Admin,
    service: &State<SettingsService>,
    data: Form<SettingsFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /settings/openai - OpenAI settings save requested");
    let db = conn.into_inner();
    let form_data = data.into_inner();
    
    if form_data.openai_api_key.trim().is_empty() {
        return ControllerBase::danger_redirect("/settings/", "API key cannot be empty.");
    }
    
    // Test the API key first
    match SettingsService::test_openai_api_key(&form_data.openai_api_key).await {
        Ok(true) => {
            // API key is valid, save it
            match service.set_openai_api_key(db, &form_data.openai_api_key).await {
                Ok(_) => ControllerBase::success_redirect("/settings/", "OpenAI API key saved successfully."),
                Err(e) => ControllerBase::danger_redirect("/settings/", &format!("Failed to save API key: {}", e)),
            }
        }
        Ok(false) => {
            ControllerBase::danger_redirect("/settings/", "Invalid API key. Please check your key and try again.")
        }
        Err(e) => {
            ControllerBase::danger_redirect("/settings/", &format!("Failed to validate API key: {}", e))
        }
    }
}

#[delete("/openai")]
async fn delete_openai_settings(
    conn: Connection<'_, Db>,
    _admin: Admin,
    service: &State<SettingsService>,
) -> Flash<Redirect> {
    log::info!("Route accessed: DELETE /settings/openai - OpenAI settings deletion requested");
    let db = conn.into_inner();
    
    match service.delete_setting(db, "openai_api_key").await {
        Ok(_) => ControllerBase::success_redirect("/settings/", "OpenAI API key removed successfully."),
        Err(e) => ControllerBase::danger_redirect("/settings/", &format!("Failed to remove API key: {}", e)),
    }
}

#[post("/ollama", data = "<data>")]
async fn save_ollama_settings(
    conn: Connection<'_, Db>,
    _admin: Admin,
    service: &State<SettingsService>,
    data: Form<OllamaSettingsFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /settings/ollama - Ollama settings save requested");
    let db = conn.into_inner();
    let form_data = data.into_inner();
    
    if form_data.ollama_url.trim().is_empty() {
        return ControllerBase::danger_redirect("/settings/", "Ollama URL cannot be empty.");
    }
    
    // Test the connection first if enabled
    if form_data.ollama_enabled {
        match service.test_ollama_connection(&form_data.ollama_url).await {
            Ok(true) => {
                // Connection is valid
            }
            Ok(false) => {
                return ControllerBase::danger_redirect("/settings/", "Cannot connect to Ollama server. Please check the URL and ensure Ollama is running.");
            }
            Err(e) => {
                return ControllerBase::danger_redirect("/settings/", &format!("Failed to test Ollama connection: {}", e));
            }
        }
    }
    
    // Save all settings
    let mut errors = Vec::new();
    
    if let Err(e) = service.set_ollama_url(db, &form_data.ollama_url).await {
        errors.push(format!("URL: {}", e));
    }
    
    if let Err(e) = service.set_ollama_model(db, &form_data.ollama_model).await {
        errors.push(format!("Model: {}", e));
    }
    
    if let Err(e) = service.set_ollama_enabled(db, form_data.ollama_enabled).await {
        errors.push(format!("Enabled: {}", e));
    }
    
    if errors.is_empty() {
        ControllerBase::success_redirect("/settings/", "Ollama settings saved successfully.")
    } else {
        ControllerBase::danger_redirect("/settings/", &format!("Failed to save settings: {}", errors.join(", ")))
    }
}

#[delete("/ollama")]
async fn delete_ollama_settings(
    conn: Connection<'_, Db>,
    _admin: Admin,
    service: &State<SettingsService>,
) -> Flash<Redirect> {
    log::info!("Route accessed: DELETE /settings/ollama - Ollama settings deletion requested");
    let db = conn.into_inner();
    
    let mut errors = Vec::new();
    
    if let Err(e) = service.delete_setting(db, "ollama_url").await {
        errors.push(format!("URL: {}", e));
    }
    
    if let Err(e) = service.delete_setting(db, "ollama_model").await {
        errors.push(format!("Model: {}", e));
    }
    
    if let Err(e) = service.delete_setting(db, "ollama_enabled").await {
        errors.push(format!("Enabled: {}", e));
    }
    
    if errors.is_empty() {
        ControllerBase::success_redirect("/settings/", "Ollama settings removed successfully.")
    } else {
        ControllerBase::danger_redirect("/settings/", &format!("Failed to remove settings: {}", errors.join(", ")))
    }
}

fn routes() -> Vec<Route> {
    routes![settings_view, save_openai_settings, delete_openai_settings, save_ollama_settings, delete_ollama_settings]
}

crate::impl_controller_routes!(Controller, "Settings Controller", routes());