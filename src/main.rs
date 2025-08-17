#![allow(renamed_and_removed_lints)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]
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
use rocket::{fairing, fairing::AdHoc, fs::FileServer, response::Redirect, Build, Request, Rocket};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Database;
use services::{TagService, ReactionService, SettingsService};
use std::time::SystemTime;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migrations::Migrator::up(conn, None).await;
    Ok(rocket)
}

#[catch(default)]
pub fn catch_default() -> Redirect {
    Redirect::to("/")
}

fn drop_rocket(meta: &log::Metadata) -> bool {
    let name = meta.target();
    if name.starts_with("rocket") || name.eq("_") {
        return false;
    }
    true
}
fn drop_sea_orm_migration(meta: &log::Metadata) -> bool {
    let name = meta.target();
    if name.starts_with("sea_orm_migration") || name.eq("_") {
        return false;
    }
    true
}
fn drop_sqlx(meta: &log::Metadata) -> bool {
    let name = meta.target();
    if name.starts_with("sqlx") || name.eq("_") {
        return false;
    }
    true
}
fn drop_hyper(meta: &log::Metadata) -> bool {
    let name = meta.target();
    if name.starts_with("hyper") || name.eq("_") {
        return false;
    }
    true
}
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
        .filter(drop_rocket)
        .filter(drop_sqlx)
        .filter(drop_sea_orm_migration)
        .filter(drop_hyper)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

#[launch]
async fn rocket() -> _ {
    let figment = rocket::Config::figment();
    let app_config = AppConfig::from_figment(&figment);
    //setup_logger().unwrap();
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
        .manage(SettingsService::new())
        .manage(app_config)
        .attach(controllers::IndexController::new("/".to_owned()))
        .attach(controllers::AuthController::new("/auth".to_owned()))
        .attach(controllers::AdminController::new("/admin".to_owned()))
        .attach(controllers::BlogController::new("/blog".to_owned()))
        .attach(controllers::CommentController::new("/comment".to_owned()))
        .attach(controllers::FeedController::new("/feed".to_owned()))
        .mount("/static", FileServer::from("./static/"))
}
