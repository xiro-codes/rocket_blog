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
use migrations::MigratorTrait;
use pool::Db;
use rocket::{fairing, fairing::AdHoc, fs::FileServer, response::Redirect, Build, Rocket};
use rocket_dyn_templates::Template;
use sea_orm_rocket::Database;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migrations::Migrator::up(conn, None).await;
    Ok(rocket)
}

#[catch(default)]
pub fn catch_default() -> Redirect {
    Redirect::to("/")
}

use std::time::SystemTime;

/// Unified log filter for noisy dependencies
fn should_filter_log(meta: &log::Metadata) -> bool {
    let target = meta.target();
    // Filter out noisy log targets
    target.starts_with("rocket") || 
    target.starts_with("sea_orm_migration") || 
    target.starts_with("sqlx") || 
    target.starts_with("hyper") || 
    target.eq("_")
}

/// Setup application logging with clean filtering
fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(Features::log_level())
        .filter(|meta| !should_filter_log(meta))
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

#[launch]
async fn rocket() -> _ {
    let figment = rocket::Config::figment();
    let app_config = AppConfig::from_figment(&figment);
    
    // Setup logging - ignore errors if already initialized
    let _ = setup_logger();
    
    // Build the base rocket instance
    let mut rocket = rocket::build()
        .register("/", catchers![catch_default])
        .attach(Db::init())
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .attach(ServiceRegistry::fairing())
        .manage(app_config);
    
    // Only attach seeding in debug builds (development mode)
    if Features::enable_seeding() {
        rocket = rocket.attach(middleware::Seeding::new(Some(0), 50));
    }
    
    // Attach all controllers
    ControllerRegistry::attach_all_controllers(rocket)
        .mount("/static", FileServer::from("./static/"))
}
