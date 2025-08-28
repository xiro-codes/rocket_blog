// Temporarily removed allow directives to identify actual issues
#[macro_use]
extern crate rocket;

mod config;
mod controllers;
mod dto;
mod features;
mod guards;
mod middleware;
mod pool;
mod registry;
mod responders;
mod services;
mod types;

#[cfg(test)]
mod tests;

use config::AppConfig;
use features::Features;
use registry::{ServiceRegistry, ControllerRegistry};
use common::database::{Db};
use rocket::{fairing::AdHoc, fs::FileServer, response::Redirect};
use rocket_dyn_templates::Template;
use sea_orm_rocket::Database;

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed - redirecting to home page");
    Redirect::to("/")
}

#[launch]
async fn rocket() -> _ {
    let figment = rocket::Config::figment();
    let app_config = AppConfig::from_figment(&figment);
    
    // Setup logging - ignore errors if already initialized
    let _ = common::utils::setup_logger();
    log::info!("Starting Rocket Blog application...");
    log::debug!("Development mode: {}", Features::is_development());
    log::debug!("Seeding enabled: {}", Features::enable_seeding());
    log::debug!("Log level: {:?}", Features::log_level());
    
    // Build the base rocket instance
    log::info!("Building Rocket instance and attaching services...");
    let mut rocket = rocket::build()
        .register("/", catchers![catch_default])
        .attach(Db::init())
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", common::database::run_migrations))
        .attach(ServiceRegistry::fairing())
        .manage(app_config);
    
    // Only attach seeding in debug builds (development mode)
    if Features::enable_seeding() {
        log::info!("Attaching database seeding middleware");
        rocket = rocket.attach(middleware::Seeding::new(Some(0), 50));
    }
    
    log::info!("Attaching controllers and static file server");
    // Attach all controllers
    ControllerRegistry::attach_all_controllers(rocket)
        .mount("/static", FileServer::from("./static/"))
}
