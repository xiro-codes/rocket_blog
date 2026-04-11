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

#[get("/handyman1")]
fn handyman1() -> Template {
    Template::render("handyman1", context! {})
}

#[get("/handyman2")]
fn handyman2() -> Template {
    Template::render("handyman2", context! {})
}

#[get("/handyman3")]
fn handyman3() -> Template {
    Template::render("handyman3", context! {})
}

#[get("/handyman4")]
fn handyman4() -> Template {
    Template::render("handyman4", context! {})
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
        .mount("/", routes![index, handyman1, handyman2, handyman3, handyman4])
        .mount("/static", FileServer::from("./static/"))
}
