//! Index controller for handling the main application routing.
//!
//! This controller manages the root route of the application and provides
//! navigation to the main blog interface.

use rocket::{fairing::{Fairing, self, Kind}, Route, response::Redirect, Rocket, Build};

/// Controller for index/home page routing.
///
/// The IndexController handles requests to the root path and redirects users
/// to the main blog listing page, providing a clean entry point to the application.
pub struct Controller {
    /// The base path this controller will handle (typically "/")
    path: String
}

impl Controller {
    /// Creates a new IndexController instance.
    ///
    /// # Arguments
    ///
    /// * `path` - The base path for this controller's routes
    ///
    /// # Returns
    ///
    /// A new Controller instance configured with the specified path.
    pub fn new(path: String) -> Self {
        Self { path }
    }
}
/// Root route handler that redirects to the blog listing.
///
/// This provides a clean entry point to the application by automatically
/// directing users to the main blog content when they visit the root URL.
///
/// # Returns
///
/// A redirect response to "/blog"
#[get("/")]
fn index()->Redirect {
    Redirect::to("/blog")
}

/// Returns the routes handled by this controller.
///
/// # Returns
///
/// A vector of Rocket routes for the index functionality.
pub fn routes() -> Vec<Route> {
    routes![
        index
    ]
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
        Ok(rocket
           .mount(self.path.to_owned(), routes()))
    }
}
