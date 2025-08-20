use rocket::{
    fairing::{self, Fairing, Kind},
    response::Redirect,
    Build, Rocket, Route,
};

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
    log::info!("Home page accessed - redirecting to blog");
    Redirect::to("/blog/?page=1")
}

fn routes() -> Vec<Route> {
    routes![index]
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
