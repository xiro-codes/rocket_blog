use models::dto::{UserRoleFormDTO, WorkTimeEntryFormDTO, TimeTrackingControlDTO, WorkTimeSummaryDTO, NotificationSettingsFormDTO, PayPeriodFormDTO, AccountFormDTO, TimezoneSettingsFormDTO, PayPeriodSettingsFormDTO, TipEntryFormDTO};
use models::user_role;
use sea_orm::EntityTrait;
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    response::{Flash, Redirect},
    routes, Build, Rocket, State,
    http::{Cookie, CookieJar},
    serde::json::Json,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use uuid::Uuid;

use crate::{
    controllers::base::ControllerBase,
    guards::{AuthenticatedUser, OptionalUser},
    pool::Db,
    services::{WorkTimeService, PayPeriodService, AuthService, SettingsService, TimezoneService},
};

/// Controller for work time tracking functionality
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
async fn home(
    conn: Connection<'_, Db>,
    user: OptionalUser,
    service: &State<WorkTimeService>,
    auth_service: &State<AuthService>,
    settings_service: &State<SettingsService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET / - Work time home page");
    let db = conn.into_inner();
    
    // If user has a token, verify it and show dashboard
    if let Some(token) = user.token {
        if let Some(account) = auth_service.check_token(db, token).await {
            // User is authenticated, show dashboard
            match service.get_user_roles(db, account.id).await {
                Ok(roles) => {
                    // Get user timezone preference
                    let user_timezone = settings_service.get_user_timezone(db, account.id).await
                        .unwrap_or_else(|_| None)
                        .unwrap_or_else(|| "UTC".to_string());
                    
                    let active_entry = service.get_active_entry(db, account.id).await.unwrap_or(None);
                    let recent_entries = service.get_work_entries_for_display(db, account.id, &user_timezone, Some(10), None).await.unwrap_or_default();
                    let summary = service.get_work_time_summary(db, account.id, None, None).await.unwrap_or_else(|_| {
                        WorkTimeSummaryDTO {
                            total_hours: 0.0,
                            total_earnings: 0.0,
                            currency: "USD".to_string(),
                            entries_count: 0,
                            current_shift_earnings: 0.0,
                            pay_period_hours: 0.0,
                        }
                    });
                    
                    return Ok(Template::render(
                        "worktime/dashboard",
                        context! {
                            page_title: "Work Time Dashboard",
                            roles: roles,
                            active_entry: active_entry,
                            recent_entries: recent_entries,
                            summary: summary,
                            username: account.username,
                            user_timezone: user_timezone,
                        }
                    ));
                }
                Err(e) => {
                    log::error!("Failed to load work time dashboard: {}", e);
                    return Err(Flash::error(Redirect::to("/worklog/"), "Failed to load dashboard"));
                }
            }
        }
    }
    
    // User is not authenticated or invalid token, show login page
    // Check if any accounts exist, return 404 if none
    if !auth_service.has_any_accounts(db).await {
        log::info!("No accounts exist in system - worktime login page not available");
        return Err(Flash::error(Redirect::to("/worklog/"), "No accounts available"));
    }
    
    log::debug!("Worktime login page served successfully");
    Ok(Template::render(
        "worktime/login",
        context! {
            page_title: "Work Time Tracker Login",
        }
    ))
}

#[post("/login", data = "<form>")]
async fn login(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    form: Form<AccountFormDTO>,
    cookies: &CookieJar<'_>,
) -> Flash<Redirect> {
    let form_data = form.into_inner();
    log::info!("Route accessed: POST /login - Worktime login attempt for username: {}", form_data.username);
    
    let db = conn.into_inner();
    if let Ok(token) = service.login(db, form_data).await {
        let mut cookie = Cookie::new("token", token.to_string());
        cookie.set_path("/");
        cookies.add_private(cookie);
        log::info!("Worktime authentication successful - Redirecting to worktime dashboard");
        Flash::success(
            Redirect::to("/worklog/"),
            "Login successful! Welcome to your work time tracker.",
        )
    } else {
        log::warn!("Worktime authentication failed - Redirecting back to login form");
        Flash::error(
            Redirect::to("/worklog/"),
            "Invalid username or password. Please try again.",
        )
    }
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    log::info!("Route accessed: GET /logout - Worktime user logout requested");
    
    let mut cookie = Cookie::from("token");
    cookie.set_path("/");
    cookies.remove_private(cookie);
    
    log::debug!("Worktime user successfully logged out - Redirecting to worktime login");
    Flash::success(
        Redirect::to("/worklog/"),
        "Logout successful.",
    )
}

