use models::{account, dto::AccountFormDTO};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::{Cookie, CookieJar},
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
};
use sea_orm_rocket::Connection;

use crate::{
    controllers::base::ControllerBase,
    pool::Db,
    services::{self, AuthService},
};

/// This Controller also provides the AuthService
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

#[post("/", data = "<data>")]
async fn login(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    jar: &CookieJar<'_>,
    data: Form<AccountFormDTO>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    if let Ok(token) = service.login(db, data.into_inner()).await {
        jar.add_private(Cookie::new("token", token.to_string()));
        ControllerBase::success_redirect("/blog", "Login successful.")
    } else {
        ControllerBase::danger_redirect("/blog", "Login failed.")
    }
}

#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private(Cookie::from("token"));
    ControllerBase::success_redirect("/blog", "Logout successful.")
}

fn routes() -> Vec<Route> {
    routes![login, logout]
}

crate::impl_controller_fairing!(Controller, AuthService, "Auth Controller", routes());
