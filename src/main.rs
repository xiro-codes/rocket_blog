#![allow(renamed_and_removed_lints)]
#[macro_use]
extern crate rocket;

mod controllers;
mod middleware;
mod pool;
mod services;

use migrations::MigratorTrait;
use pool::Db;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::response::Redirect;
use rocket::{fairing, Build, Request, Rocket};
use rocket_dyn_templates::{context, Template};
use sea_orm::*;
use sea_orm_rocket::Database;

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migrations::Migrator::up(conn, None).await;
    Ok(rocket)
}

#[catch(default)]
pub fn catch_default() -> Redirect{
    Redirect::to("/")
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .register("/", catchers![catch_default])
        .attach(controllers::IndexController::new("/".to_owned()))
        .mount("/static", FileServer::from("./static/"))
}
