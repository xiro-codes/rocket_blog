#[macro_use]
extern crate rocket;

mod controllers;
mod services;
mod guards;

use common::database::Db;
use controllers::punch_clock::PunchClockController;
use rocket::{fairing::AdHoc, fs::FileServer, response::Redirect};
use rocket_dyn_templates::Template;
use sea_orm_rocket::Database;
use services::{WorkRoleService, WorkSessionService};

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed in punch clock app - redirecting to home page");
    Redirect::to("/punch-clock")
}

#[launch]
async fn rocket() -> _ {
    // Setup logging - ignore errors if already initialized
    let _ = common::utils::setup_logger();
    log::info!("Starting Punch Clock application...");
    
    // Build the rocket instance
    rocket::build()
        .register("/", catchers![catch_default])
        .attach(Db::init())
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", common::database::run_migrations))
        .manage(WorkRoleService::new())
        .manage(WorkSessionService::new())
        .mount("/punch-clock", PunchClockController::routes())
        .mount("/static", FileServer::from("./static/"))
}