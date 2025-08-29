use models::dto::AccountFormDTO;
use rocket::{
    fairing::{self, Fairing, Info, Kind},
    form::Form,
    http::{Cookie, CookieJar, Status},
    response::{Flash, Redirect},
    routes,
    Build, Rocket, Route, State, get, post,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;

use app::{
    pool::Db,
    services::AuthService,
};

/// Worktime-specific authentication controller
pub struct WorkTimeAuthController {
    pub path: String,
}

impl WorkTimeAuthController {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    fn routes(&self) -> Vec<Route> {
        routes![
            worktime_login_view,
            worktime_login,
            worktime_logout,
            worktime_register_view,
            worktime_register
        ]
    }
}

#[rocket::async_trait]
impl Fairing for WorkTimeAuthController {
    fn info(&self) -> Info {
        Info {
            name: "WorkTime Auth Controller",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket.mount(&self.path, self.routes()))
    }
}

#[get("/")]
async fn worktime_login_view(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
) -> Result<Template, Status> {
    log::info!("Route accessed: GET /auth/ - Worktime login page requested");
    let db = conn.into_inner();
    
    // Check if any accounts exist, return 404 if none
    if !service.has_any_accounts(db).await {
        log::info!("No accounts exist in system - worktime login page not available");
        return Err(Status::NotFound);
    }
    
    log::debug!("Worktime login page served successfully");
    Ok(Template::render(
        "worktime/login",
        context! {}
    ))
}

#[post("/", data = "<form>")]
async fn worktime_login(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    form: Form<AccountFormDTO>,
    cookies: &CookieJar<'_>,
) -> Flash<Redirect> {
    let form_data = form.into_inner();
    log::info!("Route accessed: POST /auth/ - Worktime login attempt for username: {}", form_data.username);
    
    let db = conn.into_inner();
    if let Ok(token) = service.login(db, form_data).await {
        cookies.add_private(Cookie::new("token", token.to_string()));
        log::info!("Worktime authentication successful - Redirecting to worktime dashboard");
        Flash::success(
            Redirect::to("/worktime"),
            "Login successful! Welcome to your work time tracker.",
        )
    } else {
        log::warn!("Worktime authentication failed - Redirecting back to login form");
        Flash::error(
            Redirect::to("/auth"),
            "Invalid username or password. Please try again.",
        )
    }
}

#[get("/logout")]
async fn worktime_logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    log::info!("Route accessed: GET /auth/logout - Worktime user logout requested");
    
    cookies.remove_private(Cookie::from("token"));
    
    log::debug!("Worktime user successfully logged out - Redirecting to worktime login");
    Flash::success(
        Redirect::to("/auth"),
        "Logout successful.",
    )
}

#[get("/register")]
async fn worktime_register_view() -> Template {
    log::info!("Route accessed: GET /auth/register - Worktime user registration page requested");
    Template::render(
        "worktime/register",
        context! {}
    )
}

#[post("/register", data = "<form>")]
async fn worktime_register(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    form: Form<AccountFormDTO>,
) -> Flash<Redirect> {
    let form_data = form.into_inner();
    let username = form_data.username.clone();
    log::info!("Route accessed: POST /auth/register - Worktime user account registration attempt for username: {}", username);
    
    let db = conn.into_inner();
    match service.create_user_account(db, form_data).await {
        Ok(account) => {
            log::info!("Worktime user account created successfully for username: {} - Redirecting to worktime login", account.username);
            Flash::success(
                Redirect::to("/auth"),
                "Account created successfully! You can now log in to access the work time tracker.",
            )
        }
        Err(e) => {
            log::warn!("Failed to create worktime user account for username: {} - Error: {}", username, e);
            Flash::error(
                Redirect::to("/auth/register"),
                "Failed to create account. Username may already exist or be invalid.",
            )
        }
    }
}