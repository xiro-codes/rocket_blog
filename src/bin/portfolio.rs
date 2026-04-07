use app::{
    features::Features,
    template_config,
};
use rocket::{catch, catchers, fs::FileServer, get, launch, response::Redirect, routes, Build, Rocket};
use rocket_dyn_templates::{Template};
use serde_json::json;

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed - redirecting to home page");
    Redirect::to("/")
}

#[get("/")]
fn index() -> Template {
    log::info!("Route accessed: GET / - Portfolio home page accessed");
    Template::render("portfolio", json!({
        "is_portfolio": true,
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
    }))
}

#[launch]
async fn rocket() -> Rocket<Build> {
    log::info!("Starting Portfolio application...");
    log::debug!("Development mode: {}", Features::is_development());
    log::debug!("Log level: {:?}", Features::log_level());

    let rocket = rocket::build()
        .register("/", catchers![catch_default])
        .attach(template_config::create_template_fairing());

    rocket
        .mount("/", routes![index])
        .mount("/static", FileServer::from("./static/"))
}
