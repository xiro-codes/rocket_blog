use crate::controllers::base::ControllerBase;
use crate::guards::admin::Admin;
use crate::services::{AuthService, PollService};
use crate::pool::Db;
use rocket::{
    form::Form,
    get, post,
    response::{Flash, Redirect},
    routes, Route,
    State, request::FlashMessage,
    fairing::{Fairing, Kind},
    Build, Rocket,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(FromForm, Debug)]
struct CreatePollForm {
    title: String,
    description: Option<String>,
    options: Vec<String>,
}

#[derive(FromForm, Debug)]
struct VoteForm {
    option_id: String,
}

pub struct Controller {
    base: ControllerBase,
}

impl Controller {
    pub fn new(mount_path: String) -> Self {
        Self {
            base: ControllerBase::new(mount_path),
        }
    }
}

fn routes() -> Vec<Route> {
    routes![
        list,
        view,
        create_form,
        create,
        vote,
        toggle_active,
        delete
    ]
}

crate::impl_controller_routes!(Controller, "Poll Controller", routes());

/// List all polls
#[get("/")]
async fn list(
    conn: Connection<'_, Db>,
    poll_service: &State<PollService>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Template, rocket::http::Status> {
    let db = conn.into_inner();
    
    match poll_service.list_polls(db, 1, 10).await {
        Ok((polls, _total_pages)) => {
            Ok(Template::render(
                "poll/list",
                context! {
                    polls,
                    flash_kind: flash.as_ref().map(|f| f.kind()),
                    flash_message: flash.as_ref().map(|f| f.message())
                },
            ))
        }
        Err(_) => Err(rocket::http::Status::InternalServerError),
    }
}

/// View a specific poll
#[get("/<seq_id>")]
async fn view(
    conn: Connection<'_, Db>,
    poll_service: &State<PollService>,
    seq_id: i32,
    client_ip: Option<IpAddr>,
) -> Result<Template, rocket::http::Status> {
    let db = conn.into_inner();
    let ip_address = client_ip
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    match poll_service.get_poll_result(db, seq_id, &ip_address).await {
        Ok(poll_result) => {
            Ok(Template::render(
                "poll/view",
                context! {
                    poll_result
                },
            ))
        }
        Err(_) => Err(rocket::http::Status::NotFound),
    }
}

/// Show poll creation form (admin only)
#[get("/create")]
async fn create_form(
    _admin: Admin,
    flash: Option<FlashMessage<'_>>,
) -> Template {
    Template::render(
        "poll/create",
        context! {
            flash_kind: flash.as_ref().map(|f| f.kind()),
            flash_message: flash.as_ref().map(|f| f.message())
        },
    )
}

/// Create a new poll (admin only)
#[post("/", data = "<form>")]
async fn create(
    conn: Connection<'_, Db>,
    poll_service: &State<PollService>,
    _auth_service: &State<AuthService>,
    form: Form<CreatePollForm>,
    _jar: &rocket::http::CookieJar<'_>,
    _admin: Admin,
) -> Result<Flash<Redirect>, rocket::http::Status> {
    let db = conn.into_inner();
    let form_data = form.into_inner();

    // Validate we have at least 2 options
    if form_data.options.len() < 2 {
        return Ok(Flash::error(
            Redirect::to("/poll/create"),
            "A poll must have at least 2 options."
        ));
    }

    // Filter out empty options
    let options: Vec<String> = form_data.options
        .into_iter()
        .filter(|opt| !opt.trim().is_empty())
        .collect();

    if options.len() < 2 {
        return Ok(Flash::error(
            Redirect::to("/poll/create"),
            "A poll must have at least 2 non-empty options."
        ));
    }

    // Get current user from session - we'll use a simple approach for now
    // In a real application, you'd implement proper session management
    let account_id = uuid::Uuid::new_v4(); // TODO: Get from session

    let request = crate::services::poll::CreatePollRequest {
        title: form_data.title,
        description: form_data.description,
        options,
    };

    match poll_service.create(db, request, account_id).await {
        Ok(poll) => Ok(Flash::success(
            Redirect::to(format!("/poll/{}", poll.seq_id)),
            "Poll created successfully!"
        )),
        Err(_) => Ok(Flash::error(
            Redirect::to("/poll/create"),
            "Failed to create poll. Please try again."
        )),
    }
}

/// Vote on a poll
#[post("/<seq_id>/vote", data = "<form>")]
async fn vote(
    conn: Connection<'_, Db>,
    poll_service: &State<PollService>,
    seq_id: i32,
    form: Form<VoteForm>,
    client_ip: Option<IpAddr>,
) -> Result<Flash<Redirect>, rocket::http::Status> {
    let db = conn.into_inner();
    let form_data = form.into_inner();
    
    let ip_address = client_ip
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let option_id = match Uuid::parse_str(&form_data.option_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(Flash::error(
                Redirect::to(format!("/poll/{}", seq_id)),
                "Invalid option selected."
            ));
        }
    };

    match poll_service.vote(db, seq_id, option_id, &ip_address, None).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to(format!("/poll/{}", seq_id)),
            "Your vote has been recorded!"
        )),
        Err(e) => {
            let message = match e {
                sea_orm::DbErr::Custom(msg) => msg,
                _ => "Failed to record your vote. Please try again.".to_string(),
            };
            Ok(Flash::error(
                Redirect::to(format!("/poll/{}", seq_id)),
                &message
            ))
        }
    }
}

/// Toggle poll active status (admin only)
#[post("/<seq_id>/toggle")]
async fn toggle_active(
    conn: Connection<'_, Db>,
    poll_service: &State<PollService>,
    seq_id: i32,
    _admin: Admin,
) -> Result<Flash<Redirect>, rocket::http::Status> {
    let db = conn.into_inner();

    match poll_service.toggle_active(db, seq_id).await {
        Ok(poll) => {
            let status = if poll.active { "activated" } else { "deactivated" };
            Ok(Flash::success(
                Redirect::to(format!("/poll/{}", seq_id)),
                &format!("Poll has been {}.", status)
            ))
        }
        Err(_) => Ok(Flash::error(
            Redirect::to(format!("/poll/{}", seq_id)),
            "Failed to update poll status."
        )),
    }
}

/// Delete poll (admin only)
#[post("/<seq_id>/delete")]
async fn delete(
    conn: Connection<'_, Db>,
    poll_service: &State<PollService>,
    seq_id: i32,
    _admin: Admin,
) -> Result<Flash<Redirect>, rocket::http::Status> {
    let db = conn.into_inner();

    match poll_service.delete(db, seq_id).await {
        Ok(_) => Ok(Flash::success(
            Redirect::to("/poll"),
            "Poll has been deleted."
        )),
        Err(_) => Ok(Flash::error(
            Redirect::to(format!("/poll/{}", seq_id)),
            "Failed to delete poll."
        )),
    }
}