#[macro_use]
extern crate rocket;

mod controllers;
mod services;

use rocket::{response::Redirect, fs::FileServer};
use rocket_dyn_templates::Template;

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
    
    // Build the rocket instance (without database for demo purposes)
    log::info!("Building Hello World Rocket instance...");
    rocket::build()
        .register("/", catchers![catch_default])
        .attach(Template::fairing())
        .attach(services::HelloWorldService::fairing())
        .mount("/", routes![
            controllers::index,
            controllers::hello,
            controllers::selections,
            controllers::api_selections
        ])
        .mount("/static", FileServer::from("../../static/"))
}