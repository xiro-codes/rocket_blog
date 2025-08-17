// Temporarily removed allow directives to identify actual issues
#[macro_use]
extern crate rocket;

mod config;
mod controllers;
mod dto;
mod middleware;
mod pool;
mod services;
mod types;

#[cfg(test)]
mod tests;

use config::AppConfig;
use migrations::MigratorTrait;
use pool::Db;
use rocket::{fairing, fairing::AdHoc, fs::FileServer, response::Redirect, Build, Rocket};
use rocket_dyn_templates::Template;
use sea_orm_rocket::Database;
use services::{TagService, ReactionService};

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
        .level(log::LevelFilter::Debug)
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
    
    let mut rocket = rocket::build()
        .register("/", catchers![catch_default])
        .attach(Db::init())
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations));
    
    // Only attach seeding in debug builds (development mode)
    #[cfg(debug_assertions)]
    {
        rocket = rocket.attach(middleware::Seeding::new(Some(0), 50));
    }
    
    rocket
        .manage(TagService::new())
        .manage(ReactionService::new())
        .manage(app_config)
        .attach(controllers::IndexController::new("/".to_owned()))
        .attach(controllers::AuthController::new("/auth".to_owned()))
        .attach(controllers::BlogController::new("/blog".to_owned()))
        .attach(controllers::CommentController::new("/comment".to_owned()))
        .attach(controllers::FeedController::new("/feed".to_owned()))
        .mount("/static", FileServer::from("./static/"))
}
