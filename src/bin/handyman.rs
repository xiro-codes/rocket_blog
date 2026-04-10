use app::{features::Features, template_config};
use rocket::{
    catch, catchers, fs::FileServer, get, launch, response::Redirect, routes, Build, Rocket,
};
use rocket_dyn_templates::{context, Template};

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed - redirecting to home page");
    Redirect::to("/")
}

#[get("/handyman")]
fn index() -> Template {
    log::info!("Route accessed: GET / - Handyman home page accessed");
    Template::render("handyman", context! {})
}

#[launch]
async fn rocket() -> Rocket<Build> {
    log::info!("Starting Handyman application...");
    log::debug!("Development mode: {}", Features::is_development());
    log::debug!("Log level: {:?}", Features::log_level());

    let rocket = rocket::build()
        .register("/", catchers![catch_default])
        .attach(template_config::create_template_fairing());

    rocket
        .mount("/", routes![index])
        .mount("/static", FileServer::from("./static/"))
}
