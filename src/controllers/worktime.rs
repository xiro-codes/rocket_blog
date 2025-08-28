use models::dto::{UserRoleFormDTO, WorkTimeEntryFormDTO, TimeTrackingControlDTO, WorkTimeSummaryDTO};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    response::{Flash, Redirect},
    routes, Build, Rocket, State,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use uuid::Uuid;
use rust_decimal::Decimal;

use crate::{
    controllers::base::ControllerBase,
    guards::AuthenticatedUser,
    pool::Db,
    services::WorkTimeService,
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
async fn dashboard(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /worktime/ - Work time dashboard");
    let db = conn.into_inner();
    
    match service.get_user_roles(db, user.account_id).await {
        Ok(roles) => {
            let active_entry = service.get_active_entry(db, user.account_id).await.unwrap_or(None);
            let recent_entries = service.get_work_entries_with_roles(db, user.account_id, Some(10), None).await.unwrap_or_default();
            let summary = service.get_work_time_summary(db, user.account_id, None, None).await.unwrap_or_else(|_| {
                WorkTimeSummaryDTO {
                    total_hours: Decimal::from(0),
                    total_earnings: Decimal::from(0),
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
    log::info!("Route accessed: GET /worktime/roles - User roles management");
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
            Err(Flash::error(Redirect::to("/worktime"), "Failed to load roles"))
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
    log::info!("Route accessed: POST /worktime/roles - Creating user role");
    let db = conn.into_inner();
    
    match service.create_user_role(db, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worktime/roles"), "Role created successfully"),
        Err(e) => {
            log::error!("Failed to create user role: {}", e);
            Flash::error(Redirect::to("/worktime/roles"), "Failed to create role")
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
    log::info!("Route accessed: POST /worktime/start - Starting time tracking");
    let db = conn.into_inner();
    
    match service.start_time_tracking(db, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worktime"), "Time tracking started"),
        Err(e) => {
            log::error!("Failed to start time tracking: {}", e);
            Flash::error(Redirect::to("/worktime"), &format!("Failed to start tracking: {}", e))
        }
    }
}

#[post("/stop")]
async fn stop_tracking(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Flash<Redirect> {
    log::info!("Route accessed: POST /worktime/stop - Stopping time tracking");
    let db = conn.into_inner();
    
    match service.stop_time_tracking(db, user.account_id).await {
        Ok(_) => Flash::success(Redirect::to("/worktime"), "Time tracking stopped"),
        Err(e) => {
            log::error!("Failed to stop time tracking: {}", e);
            Flash::error(Redirect::to("/worktime"), "Failed to stop tracking")
        }
    }
}

#[get("/entries")]
async fn entries_view(
    conn: Connection<'_, Db>,
    user: AuthenticatedUser,
    service: &State<WorkTimeService>,
) -> Result<Template, Flash<Redirect>> {
    log::info!("Route accessed: GET /worktime/entries - Work time entries view");
    let db = conn.into_inner();
    
    match service.get_work_entries_with_roles(db, user.account_id, Some(50), None).await {
        Ok(entries) => {
            let roles = service.get_user_roles(db, user.account_id).await.unwrap_or_default();
            Ok(Template::render(
                "worktime/entries",
                context! {
                    page_title: "Work Time Entries",
                    entries: entries,
                    roles: roles,
                    username: user.username,
                }
            ))
        }
        Err(e) => {
            log::error!("Failed to load work time entries: {}", e);
            Err(Flash::error(Redirect::to("/worktime"), "Failed to load entries"))
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
    log::info!("Route accessed: POST /worktime/entries - Creating manual entry");
    let db = conn.into_inner();
    
    match service.create_manual_entry(db, user.account_id, form.into_inner()).await {
        Ok(_) => Flash::success(Redirect::to("/worktime/entries"), "Manual entry created successfully"),
        Err(e) => {
            log::error!("Failed to create manual entry: {}", e);
            Flash::error(Redirect::to("/worktime/entries"), "Failed to create entry")
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
    log::info!("Route accessed: DELETE /worktime/entries/{} - Deleting entry", entry_id);
    let db = conn.into_inner();
    
    match service.delete_work_entry(db, entry_id, user.account_id).await {
        Ok(_) => Flash::success(Redirect::to("/worktime/entries"), "Entry deleted successfully"),
        Err(e) => {
            log::error!("Failed to delete work entry: {}", e);
            Flash::error(Redirect::to("/worktime/entries"), "Failed to delete entry")
        }
    }
}

#[rocket::async_trait]
impl Fairing for Controller {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Work Time Controller",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let routes = routes![
            dashboard,
            roles_view,
            create_role,
            start_tracking,
            stop_tracking,
            entries_view,
            create_manual_entry,
            delete_entry,
        ];
        
        log::info!("Mounting work time routes at: {}", self.base.path());
        Ok(rocket.mount(self.base.path(), routes))
    }
}