#[get("/register")]
async fn register_view() -> Template {
    log::info!("Route accessed: GET /register - Worktime user registration page requested");
    Template::render(
        "worktime/register",
        context! {
            page_title: "Create Work Time Tracker Account",
        }
    )
}

#[post("/register", data = "<form>")]
async fn register(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    form: Form<AccountFormDTO>,
) -> Flash<Redirect> {
    let form_data = form.into_inner();
    let username = form_data.username.clone();
    log::info!("Route accessed: POST /register - Worktime user account registration attempt for username: {}", username);
    
    let db = conn.into_inner();
    match service.create_user_account(db, form_data).await {
        Ok(account) => {
            log::info!("Worktime user account created successfully for username: {} - Redirecting to worktime login", account.username);
            Flash::success(
                Redirect::to("/worklog/"),
                "Account created successfully! You can now log in to access the work time tracker.",
            )
        }
        Err(e) => {
            log::warn!("Failed to create worktime user account for username: {} - Error: {}", username, e);
            Flash::error(
                Redirect::to("/worklog/register"),
                "Failed to create account. Username may already exist or be invalid.",
            )
        }
    }
}

#[get("/dashboard")]
async fn dashboard(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    settings_service: &State<SettingsService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /dashboard - Work time dashboard");
    let db = conn.into_inner();
    
    match service.get_user_roles(db, user.account_id).await {
        Ok(roles) => {
            // Get user timezone preference
            let user_timezone = settings_service.get_user_timezone(db, user.account_id).await
                .unwrap_or_else(|_| None)
                .unwrap_or_else(|| "UTC".to_string());
            
            let active_entry = service.get_active_entry(db, user.account_id).await.unwrap_or(None);
            let recent_entries = service.get_work_entries_for_display(db, user.account_id, &user_timezone, Some(10), None).await.unwrap_or_default();
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
            
            Ok(Template::render(
                "worktime/dashboard",
                context! {
                    page_title: "Work Time Dashboard",
                    roles: roles,
                    active_entry: active_entry,
                    recent_entries: recent_entries,
                    summary: summary,
                    username: user.username,
                    user_timezone: user_timezone,
                }
            ))
        }
        Err(e) => {
            log::error!("Failed to load work time dashboard: {}", e);
            Err(Flash::error(Redirect::to("/worklog/"), "Failed to load dashboard"))
        }
    }
}

#[get("/roles")]
async fn roles_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /roles - User roles management");
    let db = conn.into_inner();
    
    match service.get_user_roles(db, user.account_id).await {
        Ok(roles) => Ok(Template::render(
            "worktime/roles",
            context! {
                page_title: "Manage Roles",
                roles: roles,
                username: user.username,
            }
        )),
        Err(e) => {
            log::error!("Failed to load user roles: {}", e);
            Err(Flash::error(Redirect::to("/worklog/"), "Failed to load roles"))
        }
    }
}

#[post("/roles", data = "<form>")]
async fn create_role(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    form: Form<UserRoleFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /roles - Creating user role");
    let db = conn.into_inner();
    
    match service.create_user_role(db, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/roles"), "Role created successfully"),
        Err(e) => {
            log::error!("Failed to create user role: {}", e);
            Flash::error(Redirect::to("/worklog/roles"), "Failed to create role")
        }
    }
}

#[post("/roles/edit/<role_id>", data = "<form>")]
async fn edit_role(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    role_id: Uuid,
    form: Form<UserRoleFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /roles/edit/{} - Editing user role", role_id);
    let db = conn.into_inner();
    
    match service.update_user_role(db, role_id, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/roles"), "Role updated successfully"),
        Err(e) => {
            log::error!("Failed to update user role: {}", e);
            Flash::error(Redirect::to("/worklog/roles"), "Failed to update role")
        }
    }
}

