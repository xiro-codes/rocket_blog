#[macro_use]
extern crate rocket;

mod controllers;
mod services;

use rocket::{response::Redirect, fs::FileServer};
use rocket_dyn_templates::Template;
use common::{auth::{AuthService, AuthController, AuthControllerConfig}, database::{Db, run_migrations}, config::AppConfig};
use rocket::fairing::AdHoc;
use sea_orm_rocket::Database;

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed in hello-world app - redirecting to home page");
    Redirect::to("/")
}

#[launch]
async fn rocket() -> _ {
    // Initialize app configuration
    let figment = rocket::Config::figment();
    let app_config = AppConfig::from_figment(&figment);
    
    // Setup logging
    let _ = common::utils::setup_logger();
    log::info!("Starting Hello World Selection application...");
    log::info!("App config: name={}, version={}, environment={}", app_config.name, app_config.version, app_config.environment);
    
    // Configure auth controller for hello-world app
    let auth_config = AuthControllerConfig::new(
        "/".to_string(),       // redirect after login (to hello-world home)
        "/".to_string(),       // redirect after logout (to hello-world home)
        "/auth/".to_string(),  // redirect after register (back to login)
    );
    
    // Build the rocket instance with optional database support
    log::info!("Building Hello World Rocket instance...");
    rocket::build()
        .register("/", catchers![catch_default])
        .attach(Template::fairing())
        .attach(Db::init())
        .attach(AdHoc::on_ignite("Run DB Migrations", |rocket| async {
            run_migrations(rocket).await.unwrap()
        }))
        .manage(app_config)
        .manage(AuthService::new())
        .manage(auth_config)
        .attach(services::HelloWorldService::fairing())
        .attach(AuthController::new("/auth".to_owned()))
        .mount("/", routes![
            controllers::index,
            controllers::hello,
            controllers::selections,
            controllers::api_selections
        ])
        .mount("/static", FileServer::from("../../static/"))
}