#[macro_use]
extern crate rocket;

mod controllers;
mod services;

use common::database::{run_migrations, Db};
use rocket::{fairing::AdHoc, response::Redirect, fs::FileServer};
use rocket_dyn_templates::Template;
use sea_orm_rocket::Database;

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed in hello-world app - redirecting to home page");
    Redirect::to("/")
}

#[launch]
async fn rocket() -> _ {
    // Setup logging
    let _ = common::utils::setup_logger();
    log::info!("Starting Hello World Selection application...");
    
    // Build the rocket instance
    log::info!("Building Hello World Rocket instance...");
    rocket::build()
        .register("/", catchers![catch_default])
        .attach(Db::init())
        .attach(Template::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .attach(services::HelloWorldService::fairing())
        .mount("/", routes![
            controllers::index,
            controllers::hello,
            controllers::selections,
            controllers::api_selections
        ])
        .mount("/static", FileServer::from("../../static/"))
}