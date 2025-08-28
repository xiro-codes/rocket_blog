use rocket::{
    form::Form,
    http::Status,
    response::{Flash, Redirect},
    Route, State,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use common::database::Db;
use crate::{
    guards::{User, Admin},
    services::{WorkRoleService, WorkSessionService},
};
use models::dto::{WorkRoleFormDTO, ClockInFormDTO};

pub struct PunchClockController;

impl PunchClockController {
    pub fn routes() -> Vec<Route> {
        routes![
            dashboard,
            clock_in_view,
            clock_in,
            clock_out,
            history,
            roles_view,
            roles_create_view,
            roles_create,
            roles_edit_view,
            roles_edit,
            roles_delete
        ]
    }
}

/// Main dashboard view
#[get("/")]
async fn dashboard(
    conn: Connection<'_, Db>,
    user: User,
    work_role_service: &State<WorkRoleService>,
    work_session_service: &State<WorkSessionService>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    // Get active session
    let active_session = work_session_service.find_active_session(db, user.id).await
        .map_err(|_| Status::InternalServerError)?;
    
    // Get today's summary
    let (today_minutes, today_earnings) = work_session_service.get_today_summary(db, user.id).await
        .map_err(|_| Status::InternalServerError)?;
    
    // Get total summary
    let (total_minutes, total_earnings) = work_session_service.get_work_summary(db, user.id).await
        .map_err(|_| Status::InternalServerError)?;
    
    // Get available roles for clock in
    let available_roles = work_role_service.find_active(db).await
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Template::render("punch_clock/dashboard", context! {
        active_session,
        today_hours: today_minutes / 60,
        today_minutes: today_minutes % 60,
        today_earnings,
        total_hours: total_minutes / 60,
        total_minutes: total_minutes % 60,
        total_earnings,
        available_roles,
    }))
}

/// Clock in view
#[get("/clock-in")]
async fn clock_in_view(
    conn: Connection<'_, Db>,
    _user: User,
    work_role_service: &State<WorkRoleService>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    let roles = work_role_service.find_active(db).await
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Template::render("punch_clock/clock_in", context! {
        roles,
    }))
}

/// Process clock in
#[post("/clock-in", data = "<data>")]
async fn clock_in(
    conn: Connection<'_, Db>,
    user: User,
    work_session_service: &State<WorkSessionService>,
    data: Form<ClockInFormDTO>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    let form_data = data.into_inner();
    
    match work_session_service.clock_in(db, user.id, form_data).await {
        Ok(_) => {
            Flash::success(Redirect::to("/punch-clock"), "Successfully clocked in!")
        }
        Err(e) => {
            log::error!("Clock in failed: {}", e);
            Flash::error(Redirect::to("/punch-clock"), &format!("Clock in failed: {}", e))
        }
    }
}

/// Process clock out
#[post("/clock-out")]
async fn clock_out(
    conn: Connection<'_, Db>,
    user: User,
    work_session_service: &State<WorkSessionService>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    
    match work_session_service.clock_out(db, user.id).await {
        Ok(session) => {
            let hours = session.duration_minutes.unwrap_or(0) / 60;
            let minutes = session.duration_minutes.unwrap_or(0) % 60;
            let earnings = session.earnings.unwrap_or_default();
            
            Flash::success(
                Redirect::to("/punch-clock"),
                &format!("Clocked out! Worked {}h {}m, earned ${:.2}", hours, minutes, earnings)
            )
        }
        Err(e) => {
            log::error!("Clock out failed: {}", e);
            Flash::error(Redirect::to("/punch-clock"), &format!("Clock out failed: {}", e))
        }
    }
}

/// Work history view
#[get("/history")]
async fn history(
    conn: Connection<'_, Db>,
    user: User,
    work_session_service: &State<WorkSessionService>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    let sessions = work_session_service.find_sessions_with_role(db, user.id).await
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Template::render("punch_clock/history", context! {
        sessions,
    }))
}

/// Roles management view
#[get("/roles")]
async fn roles_view(
    conn: Connection<'_, Db>,
    _admin: Admin,
    work_role_service: &State<WorkRoleService>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    let roles = work_role_service.find_all(db).await
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Template::render("punch_clock/roles", context! {
        roles,
    }))
}

/// Create role view
#[get("/roles/create")]
async fn roles_create_view(_admin: Admin) -> Template {
    Template::render("punch_clock/role_form", context! {
        title: "Create Work Role",
        action: "/punch-clock/roles/create",
    })
}

/// Process create role
#[post("/roles/create", data = "<data>")]
async fn roles_create(
    conn: Connection<'_, Db>,
    _admin: Admin,
    work_role_service: &State<WorkRoleService>,
    data: Form<WorkRoleFormDTO>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    let form_data = data.into_inner();
    
    match work_role_service.create(db, form_data).await {
        Ok(_) => {
            Flash::success(Redirect::to("/punch-clock/roles"), "Work role created successfully!")
        }
        Err(e) => {
            log::error!("Role creation failed: {}", e);
            Flash::error(Redirect::to("/punch-clock/roles/create"), &format!("Failed to create role: {}", e))
        }
    }
}

/// Edit role view
#[get("/roles/<id>/edit")]
async fn roles_edit_view(
    id: String,
    conn: Connection<'_, Db>,
    _admin: Admin,
    work_role_service: &State<WorkRoleService>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    let role_id = uuid::Uuid::parse_str(&id).map_err(|_| Status::BadRequest)?;
    let role = work_role_service.find_by_id(db, role_id).await
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;
    
    Ok(Template::render("punch_clock/role_form", context! {
        title: "Edit Work Role",
        action: format!("/punch-clock/roles/{}/edit", id),
        role,
    }))
}

/// Process edit role
#[post("/roles/<id>/edit", data = "<data>")]
async fn roles_edit(
    id: String,
    conn: Connection<'_, Db>,
    _admin: Admin,
    work_role_service: &State<WorkRoleService>,
    data: Form<WorkRoleFormDTO>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    let form_data = data.into_inner();
    
    let role_id = match uuid::Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Flash::error(Redirect::to("/punch-clock/roles"), "Invalid role ID"),
    };
    
    match work_role_service.update(db, role_id, form_data).await {
        Ok(_) => {
            Flash::success(Redirect::to("/punch-clock/roles"), "Work role updated successfully!")
        }
        Err(e) => {
            log::error!("Role update failed: {}", e);
            Flash::error(Redirect::to("/punch-clock/roles"), &format!("Failed to update role: {}", e))
        }
    }
}

/// Delete role
#[post("/roles/<id>/delete")]
async fn roles_delete(
    id: String,
    conn: Connection<'_, Db>,
    _admin: Admin,
    work_role_service: &State<WorkRoleService>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    
    let role_id = match uuid::Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Flash::error(Redirect::to("/punch-clock/roles"), "Invalid role ID"),
    };
    
    match work_role_service.delete(db, role_id).await {
        Ok(_) => {
            Flash::success(Redirect::to("/punch-clock/roles"), "Work role deleted successfully!")
        }
        Err(e) => {
            log::error!("Role deletion failed: {}", e);
            Flash::error(Redirect::to("/punch-clock/roles"), &format!("Failed to delete role: {}", e))
        }
    }
}