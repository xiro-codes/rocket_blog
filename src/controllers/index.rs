use rocket::{
    fairing::{self, Fairing, Kind},
    request::{FromRequest, Outcome, Request},
    response::Redirect,
    Build, Rocket, Route,
};
use rocket_dyn_templates::{context, Template};
use serde_json::json;

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

pub struct HostHeader(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HostHeader {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let host = request.headers().get_one("Host").unwrap_or("").to_string();
        Outcome::Success(HostHeader(host))
    }
}

#[get("/")]
fn index(host: HostHeader) -> Result<Template, Redirect> {
    if host.0.starts_with("blog.") {
        log::info!("Route accessed: GET / - Blog home page accessed, redirecting to blog list");
        Err(Redirect::to("/blog/?page=1"))
    } else {
        log::info!("Route accessed: GET / - Portfolio home page accessed");
        Ok(Template::render("portfolio", json!({
            "contact": {
                "first": "Travis O.",
                "last": "Davis",
                "position": "System & Software Developer",
                "address": "Cordova, Tennessee",
                "email": "me@tdavis.dev",
                "mobile": "(901)-505-9122",
                "github": "xiro-codes"
            },
            "all_skills": {
                "systems": "Rust, Nix/NixOS, Python, Shell Scripting",
                "backend": "SQL (PostgreSQL, MySql, SQLite), REST APIs",
                "web": "React, Javascript, HTML5, CSS3"
            },
            "experience": [
                {
                    "title": "Instructor/Software Developer",
                    "org": "York Solutions",
                    "location": "Westchester, Il.",
                    "date": "Apr. 2022 - Apr. 2023",
                    "items": [
                        "Conducted comprehensive training sessions for groups of up to 20 individuals on Full Stack Web Development.",
                        "Collaborated in the development of internal tools to streamline testing procedures.",
                        "Offered continuous support and supplementary training to contracted developers."
                    ]
                },
                {
                    "title": "Assistant Programmer",
                    "org": "Upper Edge Technologies",
                    "location": "West Memphis, Ar.",
                    "date": "Dec. 2019 - Jan. 2022",
                    "items": [
                        "Developed a Customer Quoting tool, integrating catalog browsing and automating manual bookkeeping.",
                        "Engineered a digital RMA process using third-party APIs including QuickBooks, BigCommerce, & Jira.",
                        "Created REST APIs for receiving parts and laptop specifications alongside inventory numbers."
                    ]
                },
                {
                    "title": "Teacher's Assistant",
                    "org": "Tech 901",
                    "location": "Memphis, Tn.",
                    "date": "Mar. 2022 - Jul. 2022",
                    "items": [
                        "Collaborated with teachers in planning and preparing lessons to increase engagement.",
                        "Provided technical support to students and staff for hardware and software issues."
                    ]
                }
            ]
        })))
    }
}

#[get("/offline")]
fn offline() -> Template {
    log::info!("Route accessed: GET /offline - PWA offline page");
    Template::render("offline", context! {
        page_title: "Offline - Work Time Tracker",
        page_description: "Work Time Tracker is currently offline but still functional"
    })
}

fn routes() -> Vec<Route> {
    routes![index, offline]
}

crate::impl_controller_routes!(Controller, "Index Controller", routes());
