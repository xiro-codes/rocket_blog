use rocket::{serde::json::Json, State};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use crate::services::HelloWorldService;

#[derive(Serialize, Deserialize, Clone)]
pub struct SelectionOption {
    pub id: u32,
    pub name: String,
    pub description: String,
}

#[get("/")]
pub fn index() -> Template {
    log::info!("Route accessed: GET / - Hello World index page");
    Template::render("index", context! {
        title: "Hello World Selection App",
        message: "Welcome to the Hello World Selection Application!"
    })
}

#[get("/hello")]
pub fn hello() -> Template {
    log::info!("Route accessed: GET /hello - Hello page");
    Template::render("hello", context! {
        title: "Hello",
        greeting: "Hello, World!",
        message: "This is a simple hello world page."
    })
}

#[get("/selections")]
pub fn selections(service: &State<HelloWorldService>) -> Template {
    log::info!("Route accessed: GET /selections - Selections page");
    let options = service.get_selection_options();
    Template::render("selections", context! {
        title: "Make a Selection",
        options: options,
        message: "Choose one of the options below:"
    })
}

#[get("/api/selections")]
pub fn api_selections(service: &State<HelloWorldService>) -> Json<Vec<SelectionOption>> {
    log::info!("Route accessed: GET /api/selections - API endpoint for selections");
    let options = service.get_selection_options();
    Json(options)
}