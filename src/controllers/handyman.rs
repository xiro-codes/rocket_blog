use rocket::{
    fairing::{self, Fairing, Kind},
    Build, Rocket, Route,
};
use rocket_dyn_templates::{context, Template};

use crate::controllers::base::ControllerBase;

pub struct Controller {
    base: ControllerBase,
}

impl Controller {
    pub fn new(path: String) -> Self {
        Self {
            base: ControllerBase::new(path),
        }
    }
}

#[get("/")]
fn index() -> Template {
    log::info!("Route accessed: GET /handyman - Handyman home page accessed");
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

fn routes() -> Vec<Route> {
    routes![index, handyman1, handyman2, handyman3, handyman4]
}

crate::impl_controller_routes!(Controller, "Handyman Controller", routes());
