// Shared library for both blog and worktime binaries
#[macro_use]
extern crate rocket;

pub mod config;
pub mod controllers;
pub mod dto;
pub mod features;
pub mod guards;
pub mod middleware;
pub mod pool;
pub mod registry;
pub mod responders;
pub mod services;
pub mod types;

#[cfg(test)]
pub mod tests;

use config::AppConfig;
use features::Features;
use migrations::MigratorTrait;
use pool::Db;
use rocket::{fairing, fairing::AdHoc, Build, Rocket};
use sea_orm_rocket::Database;
use std::time::SystemTime;

pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    log::info!("Starting database migrations...");
    let conn = &Db::fetch(&rocket).unwrap().conn;
    
    match migrations::Migrator::up(conn, None).await {
        Ok(_) => {
            log::info!("Database migrations completed successfully");
        }
        Err(e) => {
            log::error!("Database migration failed: {}", e);
        }
    }
    
    Ok(rocket)
}

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
pub fn setup_logger() -> Result<(), fern::InitError> {
    let mut dispatch = fern::Dispatch::new()
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
        .chain(fern::log_file("output.log")?);
    
    // In development mode, also log to stdout for real-time feedback
    if Features::is_development() {
        dispatch = dispatch.chain(std::io::stdout());
    }
    
    dispatch.apply()?;
    Ok(())
}

/// Create base rocket instance with common configuration
pub fn create_base_rocket() -> Rocket<Build> {
    let figment = rocket::Config::figment();
    let app_config = AppConfig::from_figment(&figment);
    
    // Setup logging - ignore errors if already initialized
    let _ = setup_logger();
    
    // Build the base rocket instance with common components
    let rocket = rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .manage(app_config);
    
    rocket
}