#[post("/roles/delete/<role_id>")]
async fn delete_role(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    role_id: Uuid,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /roles/delete/{} - Deleting user role", role_id);
    let db = conn.into_inner();
    
    match service.delete_user_role(db, role_id, user.account_id).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/roles"), "Role deleted successfully"),
        Err(e) => {
            log::error!("Failed to delete user role: {}", e);
            Flash::error(Redirect::to("/worklog/roles"), "Failed to delete role")
        }
    }
}

#[post("/start", data = "<form>")]
async fn start_tracking(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    form: Form<TimeTrackingControlDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /start - Starting time tracking");
    let db = conn.into_inner();
    
    match service.start_time_tracking(db, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/"), "Time tracking started"),
        Err(e) => {
            log::error!("Failed to start time tracking: {}", e);
            Flash::error(Redirect::to("/worklog/"), &format!("Failed to start tracking: {}", e))
        }
    }
}

#[post("/stop")]
async fn stop_tracking(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /stop - Stopping time tracking");
    let db = conn.into_inner();
    
    match service.stop_time_tracking_with_role_info(db, user.account_id).await {
        Ok((entry, is_tipped)) => {
            if is_tipped {
                // For tipped roles, redirect to a tips entry page
                Flash::success(Redirect::to(format!("/entries/{}/tips", entry.id)), 
                              "Shift ended! Please enter your tips.")
            } else {
                Flash::success(Redirect::to("/worklog/"), "Time tracking stopped")
            }
        },
        Err(e) => {
            log::error!("Failed to stop time tracking: {}", e);
            Flash::error(Redirect::to("/worklog/"), "Failed to stop tracking")
        }
    }
}

#[get("/entries")]
async fn entries_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    settings_service: &State<SettingsService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /entries - Work time entries view");
    let db = conn.into_inner();
    
    match service.get_work_entries_with_roles(db, user.account_id, Some(50), None).await {
        Ok(raw_entries) => {
            // Get user timezone preference
            let user_timezone = settings_service.get_user_timezone(db, user.account_id).await
                .unwrap_or_else(|_| None)
                .unwrap_or_else(|| "UTC".to_string());
            
            // Convert to display format
            let entries = WorkTimeService::format_entries_for_display(raw_entries, &user_timezone);
            
            let roles = service.get_user_roles(db, user.account_id).await.unwrap_or_default();
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
            
            Ok(Template::render(
                "worktime/entries",
                context! {
                    page_title: "Work Time Entries",
                    entries: entries,
                    roles: roles,
                    username: user.username,
                    total_entries: summary.entries_count,
                    total_hours: summary.total_hours,
                    total_earnings: summary.total_earnings,
                    user_timezone: user_timezone,
                }
            ))
        }
        Err(e) => {
            log::error!("Failed to load work time entries: {}", e);
            Err(Flash::error(Redirect::to("/worklog/"), "Failed to load entries"))
        }
    }
}

#[post("/entries", data = "<form>")]
async fn create_manual_entry(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    form: Form<WorkTimeEntryFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /entries - Creating manual entry");
    let db = conn.into_inner();
    
    match service.create_manual_entry(db, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/entries"), "Manual entry created successfully"),
        Err(e) => {
            log::error!("Failed to create manual entry: {}", e);
            Flash::error(Redirect::to("/worklog/entries"), "Failed to create entry")
        }
    }
}

#[delete("/entries/<entry_id>")]
async fn delete_entry(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
) -> Flash<Redirect> {
    log::info!("Route accessed: DELETE /entries/{} - Deleting entry", entry_id);
    let db = conn.into_inner();
    
    match service.delete_work_entry(db, entry_id, user.account_id).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/entries"), "Entry deleted successfully"),
        Err(e) => {
            log::error!("Failed to delete work entry: {}", e);
            Flash::error(Redirect::to("/worklog/entries"), "Failed to delete entry")
        }
    }
}

#[post("/entries/<entry_id>/delete")]
async fn delete_entry_post(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /entries/{}/delete - Deleting entry", entry_id);
    let db = conn.into_inner();
    
    match service.delete_work_entry(db, entry_id, user.account_id).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/entries"), "Entry deleted successfully"),
        Err(e) => {
            log::error!("Failed to delete work entry: {}", e);
            Flash::error(Redirect::to("/worklog/entries"), "Failed to delete entry")
        }
    }
}

