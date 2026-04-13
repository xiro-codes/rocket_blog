use models::dto::{
    UserRoleFormDTO, WorkTimeEntryFormDTO, TimeTrackingControlDTO, 
    WorkTimeSummaryDTO, TipEntryFormDTO, WorkTimeEntryDisplayDTO, AccountFormDTO,
    NotificationSettingsFormDTO, TimezoneSettingsFormDTO, PayPeriodSettingsFormDTO,
    PayPeriodFormDTO,
};
use sea_orm::EntityTrait;
use rocket::{
    get, post, delete, routes, Build, Rocket, State,
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
    guards::{ApiUser, OptionalApiUser},
    pool::Db,
    services::{WorkTimeService, SettingsService, AuthService, PayPeriodService, TimezoneService},
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
    user: ApiUser,
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
    user: ApiUser,
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
    user: ApiUser,
    service: &State<WorkTimeService>,
    data: Json<UserRoleFormDTO>,
) -> Result<Json<models::user_role::Model>, Json<Value>> {
    let db = conn.into_inner();
    match service.create_user_role(db, user.account_id, data.into_inner()).await {
        Ok(role) => Ok(Json(role)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/roles/<role_id>/edit", data = "<data>")]
async fn edit_role(
    conn: Connection<'_, Db>,
    user: ApiUser,
    service: &State<WorkTimeService>,
    role_id: Uuid,
    data: Json<UserRoleFormDTO>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    match service.update_user_role(db, role_id, user.account_id, data.into_inner()).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[delete("/roles/<role_id>")]
async fn delete_role(
    conn: Connection<'_, Db>,
    user: ApiUser,
    service: &State<WorkTimeService>,
    role_id: Uuid,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    match service.delete_user_role(db, role_id, user.account_id).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/start", data = "<data>")]
async fn start_tracking(
    conn: Connection<'_, Db>,
    user: ApiUser,
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
    user: ApiUser,
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
    user: ApiUser,
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

#[get("/entries/<entry_id>")]
async fn get_entry(
    conn: Connection<'_, Db>,
    user: ApiUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
) -> Result<Json<models::work_time_entry::Model>, Json<Value>> {
    let db = conn.into_inner();
    match service.get_work_entry_by_id(db, entry_id, user.account_id).await {
        Ok(Some(entry)) => Ok(Json(entry)),
        Ok(None) => Err(Json(json!({"error": "Entry not found"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/entries", data = "<data>")]
async fn create_entry(
    conn: Connection<'_, Db>,
    user: ApiUser,
    service: &State<WorkTimeService>,
    data: Json<WorkTimeEntryFormDTO>,
) -> Result<Json<models::work_time_entry::Model>, Json<Value>> {
    let db = conn.into_inner();
    match service.create_manual_entry(db, user.account_id, data.into_inner()).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/entries/<entry_id>/edit", data = "<data>")]
async fn edit_entry(
    conn: Connection<'_, Db>,
    user: ApiUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
    data: Json<WorkTimeEntryFormDTO>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    match service.update_work_entry(db, entry_id, user.account_id, data.into_inner()).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[delete("/entries/<entry_id>")]
async fn delete_entry(
    conn: Connection<'_, Db>,
    user: ApiUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    match service.delete_work_entry(db, entry_id, user.account_id).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/entries/<entry_id>/tips", data = "<data>")]
async fn add_tips(
    conn: Connection<'_, Db>,
    user: ApiUser,
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

#[get("/notifications")]
async fn get_notifications(
    conn: Connection<'_, Db>,
    user: ApiUser,
    service: &State<WorkTimeService>,
) -> Result<Json<models::notification_settings::Model>, Json<Value>> {
    let db = conn.into_inner();
    match service.get_notification_settings(db, user.account_id).await {
        Ok(Some(settings)) => Ok(Json(settings)),
        Ok(None) => Err(Json(json!({"error": "Settings not found"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/notifications", data = "<data>")]
async fn update_notifications(
    conn: Connection<'_, Db>,
    user: ApiUser,
    service: &State<WorkTimeService>,
    data: Json<NotificationSettingsFormDTO>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    match service.create_or_update_notification_settings(db, user.account_id, data.into_inner()).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[get("/settings/timezone")]
async fn get_timezone(
    conn: Connection<'_, Db>,
    user: ApiUser,
    settings_service: &State<SettingsService>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    let timezone = settings_service.get_user_timezone(db, user.account_id).await
        .unwrap_or_else(|_| None)
        .unwrap_or_else(|| "UTC".to_string());
    Ok(Json(json!({"timezone": timezone})))
}

#[post("/settings/timezone", data = "<data>")]
async fn update_timezone(
    conn: Connection<'_, Db>,
    user: ApiUser,
    settings_service: &State<SettingsService>,
    data: Json<TimezoneSettingsFormDTO>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    let form_data = data.into_inner();
    
    if !TimezoneService::is_valid_timezone(&form_data.timezone) {
        return Err(Json(json!({"error": "Invalid timezone"})));
    }
    
    match settings_service.set_user_timezone(db, user.account_id, &form_data.timezone).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[get("/settings/payperiod")]
async fn get_payperiod_settings(
    conn: Connection<'_, Db>,
    user: ApiUser,
    settings_service: &State<SettingsService>,
) -> Result<Json<models::dto::PayPeriodSettingsDTO>, Json<Value>> {
    let db = conn.into_inner();
    match settings_service.get_user_pay_period_settings(db, user.account_id).await {
        Ok(Some(settings)) => Ok(Json(settings)),
        Ok(None) => Ok(Json(models::dto::PayPeriodSettingsDTO {
            start_day: "monday".to_string(),
            period_length: 2,
        })),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/settings/payperiod", data = "<data>")]
async fn update_payperiod_settings(
    conn: Connection<'_, Db>,
    user: ApiUser,
    settings_service: &State<SettingsService>,
    data: Json<PayPeriodSettingsFormDTO>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    let form_data = data.into_inner();
    
    let valid_days = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
    if !valid_days.contains(&form_data.start_day.as_str()) {
        return Err(Json(json!({"error": "Invalid start day"})));
    }
    
    if ![1, 2, 4].contains(&form_data.period_length) {
        return Err(Json(json!({"error": "Invalid period length"})));
    }
    
    match settings_service.set_user_pay_period_settings(db, user.account_id, &form_data).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[get("/payperiods")]
async fn get_payperiods(
    conn: Connection<'_, Db>,
    user: ApiUser,
    pay_period_service: &State<PayPeriodService>,
) -> Result<Json<Vec<models::dto::PayPeriodWithSummaryDTO>>, Json<Value>> {
    let db = conn.into_inner();
    match pay_period_service.get_pay_periods_with_summary(db, user.account_id).await {
        Ok(periods) => Ok(Json(periods)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/payperiods", data = "<data>")]
async fn create_payperiod(
    conn: Connection<'_, Db>,
    user: ApiUser,
    pay_period_service: &State<PayPeriodService>,
    data: Json<PayPeriodFormDTO>,
) -> Result<Json<models::pay_period::Model>, Json<Value>> {
    let db = conn.into_inner();
    match pay_period_service.create_pay_period(db, user.account_id, data.into_inner()).await {
        Ok(period) => Ok(Json(period)),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[delete("/payperiods/<period_id>")]
async fn delete_payperiod(
    conn: Connection<'_, Db>,
    user: ApiUser,
    pay_period_service: &State<PayPeriodService>,
    period_id: Uuid,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    match pay_period_service.delete_pay_period(db, period_id, user.account_id).await {
        Ok(_) => Ok(Json(json!({"status": "success"}))),
        Err(e) => Err(Json(json!({"error": e.to_string()})))
    }
}

#[post("/payperiods/assign")]
async fn auto_assign_entries(
    conn: Connection<'_, Db>,
    user: ApiUser,
    pay_period_service: &State<PayPeriodService>,
) -> Result<Json<Value>, Json<Value>> {
    let db = conn.into_inner();
    match pay_period_service.auto_assign_entries_to_pay_periods(db, user.account_id).await {
        Ok(count) => Ok(Json(json!({"status": "success", "assigned_count": count}))),
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
        edit_role,
        delete_role,
        start_tracking,
        stop_tracking,
        get_entries,
        get_entry,
        create_entry,
        edit_entry,
        delete_entry,
        add_tips,
        get_notifications,
        update_notifications,
        get_timezone,
        update_timezone,
        get_payperiod_settings,
        update_payperiod_settings,
        get_payperiods,
        create_payperiod,
        delete_payperiod,
        auto_assign_entries,
    ]
}

crate::impl_controller_routes!(Controller, "Work Time JSON API Controller", routes());