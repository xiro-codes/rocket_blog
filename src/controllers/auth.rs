use models::account;
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::{Cookie, CookieJar},
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
};
use sea_orm_rocket::Connection;

use crate::{
    pool::Db,
    services::{self, AuthService},
};
/// This Controller also provide the AuthService
pub struct Controller {
    path: String,
}
impl Controller {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[post("/", data = "<data>")]
async fn login(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    jar: &CookieJar<'_>,
    data: Form<account::FormDTO>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    if let Ok(token) = service.login(db, data.into_inner()).await {
        jar.add_private(Cookie::new("token", token.to_string()));
        Flash::success(Redirect::to("/blog"), "Login successful.")
    } else {
        Flash::new(Redirect::to("/blog"), "danger", "Login failed.")
    }

}
#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private(Cookie::from("token"));
    Flash::success(Redirect::to("/blog"), "Logout successful.")
}

fn routes() -> Vec<Route> {
    routes![login, logout]
}

#[rocket::async_trait]
impl Fairing for Controller {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Auth Controller",
            kind: Kind::Ignite,
        }
    }
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket
            .manage(AuthService::new())
            .mount(self.path.to_owned(), routes()))
    }
}
