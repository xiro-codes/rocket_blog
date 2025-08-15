use rocket::{
    fairing::{self, Fairing, Kind},
    http::{CookieJar, Status},
    request::FlashMessage,
    response::{Flash, Redirect},
    Build, Rocket, Route, State,
};
use sea_orm_rocket::Connection;
use uuid::Uuid;

use crate::{pool::Db, services::AuthService};

/// Generic controller trait for common functionality
pub trait BaseController {
    /// Get the mount path for this controller
    fn path(&self) -> &str;

    /// Get the controller name for fairing info
    fn name(&self) -> &'static str;

    /// Get the routes for this controller
    fn routes() -> Vec<Route>;
}

/// Common controller functionality
pub struct ControllerBase {
    path: String,
}

impl ControllerBase {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    /// Check if user is authenticated and return token
    pub fn check_auth(jar: &CookieJar<'_>) -> Result<Option<String>, Status> {
        Ok(jar.get_private("token").map(|c| c.value().to_owned()))
    }

    /// Require authentication and return token, or return Unauthorized status
    pub fn require_auth(jar: &CookieJar<'_>) -> Result<String, Status> {
        Self::check_auth(jar)?.ok_or(Status::Unauthorized)
    }

    /// Check if user is authenticated and is admin
    pub async fn check_admin_auth(
        conn: Connection<'_, Db>,
        auth_service: &State<AuthService>,
        jar: &CookieJar<'_>,
    ) -> Result<bool, Status> {
        if let Some(token) = Self::check_auth(jar)? {
            let db = conn.into_inner();
            let token_uuid = Uuid::parse_str(&token).map_err(|_| Status::Unauthorized)?;
            if let Some(account) = auth_service.check_token(db, token_uuid).await {
                return Ok(account.admin);
            }
        }
        Ok(false)
    }

    /// Require admin authentication
    pub async fn require_admin_auth(
        conn: Connection<'_, Db>,
        auth_service: &State<AuthService>,
        jar: &CookieJar<'_>,
    ) -> Result<(), Status> {
        if Self::check_admin_auth(conn, auth_service, jar).await? {
            Ok(())
        } else {
            Err(Status::Unauthorized)
        }
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

    /// Extract flash message content for template context
    pub fn extract_flash(flash: Option<FlashMessage<'_>>) -> Option<(String, String)> {
        flash.map(|f| {
            let (kind, message) = f.into_inner();
            (kind.to_string(), message)
        })
    }
}

/// Macro to implement Fairing for controllers with a service
#[macro_export]
macro_rules! impl_controller_fairing {
    ($controller:ty, $service:ty, $name:expr, $routes:expr) => {
        #[rocket::async_trait]
        impl Fairing for $controller {
            fn info(&self) -> fairing::Info {
                fairing::Info {
                    name: $name,
                    kind: Kind::Ignite,
                }
            }

            async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
                Ok(rocket
                    .manage(<$service>::new())
                    .mount(self.base.path(), $routes))
            }
        }
    };
}
