use models::dto::{AccountFormDTO, AdminCreateFormDTO};
use rocket::{
    fairing::{self, Fairing, Kind},
    form::Form,
    http::{Cookie, CookieJar, Status},
    response::{Flash, Redirect},
    routes, get, post,
    Build, Rocket, Route, State,
};
use rocket_dyn_templates::{context, Template};
use sea_orm_rocket::Connection;

use super::AuthService;
use crate::database::Db;

/// Base controller functionality for authentication
pub struct ControllerBase;

impl ControllerBase {
    /// Check if user is authenticated and return token
    pub fn check_auth(jar: &CookieJar<'_>) -> Result<Option<String>, Status> {
        Ok(jar.get_private("token").map(|c| c.value().to_owned()))
    }

    /// Require authentication and return token, or return Unauthorized status
    pub fn require_auth(jar: &CookieJar<'_>) -> Result<String, Status> {
        Self::check_auth(jar)?.ok_or(Status::Unauthorized)
    }

    /// Create a success flash redirect
    pub fn success_redirect<T: Into<String>, U: Into<String>>(
        to: T,
        message: U,
    ) -> Flash<Redirect> {
        Flash::success(Redirect::to(to.into()), message.into())
    }

    /// Create a danger flash redirect
    pub fn danger_redirect<T: Into<String>, U: Into<String>>(to: T, message: U) -> Flash<Redirect> {
        Flash::new(Redirect::to(to.into()), "danger", message.into())
    }
}

/// Configuration for the Auth Controller
pub struct AuthControllerConfig {
    pub redirect_after_login: String,
    pub redirect_after_logout: String,
    pub redirect_after_register: String,
    pub template_prefix: String,
}

impl AuthControllerConfig {
    pub fn new(
        redirect_after_login: String,
        redirect_after_logout: String,
        redirect_after_register: String,
    ) -> Self {
        Self {
            redirect_after_login,
            redirect_after_logout,
            redirect_after_register,
            template_prefix: "auth".to_string(), // default to "auth"
        }
    }

    pub fn with_template_prefix(mut self, template_prefix: String) -> Self {
        self.template_prefix = template_prefix;
        self
    }
}

/// Shared Auth Controller that can be configured for different applications
pub struct AuthController {
    path: String,
}

impl AuthController {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Get the mount path for this controller
    pub fn path(&self) -> &str {
        &self.path
    }
}

#[get("/")]
async fn login_view(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    config: &State<AuthControllerConfig>,
) -> Result<Template, Status> {
    log::info!("Route accessed: GET /auth/ - Login page requested");
    let db = conn.into_inner();
    
    // Check if any accounts exist, return 404 if none
    if !service.has_any_accounts(db).await {
        log::info!("No accounts exist in system - login page not available");
        return Err(Status::NotFound);
    }
    
    log::debug!("Login page served successfully");
    Ok(Template::render(
        format!("{}/login", config.template_prefix),
        context! {}
    ))
}

#[post("/", data = "<data>")]
async fn login(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    config: &State<AuthControllerConfig>,
    jar: &CookieJar<'_>,
    data: Form<AccountFormDTO>,
) -> Flash<Redirect> {
    let form_data = data.into_inner();
    log::info!("Route accessed: POST /auth/ - Login attempt for username: {}", form_data.username);
    
    let db = conn.into_inner();
    if let Ok(token) = service.login(db, form_data).await {
        jar.add_private(Cookie::new("token", token.to_string()));
        log::info!("Login successful - redirecting");
        ControllerBase::success_redirect(&config.redirect_after_login, "Login successful.")
    } else {
        log::warn!("Login failed - redirecting back to login form");
        ControllerBase::danger_redirect("/auth/", "Login failed.")
    }
}

#[get("/logout")]
async fn logout(config: &State<AuthControllerConfig>, jar: &CookieJar<'_>) -> Flash<Redirect> {
    log::info!("Route accessed: GET /auth/logout - User logout requested");
    jar.remove_private(Cookie::from("token"));
    log::info!("User logged out successfully");
    ControllerBase::success_redirect(&config.redirect_after_logout, "Logout successful.")
}

#[get("/create-admin")]
async fn create_admin_view(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    config: &State<AuthControllerConfig>,
) -> Result<Template, Status> {
    log::info!("Route accessed: GET /auth/create-admin - Create admin page requested");
    let db = conn.into_inner();
    
    // Check if any accounts exist, redirect if they do
    if service.has_any_accounts(db).await {
        log::info!("Admin creation blocked - accounts already exist");
        return Err(Status::NotFound);
    }
    
    log::debug!("Create admin page served successfully");
    Ok(Template::render(
        format!("{}/create_admin", config.template_prefix),
        context! {}
    ))
}

#[post("/create-admin", data = "<data>")]
async fn create_admin(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    config: &State<AuthControllerConfig>,
    _jar: &CookieJar<'_>,
    data: Form<AdminCreateFormDTO>,
) -> Flash<Redirect> {
    let form_data = data.into_inner();
    log::info!("Route accessed: POST /auth/create-admin - Admin account creation attempt for username: {}", form_data.username);
    
    let db = conn.into_inner();
    
    match service.create_admin_account(db, form_data).await {
        Ok(account) => {
            log::info!("Admin account creation successful for: {} ({})", account.username, account.id);
            ControllerBase::success_redirect(&config.redirect_after_login, "Admin account created successfully! You can now log in.")
        }
        Err(e) => {
            log::error!("Admin account creation failed: {}", e);
            ControllerBase::danger_redirect("/auth/create-admin", "Failed to create admin account. It may already exist.")
        }
    }
}

#[get("/register")]
async fn register_view(config: &State<AuthControllerConfig>) -> Template {
    log::info!("Route accessed: GET /auth/register - User registration page requested");
    Template::render(
        format!("{}/register", config.template_prefix),
        context! {}
    )
}

#[post("/register", data = "<data>")]
async fn register(
    conn: Connection<'_, Db>,
    service: &State<AuthService>,
    config: &State<AuthControllerConfig>,
    _jar: &CookieJar<'_>,
    data: Form<AccountFormDTO>,
) -> Flash<Redirect> {
    let form_data = data.into_inner();
    log::info!("Route accessed: POST /auth/register - User account registration attempt for username: {}", form_data.username);
    
    let db = conn.into_inner();
    
    match service.create_user_account(db, form_data).await {
        Ok(account) => {
            log::info!("User account registration successful for: {} ({})", account.username, account.id);
            ControllerBase::success_redirect(&config.redirect_after_register, "Account created successfully! You can now log in.")
        }
        Err(e) => {
            log::error!("User account registration failed: {}", e);
            ControllerBase::danger_redirect("/auth/register", "Failed to create account. Username may already exist.")
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![login_view, login, logout, create_admin_view, create_admin, register_view, register]
}

#[rocket::async_trait]
impl Fairing for AuthController {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Shared Auth Controller",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket.mount(self.path(), routes()))
    }
}