use models::dto::{AccountFormDTO, AdminCreateFormDTO};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::{Cookie, CookieJar, Status},
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;

use crate::{
    controllers::base::ControllerBase,
    pool::Db,
    services::AuthService,
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

#[get("/")]
async fn login_view(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    // Check if any accounts exist, return 404 if none
    if !service.has_any_accounts(db).await {
        return Err(Status::NotFound);
    }
    
    Ok(Template::render(
        "auth/login",
        context! {}
    ))
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
        ControllerBase::success_redirect("/blog/", "Login successful.")
    } else {
        ControllerBase::danger_redirect("/auth/", "Login failed.")
    }
}

#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private(Cookie::from("token"));
    ControllerBase::success_redirect("/blog/", "Logout successful.")
}

#[get("/create-admin")]
async fn create_admin_view(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
) -> Result<Template, Status> {
    let db = conn.into_inner();
    
    // Check if any accounts exist, redirect if they do
    if service.has_any_accounts(db).await {
        return Err(Status::NotFound);
    }
    
    Ok(Template::render(
        "auth/create_admin",
        context! {}
    ))
}

#[post("/create-admin", data = "<data>")]
async fn create_admin(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    _jar: &CookieJar<'_>,
    data: Form<AdminCreateFormDTO>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    
    match service.create_admin_account(db, data.into_inner()).await {
        Ok(_) => {
            ControllerBase::success_redirect("/blog", "Admin account created successfully! You can now log in.")
        }
        Err(_) => {
            ControllerBase::danger_redirect("/auth/create-admin", "Failed to create admin account. It may already exist.")
        }
    }
}

fn routes() -> Vec<Route> {
    routes![login_view, login, logout, create_admin_view, create_admin]
}

crate::impl_controller_routes!(Controller, "Auth Controller", routes());