#[get("/entries/<entry_id>/edit")]
async fn edit_entry_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /entries/{}/edit - Edit entry view", entry_id);
    let db = conn.into_inner();
    
    match service.get_work_entry_by_id(db, entry_id, user.account_id).await {
        Ok(Some(entry)) => {
            let roles = service.get_user_roles(db, user.account_id).await.unwrap_or_default();
            
            // Format times for HTML datetime-local input
            let start_time_str = entry.start_time.format("%Y-%m-%dT%H:%M").to_string();
            let end_time_str = entry.end_time.map(|t| t.format("%Y-%m-%dT%H:%M").to_string());
            
            Ok(Template::render(
                "worktime/edit_entry",
                context! {
                    page_title: "Edit Work Time Entry",
                    entry: entry,
                    roles: roles,
                    username: user.username,
                    start_time_str: start_time_str,
                    end_time_str: end_time_str,
                }
            ))
        }
        Ok(None) => Err(Flash::error(Redirect::to("/worklog/entries"), "Entry not found")),
        Err(e) => {
            log::error!("Failed to load work entry: {}", e);
            Err(Flash::error(Redirect::to("/worklog/entries"), "Failed to load entry"))
        }
    }
}

#[post("/entries/<entry_id>/edit", data = "<form>")]
async fn edit_entry(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
    form: Form<WorkTimeEntryFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /entries/{}/edit - Update entry", entry_id);
    let db = conn.into_inner();
    
    match service.update_work_entry(db, entry_id, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/entries"), "Entry updated successfully"),
        Err(e) => {
            log::error!("Failed to update work entry: {}", e);
            Flash::error(Redirect::to(format!("/entries/{}/edit", entry_id)), "Failed to update entry")
        }
    }
}

#[get("/notifications")]
async fn notifications_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /notifications - Notification settings view");
    let db = conn.into_inner();
    
    match service.get_notification_settings(db, user.account_id).await {
        Ok(settings) => Ok(Template::render(
            "worktime/notifications",
            context! {
                page_title: "Notification Settings",
                settings: settings,
                username: user.username,
            }
        )),
        Err(e) => {
            log::error!("Failed to load notification settings: {}", e);
            Err(Flash::error(Redirect::to("/worklog/"), "Failed to load notification settings"))
        }
    }
}

#[post("/notifications", data = "<form>")]
async fn update_notifications(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    form: Form<NotificationSettingsFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /notifications - Updating notification settings");
    let db = conn.into_inner();
    
    match service.create_or_update_notification_settings(db, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/notifications"), "Notification settings updated successfully"),
        Err(e) => {
            log::error!("Failed to update notification settings: {}", e);
            Flash::error(Redirect::to("/worklog/notifications"), "Failed to update notification settings")
        }
    }
}

// Pay period management routes
#[get("/payperiods")]
async fn payperiods_view(
    _user: AuthenticatedUser,
) -> Flash<Redirect> {
    log::info!("Route accessed: GET /payperiods - Redirecting to settings page");
    Flash::success(Redirect::to("/worklog/settings"), "Pay period settings are now available in the Settings page")
}

#[post("/payperiods", data = "<_form>")]
async fn create_pay_period(
    _user: AuthenticatedUser,
    _form: Form<PayPeriodFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /payperiods - Redirecting to settings page");
    Flash::success(Redirect::to("/worklog/settings"), "Pay period configuration is now available in the Settings page")
}

#[post("/payperiods/assign")]
async fn auto_assign_entries(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    pay_period_service: &State<PayPeriodService>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /payperiods/assign - Auto-assign entries");
    let db = conn.into_inner();
    
    match pay_period_service.auto_assign_entries_to_pay_periods(db, user.account_id).await {
        Ok(count) => Flash::success(
            Redirect::to("/worklog/payperiods"), 
            &format!("Successfully assigned {} work entries to pay periods", count)
        ),
        Err(e) => {
            log::error!("Failed to auto-assign entries: {}", e);
            Flash::error(Redirect::to("/worklog/payperiods"), "Failed to auto-assign entries")
        }
    }
}

