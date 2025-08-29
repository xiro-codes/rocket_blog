use models::dto::{UserRoleFormDTO, WorkTimeEntryFormDTO, TimeTrackingControlDTO, WorkTimeSummaryDTO, NotificationSettingsFormDTO, PayPeriodFormDTO, AccountFormDTO};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    response::{Flash, Redirect},
    routes, Build, Rocket, State,
    http::{Cookie, CookieJar},
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use uuid::Uuid;

use crate::{
    controllers::base::ControllerBase,
    guards::{AuthenticatedUser, OptionalUser},
    pool::Db,
    services::{WorkTimeService, PayPeriodService, AuthService},
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
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET / - Work time home page");
    let db = conn.into_inner();
    
    // If user has a token, verify it and show dashboard
    if let Some(token) = user.token {
        if let Some(account) = auth_service.check_token(db, token).await {
            // User is authenticated, show dashboard
            match service.get_user_roles(db, account.id).await {
                Ok(roles) => {
                    let active_entry = service.get_active_entry(db, account.id).await.unwrap_or(None);
                    let recent_entries = service.get_work_entries_with_roles(db, account.id, Some(10), None).await.unwrap_or_default();
                    let summary = service.get_work_time_summary(db, account.id, None, None).await.unwrap_or_else(|_| {
                        WorkTimeSummaryDTO {
                            total_hours: 0.0,
                            total_earnings: 0.0,
                            currency: "USD".to_string(),
                            entries_count: 0,
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
                        }
                    ));
                }
                Err(e) => {
                    log::error!("Failed to load work time dashboard: {}", e);
                    return Err(Flash::error(Redirect::to("/"), "Failed to load dashboard"));
                }
            }
        }
    }
    
    // User is not authenticated or invalid token, show login page
    // Check if any accounts exist, return 404 if none
    if !auth_service.has_any_accounts(db).await {
        log::info!("No accounts exist in system - worktime login page not available");
        return Err(Flash::error(Redirect::to("/"), "No accounts available"));
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
        cookies.add_private(Cookie::new("token", token.to_string()));
        log::info!("Worktime authentication successful - Redirecting to worktime dashboard");
        Flash::success(
            Redirect::to("/"),
            "Login successful! Welcome to your work time tracker.",
        )
    } else {
        log::warn!("Worktime authentication failed - Redirecting back to login form");
        Flash::error(
            Redirect::to("/"),
            "Invalid username or password. Please try again.",
        )
    }
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    log::info!("Route accessed: GET /logout - Worktime user logout requested");
    
    cookies.remove_private(Cookie::from("token"));
    
    log::debug!("Worktime user successfully logged out - Redirecting to worktime login");
    Flash::success(
        Redirect::to("/"),
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
                Redirect::to("/"),
                "Account created successfully! You can now log in to access the work time tracker.",
            )
        }
        Err(e) => {
            log::warn!("Failed to create worktime user account for username: {} - Error: {}", username, e);
            Flash::error(
                Redirect::to("/register"),
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
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /dashboard - Work time dashboard");
    let db = conn.into_inner();
    
    match service.get_user_roles(db, user.account_id).await {
        Ok(roles) => {
            let active_entry = service.get_active_entry(db, user.account_id).await.unwrap_or(None);
            let recent_entries = service.get_work_entries_with_roles(db, user.account_id, Some(10), None).await.unwrap_or_default();
            let summary = service.get_work_time_summary(db, user.account_id, None, None).await.unwrap_or_else(|_| {
                WorkTimeSummaryDTO {
                    total_hours: 0.0,
                    total_earnings: 0.0,
                    currency: "USD".to_string(),
                    entries_count: 0,
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
                }
            ))
        }
        Err(e) => {
            log::error!("Failed to load work time dashboard: {}", e);
            Err(Flash::error(Redirect::to("/"), "Failed to load dashboard"))
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
            Err(Flash::error(Redirect::to("/"), "Failed to load roles"))
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
        Ok(_) => Flash::success(Redirect::to("/roles"), "Role created successfully"),
        Err(e) => {
            log::error!("Failed to create user role: {}", e);
            Flash::error(Redirect::to("/roles"), "Failed to create role")
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
        Ok(_) => Flash::success(Redirect::to("/"), "Time tracking started"),
        Err(e) => {
            log::error!("Failed to start time tracking: {}", e);
            Flash::error(Redirect::to("/"), &format!("Failed to start tracking: {}", e))
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
    
    match service.stop_time_tracking(db, user.account_id).await {
        Ok(_) => Flash::success(Redirect::to("/"), "Time tracking stopped"),
        Err(e) => {
            log::error!("Failed to stop time tracking: {}", e);
            Flash::error(Redirect::to("/"), "Failed to stop tracking")
        }
    }
}

#[get("/entries")]
async fn entries_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /entries - Work time entries view");
    let db = conn.into_inner();
    
    match service.get_work_entries_with_roles(db, user.account_id, Some(50), None).await {
        Ok(entries) => {
            let roles = service.get_user_roles(db, user.account_id).await.unwrap_or_default();
            let summary = service.get_work_time_summary(db, user.account_id, None, None).await.unwrap_or_else(|_| {
                WorkTimeSummaryDTO {
                    total_hours: 0.0,
                    total_earnings: 0.0,
                    currency: "USD".to_string(),
                    entries_count: 0,
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
                }
            ))
        }
        Err(e) => {
            log::error!("Failed to load work time entries: {}", e);
            Err(Flash::error(Redirect::to("/"), "Failed to load entries"))
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
        Ok(_) => Flash::success(Redirect::to("/entries"), "Manual entry created successfully"),
        Err(e) => {
            log::error!("Failed to create manual entry: {}", e);
            Flash::error(Redirect::to("/entries"), "Failed to create entry")
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
        Ok(_) => Flash::success(Redirect::to("/entries"), "Entry deleted successfully"),
        Err(e) => {
            log::error!("Failed to delete work entry: {}", e);
            Flash::error(Redirect::to("/entries"), "Failed to delete entry")
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
            Err(Flash::error(Redirect::to("/"), "Failed to load notification settings"))
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
        Ok(_) => Flash::success(Redirect::to("/notifications"), "Notification settings updated successfully"),
        Err(e) => {
            log::error!("Failed to update notification settings: {}", e);
            Flash::error(Redirect::to("/notifications"), "Failed to update notification settings")
        }
    }
}

// Pay period management routes
#[get("/payperiods")]
async fn payperiods_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    pay_period_service: &State<PayPeriodService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /payperiods - Pay periods management");
    let db = conn.into_inner();
    
    match pay_period_service.get_pay_periods_with_summary(db, user.account_id).await {
        Ok(pay_periods) => Ok(Template::render(
            "worktime/payperiods",
            context! {
                page_title: "Manage Pay Periods",
                pay_periods: pay_periods,
                username: user.username,
            }
        )),
        Err(e) => {
            log::error!("Failed to load pay periods: {}", e);
            Err(Flash::error(Redirect::to("/"), "Failed to load pay periods"))
        }
    }
}

#[post("/payperiods", data = "<form>")]
async fn create_pay_period(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    pay_period_service: &State<PayPeriodService>,
    form: Form<PayPeriodFormDTO>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /payperiods - Creating pay period");
    let db = conn.into_inner();
    
    match pay_period_service.create_pay_period(db, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/payperiods"), "Pay period created successfully"),
        Err(e) => {
            log::error!("Failed to create pay period: {}", e);
            Flash::error(Redirect::to("/payperiods"), &format!("Failed to create pay period: {}", e))
        }
    }
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
            Redirect::to("/payperiods"), 
            &format!("Successfully assigned {} work entries to pay periods", count)
        ),
        Err(e) => {
            log::error!("Failed to auto-assign entries: {}", e);
            Flash::error(Redirect::to("/payperiods"), "Failed to auto-assign entries")
        }
    }
}

#[post("/payperiods/<period_id>/delete")]
async fn delete_pay_period(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    pay_period_service: &State<PayPeriodService>,
    period_id: Uuid,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /payperiods/{}/delete - Deleting pay period", period_id);
    let db = conn.into_inner();
    
    match pay_period_service.delete_pay_period(db, period_id, user.account_id).await {
        Ok(_) => Flash::success(Redirect::to("/payperiods"), "Pay period deleted successfully"),
        Err(e) => {
            log::error!("Failed to delete pay period: {}", e);
            Flash::error(Redirect::to("/payperiods"), "Failed to delete pay period")
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
        roles_view,
        create_role,
        start_tracking,
        stop_tracking,
        entries_view,
        create_manual_entry,
        delete_entry,
        notifications_view,
        update_notifications,
        // payperiods_view,
        // create_pay_period,
        // auto_assign_entries,
        // delete_pay_period,  // Temporarily disabled for debugging
    ]
}

crate::impl_controller_routes!(Controller, "Work Time Controller", routes());