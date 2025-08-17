use rocket::{
    http::{CookieJar, Status},
    request::FlashMessage,
    response::{Flash, Redirect},
    Build, Rocket, State,
    fairing::{self, Fairing, Kind},
};
use sea_orm_rocket::Connection;

use crate::{pool::Db, services::CoordinatorService};

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

    /// Check if user is authenticated and is admin using coordinator service
    #[allow(dead_code)]
    pub async fn check_admin_auth(
        conn: Connection<'_, Db>,
        coordinator: &State<CoordinatorService>,
        jar: &CookieJar<'_>,
    ) -> Result<bool, Status> {
        let token = Self::check_auth(jar)?;
        let db = conn.into_inner();
        Ok(coordinator.is_admin(db, token.as_deref()).await)
    }

    /// Require admin authentication using coordinator service
    #[allow(dead_code)]
    pub async fn require_admin_auth(
        conn: Connection<'_, Db>,
        coordinator: &State<CoordinatorService>,
        jar: &CookieJar<'_>,
    ) -> Result<(), Status> {
        let token = Self::check_auth(jar)?;
        let db = conn.into_inner();
        coordinator.require_admin(db, token.as_deref()).await
            .map_err(|_| Status::Unauthorized)?;
        Ok(())
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

/// Macro to implement Fairing for controllers with a service (deprecated - use impl_controller_routes!)
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

/// Macro to implement Fairing for controllers that only mount routes (services managed by ServiceRegistry)
#[macro_export]
macro_rules! impl_controller_routes {
    ($controller:ty, $name:expr, $routes:expr) => {
        #[rocket::async_trait]
        impl Fairing for $controller {
            fn info(&self) -> fairing::Info {
                fairing::Info {
                    name: $name,
                    kind: Kind::Ignite,
                }
            }

            async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
                Ok(rocket.mount(self.base.path(), $routes))
            }
        }
    };
}
