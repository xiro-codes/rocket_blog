use crate::services::{AuthService, BlogService, CommentService, OpenAIService, ReactionService, SettingsService, TagService, CoordinatorService};
use crate::controllers;
use crate::config::AppConfig;
use rocket::{fairing::AdHoc, Build, Rocket, State};

/// Service registry for managing application services
pub struct ServiceRegistry;

impl ServiceRegistry {
    /// Register all application services with Rocket
    pub fn attach_all_services(rocket: Rocket<Build>) -> Rocket<Build> {
        rocket
            .manage(AuthService::new())
            .manage(BlogService::new())
            .manage(CommentService::new())
            .manage(OpenAIService::new()) // OpenAI will read from database now
            .manage(ReactionService::new())
            .manage(SettingsService::new())
            .manage(TagService::new())
            .manage(CoordinatorService::new())
    }
    
    /// Create a fairing that initializes services
    pub fn fairing() -> AdHoc {
        AdHoc::on_ignite("Service Registry", |rocket| async {
            Self::attach_all_services(rocket)
        })
    }
}

/// Controller registry for managing application controllers
pub struct ControllerRegistry;

impl ControllerRegistry {
    /// Attach all application controllers to Rocket
    pub fn attach_all_controllers(rocket: Rocket<Build>) -> Rocket<Build> {
        rocket
            .attach(controllers::IndexController::new("/".to_owned()))
            .attach(controllers::AuthController::new("/auth".to_owned()))
            .attach(controllers::BlogController::new("/blog".to_owned()))
            .attach(controllers::CommentController::new("/comment".to_owned()))
            .attach(controllers::FeedController::new("/feed".to_owned()))
            .attach(controllers::SettingsController::new("/settings".to_owned()))
    }
    
    /// Create a fairing that initializes controllers
    pub fn fairing() -> AdHoc {
        AdHoc::on_ignite("Controller Registry", |rocket| async {
            Self::attach_all_controllers(rocket)
        })
    }
}