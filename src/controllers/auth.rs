use models::account;
use rocket::{
    fairing::{Fairing, Kind, Info, Result as FairingResult},
    form::Form,
    http::{Cookie, CookieJar},
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
};
use sea_orm_rocket::Connection;

use crate::{
    pool::Db,
    services::AuthService,
    generic::controller,
};

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

// Use the macro to generate the controller boilerplate
controller!(Controller, AuthService, "Auth Controller", routes());
