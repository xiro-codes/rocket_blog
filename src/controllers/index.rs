use rocket::{
    fairing::{self, Fairing, Kind},
    response::Redirect,
    Build, Rocket, Route,
};

pub struct Controller {
    path: String,
}
impl Controller {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}
#[get("/")]
fn index() -> Redirect {
    Redirect::to("/blog?page=1")
}
pub fn routes() -> Vec<Route> {
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
        Ok(rocket.mount(self.path.to_owned(), routes()))
    }
}
