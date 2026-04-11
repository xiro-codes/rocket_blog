use models::dto::{UserRoleFormDTO, WorkTimeEntryFormDTO, TimeTrackingControlDTO, WorkTimeSummaryDTO, TipEntryFormDTO, WorkTimeEntryDisplayDTO, AccountFormDTO};
use sea_orm::EntityTrait;
use rocket::{
    get, post, routes, Build, Rocket, State,
    serde::json::{Json, Value},
    fairing::{self, Fairing, Kind, Info},
    http::{Cookie, CookieJar},
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use uuid::Uuid;
use serde_json::json;

use crate::{
    controllers::base::ControllerBase,
    guards::{AuthenticatedUser, OptionalUser},
    pool::Db,
    services::{WorkTimeService, SettingsService, AuthService},
};

/// Controller for work time tracking JSON API
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

#[get("/playground")]
async fn playground() -> Template {
    Template::render(
        "worktime/api_playground",
        context! {
            page_title: "API Playground",
        }
    )
}

#[post("/login", data = "<data>")]
async fn login(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    data: Json<AccountFormDTO>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    if let Ok(token) = service.login(db, data.into_inner()).await {
        let mut cookie = Cookie::new("token", token.to_string());
        cookie.set_path("/");
        cookies.add_private(cookie);
        Ok(Json(json!({"status": "success", "token": token.to_string()})))
    } else {
        Err(Json(json!({"error": "Invalid username or password"})))
    }
}

#[get("/stats")]
async fn api_stats(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Json<WorkTimeSummaryDTO> {
    let db = conn.into_inner();
    
    let summary = service.get_work_time_summary(db, user.account_id, None, None).await.unwrap_or_else(|_| {
        WorkTimeSummaryDTO {
            total_hours: 0.0,
            total_earnings: 0.0,
            currency: "USD".to_string(),
            entries_count: 0,
            current_shift_earnings: 0.0,
            pay_period_hours: 0.0,
        }
    });
    
    Json(summary)
}

#[get("/roles")]
async fn get_roles(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Result<Json<Vec<models::user_role::Model>>, Json<Value>> {
    let db = conn.into_inner();
    match service.get_user_roles(db, user.account_id).await {
        Ok(roles) => Ok(Json(roles)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/roles", data = "<data>")]
async fn create_role(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    data: Json<UserRoleFormDTO>,
) -> Result<Json<models::user_role::Model>, Json<Value>> {
    let db = conn.into_inner();
    match service.create_user_role(db, user.account_id, data.into_inner()).await {
        Ok(role) => Ok(Json(role)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/start", data = "<data>")]
async fn start_tracking(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    data: Json<TimeTrackingControlDTO>,
) -> Result<Json<models::work_time_entry::Model>, Json<Value>> {
    let db = conn.into_inner();
    match service.start_time_tracking(db, user.account_id, data.into_inner()).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/stop")]
async fn stop_tracking(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    match service.stop_time_tracking_with_role_info(db, user.account_id).await {
        Ok((entry, is_tipped)) => Ok(Json(json!({
            "entry": entry,
            "is_tipped": is_tipped
        }))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[get("/entries")]
async fn get_entries(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    settings_service: &State<SettingsService>,
) -> Result<Json<Vec<WorkTimeEntryDisplayDTO>>, Json<Value>> {
    let db = conn.into_inner();
    let user_timezone = settings_service.get_user_timezone(db, user.account_id).await
        .unwrap_or_else(|_| None)
        .unwrap_or_else(|| "UTC".to_string());
    
    match service.get_work_entries_for_display(db, user.account_id, &user_timezone, Some(50), None).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/entries", data = "<data>")]
async fn create_entry(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    data: Json<WorkTimeEntryFormDTO>,
) -> Result<Json<models::work_time_entry::Model>, Json<Value>> {
    let db = conn.into_inner();
    match service.create_manual_entry(db, user.account_id, data.into_inner()).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/entries/<entry_id>/tips", data = "<data>")]
async fn add_tips(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
    data: Json<TipEntryFormDTO>,
) -> Result<Json<models::work_time_entry::Model>, Json<Value>> {
    let db = conn.into_inner();
    match service.add_tips_to_entry(db, entry_id, user.account_id, data.into_inner()).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

fn routes() -> Vec<rocket::Route> {
    routes![
        playground,
        login,
        api_stats,
        get_roles,
        create_role,
        start_tracking,
        stop_tracking,
        get_entries,
        create_entry,
        add_tips,
    ]
}

crate::impl_controller_routes!(Controller, "Work Time JSON API Controller", routes());