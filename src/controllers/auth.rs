//! Authentication controller for user login and logout functionality.
//!
//! This controller handles user authentication, including login with credentials,
//! session management through secure cookies, and logout functionality.

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

/// Authentication controller for handling user login and logout.
///
/// This controller manages user authentication workflows, including:
/// - User login with credential validation
/// - Session management via secure cookies
/// - User logout with session cleanup
/// - Integration with the AuthService for business logic
pub struct Controller {
    /// The base path for authentication routes (typically "/auth")
    path: String,
}

impl Controller {
    /// Creates a new AuthController instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The base path for authentication routes
    ///
    /// # Returns
    ///
    /// A new Controller instance configured for authentication endpoints.
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

/// Handles user login with email/username and password.
///
/// Authenticates user credentials and creates a secure session cookie upon success.
/// Returns appropriate flash messages for both success and failure cases.
///
/// # Arguments
///
/// * `conn` - Database connection from the connection pool
/// * `service` - AuthService for handling authentication logic
/// * `jar` - Cookie jar for managing session cookies
/// * `data` - Form data containing user credentials
///
/// # Returns
///
/// A Flash redirect response with success or error message
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

/// Handles user logout by clearing the session cookie.
///
/// Removes the authentication token from the user's session and redirects
/// to the blog listing with a success message.
///
/// # Arguments
///
/// * `jar` - Cookie jar for managing session cookies
///
/// # Returns
///
/// A Flash redirect response confirming successful logout
#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private(Cookie::from("token"));
    Flash::success(Redirect::to("/blog"), "Logout successful.")
}

/// Returns the routes handled by this controller.
///
/// # Returns
///
/// A vector of Rocket routes for authentication functionality
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
