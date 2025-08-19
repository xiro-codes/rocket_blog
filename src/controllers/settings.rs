use models::dto::SettingsFormDTO;
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
    let db = conn.into_inner();
    
    // Get current settings (non-sensitive values)
    let openai_configured = service
        .get_openai_api_key(db)
        .await
        .unwrap_or(None)
        .is_some();
    
    Ok(Template::render(
        "settings/index",
        context! {
            openai_configured,
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
    let db = conn.into_inner();
    
    match service.delete_setting(db, "openai_api_key").await {
        Ok(_) => ControllerBase::success_redirect("/settings/", "OpenAI API key removed successfully."),
        Err(e) => ControllerBase::danger_redirect("/settings/", &format!("Failed to remove API key: {}", e)),
    }
}

fn routes() -> Vec<Route> {
    routes![settings_view, save_openai_settings, delete_openai_settings]
}

crate::impl_controller_routes!(Controller, "Settings Controller", routes());