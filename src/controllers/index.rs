use rocket::{
    fairing::{self, Fairing, Kind},
    response::Redirect,
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
fn index() -> Redirect {
    log::info!("Route accessed: GET / - Home page accessed, redirecting to blog");
    Redirect::to("/blog/?page=1")
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

#[rocket::async_trait]
impl Fairing for Controller {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Index Controller",
            kind: Kind::Ignite,
        }
    }
    
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket.mount(self.base.path(), routes()))
    }
}
