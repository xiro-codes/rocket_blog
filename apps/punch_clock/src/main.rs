#[macro_use]
extern crate rocket;

mod controllers;
mod guards;
mod services;

use common::{database::Db, auth::{AuthService, controller::{AuthController, AuthControllerConfig}}, config::AppConfig};
use services::{WorkRoleService, WorkSessionService};
use controllers::punch_clock::PunchClockController;
use rocket::{fairing::AdHoc, fs::FileServer, response::Redirect};
use rocket_dyn_templates::Template;
use sea_orm_rocket::Database;

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed in punch clock app - redirecting to home page");
    Redirect::to("/punch-clock")
}

#[launch]
async fn rocket() -> _ {
    // Initialize app configuration
    let figment = rocket::Config::figment();
    let app_config = AppConfig::from_figment(&figment);
    
    // Setup logging - ignore errors if already initialized
    let _ = common::utils::setup_logger();
    log::info!("Starting Punch Clock application...");
    log::info!("App config: name={}, version={}, environment={}", app_config.name, app_config.version, app_config.environment);
    
    // Configure auth redirects for punch clock app
    let auth_config = AuthControllerConfig::new(
        "/punch-clock".to_string(),          // redirect_after_login
        "/punch-clock".to_string(),          // redirect_after_logout  
        "/punch-clock/auth".to_string(),     // redirect_after_register
    );
    
    // Build the rocket instance
    rocket::build()
        .register("/", catchers![catch_default])
        .attach(Db::init())
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", common::database::run_migrations))
        .attach(AuthController::new("/punch-clock/auth".to_string()))
        .manage(app_config)
        .manage(AuthService::new())
        .manage(auth_config)
        .manage(WorkRoleService::new())
        .manage(WorkSessionService::new())
        .mount("/punch-clock", PunchClockController::routes())
        .mount("/punch-clock/static", FileServer::from("./static/"))
        .mount("/static", FileServer::from("./static/"))
}