#[post("/payperiods/<_period_id>/delete")]
async fn delete_pay_period(
    _user: AuthenticatedUser,
    _period_id: Uuid,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /payperiods/delete - Redirecting to settings page");
    Flash::success(Redirect::to("/worklog/settings"), "Pay period management is now available in the Settings page")
}

#[get("/timezone")]
async fn timezone_settings_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    settings_service: &State<SettingsService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /timezone - Timezone settings view");
    let db = conn.into_inner();
    
    match settings_service.get_user_timezone(db, user.account_id).await {
        Ok(current_timezone) => {
            let timezones = TimezoneService::get_common_timezones();
            let user_timezone = current_timezone.unwrap_or_else(|| "UTC".to_string());
            
            Ok(Template::render(
                "worktime/timezone",
                context! {
                    page_title: "Timezone Settings",
                    username: user.username,
                    current_timezone: user_timezone,
                    timezones: timezones,
                }
            ))
        },
        Err(e) => {
            log::error!("Failed to load timezone settings: {}", e);
            Err(Flash::error(Redirect::to("/worklog/"), "Failed to load timezone settings"))
        }
    }
}

#[post("/timezone", data = "<form>")]
async fn update_timezone_settings(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    settings_service: &State<SettingsService>,
    form: Form<TimezoneSettingsFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /timezone - Updating timezone settings");
    let db = conn.into_inner();
    let form_data = form.into_inner();
    
    // Validate timezone
    if !TimezoneService::is_valid_timezone(&form_data.timezone) {
        log::warn!("Invalid timezone submitted: {}", form_data.timezone);
        return Flash::error(Redirect::to("/worklog/timezone"), "Invalid timezone selection");
    }
    
    match settings_service.set_user_timezone(db, user.account_id, &form_data.timezone).await {
        Ok(_) => {
            log::info!("Timezone updated successfully for user {} to {}", user.username, form_data.timezone);
            Flash::success(Redirect::to("/worklog/timezone"), "Timezone settings updated successfully")
        },
        Err(e) => {
            log::error!("Failed to update timezone settings: {}", e);
            Flash::error(Redirect::to("/worklog/timezone"), "Failed to update timezone settings")
        }
    }
}

#[get("/settings")]
async fn settings_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    settings_service: &State<SettingsService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /settings - Settings page");
    let db = conn.into_inner();
    
    // Get user timezone preference
    let user_timezone = settings_service.get_user_timezone(db, user.account_id).await
        .unwrap_or_else(|_| None)
        .unwrap_or_else(|| "UTC".to_string());
    
    // Get pay period settings
    let pay_period_settings = settings_service.get_user_pay_period_settings(db, user.account_id).await
        .unwrap_or_else(|_| None)
        .unwrap_or_else(|| models::dto::PayPeriodSettingsDTO {
            start_day: "monday".to_string(),
            period_length: 2,
        });
    
    // Get available timezones
    let timezones = TimezoneService::get_common_timezones();
    
    Ok(Template::render(
        "worktime/settings",
        context! {
            page_title: "Settings",
            username: user.username,
            current_timezone: user_timezone,
            timezones: timezones,
            pay_period_settings: pay_period_settings,
        }
    ))
}

#[post("/settings/timezone", data = "<form>")]
async fn update_timezone_settings_from_settings(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    settings_service: &State<SettingsService>,
    form: Form<TimezoneSettingsFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /settings/timezone - Updating timezone settings");
    let db = conn.into_inner();
    let form_data = form.into_inner();
    
    // Validate timezone
    if !TimezoneService::is_valid_timezone(&form_data.timezone) {
        log::warn!("Invalid timezone submitted: {}", form_data.timezone);
        return Flash::error(Redirect::to("/worklog/settings"), "Invalid timezone selection");
    }
    
    match settings_service.set_user_timezone(db, user.account_id, &form_data.timezone).await {
        Ok(_) => {
            log::info!("Timezone updated successfully for user {} to {}", user.username, form_data.timezone);
            Flash::success(Redirect::to("/worklog/settings"), "Timezone settings updated successfully")
        },
        Err(e) => {
            log::error!("Failed to update timezone settings: {}", e);
            Flash::error(Redirect::to("/worklog/settings"), "Failed to update timezone settings")
        }
    }
}

#[post("/settings/payperiod", data = "<form>")]
async fn update_pay_period_settings(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    settings_service: &State<SettingsService>,
    form: Form<PayPeriodSettingsFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /settings/payperiod - Updating pay period settings");
    let db = conn.into_inner();
    let form_data = form.into_inner();
    
    // Validate inputs
    let valid_days = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
    if !valid_days.contains(&form_data.start_day.as_str()) {
        log::warn!("Invalid start day submitted: {}", form_data.start_day);
        return Flash::error(Redirect::to("/worklog/settings"), "Invalid start day selection");
    }
    
    if ![1, 2, 4].contains(&form_data.period_length) {
        log::warn!("Invalid period length submitted: {}", form_data.period_length);
        return Flash::error(Redirect::to("/worklog/settings"), "Invalid period length selection");
    }
    
    match settings_service.set_user_pay_period_settings(db, user.account_id, &form_data).await {
        Ok(_) => {
            log::info!("Pay period settings updated successfully for user {}", user.username);
            Flash::success(Redirect::to("/worklog/settings"), "Pay period settings updated successfully")
        },
        Err(e) => {
            log::error!("Failed to update pay period settings: {}", e);
            Flash::error(Redirect::to("/worklog/settings"), "Failed to update pay period settings")
        }
    }
}

#[get("/api/stats")]
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

#[get("/entries/<entry_id>/tips")]
async fn tips_entry_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /entries/{}/tips - Tips entry view", entry_id);
    let db = conn.into_inner();
    
    match service.get_work_entry_by_id(db, entry_id, user.account_id).await {
        Ok(Some(entry)) => {
            // Get role info to verify it's a tipped role
            let role = user_role::Entity::find_by_id(entry.user_role_id)
                .one(db)
                .await
                .map_err(|_| Flash::error(Redirect::to("/worklog/"), "Failed to load role"))?
                .ok_or(Flash::error(Redirect::to("/worklog/"), "Role not found"))?;
                
            if !role.is_tipped {
                return Err(Flash::error(Redirect::to("/worklog/"), "This role is not configured for tips"));
            }
            
            // Calculate base earnings
            let duration_hours = entry.duration.unwrap_or(0) as f64 / 60.0;
            let base_earnings = duration_hours * role.hourly_wage;
            
            Ok(Template::render(
                "worktime/tips_entry",
                context! {
                    page_title: "Enter Tips",
                    entry: entry,
                    role: role,
                    username: user.username,
                    duration_hours: duration_hours,
                    base_earnings: base_earnings,
                }
            ))
        }
        Ok(None) => Err(Flash::error(Redirect::to("/worklog/"), "Entry not found")),
        Err(e) => {
            log::error!("Failed to load work entry: {}", e);
            Err(Flash::error(Redirect::to("/worklog/"), "Failed to load entry"))
        }
    }
}

#[post("/entries/<entry_id>/tips", data = "<form>")]
async fn submit_tips(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
    entry_id: Uuid,
    form: Form<TipEntryFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /entries/{}/tips - Submit tips", entry_id);
    let db = conn.into_inner();
    
    match service.add_tips_to_entry(db, entry_id, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worklog/"), "Tips added successfully!"),
        Err(e) => {
            log::error!("Failed to add tips: {}", e);
            Flash::error(Redirect::to(format!("/entries/{}/tips", entry_id)), "Failed to add tips")
        }
    }
}

fn routes() -> Vec<rocket::Route> {
    routes![
        home,
        login,
        logout,
        register_view,
        register,
        dashboard,
        api_stats,
        roles_view,
        create_role,
        edit_role,
        delete_role,
        start_tracking,
        stop_tracking,
        entries_view,
        create_manual_entry,
        edit_entry_view,
        edit_entry,
        delete_entry,
        delete_entry_post,
        tips_entry_view,
        submit_tips,
        notifications_view,
        update_notifications,
        payperiods_view,
        auto_assign_entries,
        create_pay_period,
        delete_pay_period,
        timezone_settings_view,
        update_timezone_settings,
        settings_view,
        update_timezone_settings_from_settings,
        update_pay_period_settings,
    ]
}

crate::impl_controller_routes!(Controller, "Work Time Controller", routes());
