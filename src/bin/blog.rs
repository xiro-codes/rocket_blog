//! Blog Application Binary
//! 
//! This binary contains the main blog functionality including:
//! - Blog posts and content management
//! - User authentication and settings
//! - Comments and reactions
//! - SEO optimization
//! - AI services integration
//! - Background job processing

use app::{
    features::Features,
    database::parse_database_args_with_fallback,
    controllers,
    services::{
        AuthService, BlogService, CommentService, OpenAIService, OllamaService, 
        AIProviderService, ReactionService, SettingsService, TagService, 
        CoordinatorService, YoutubeDownloadService, BackgroundJobService
    },
    middleware,
    template_config,
    create_base_rocket_with_database
};
use rocket::{fs::FileServer, response::Redirect, Build, Rocket, catchers, catch, launch};

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed - redirecting to home page");
    Redirect::to("/")
}

/// Blog-specific service registry
pub struct BlogServiceRegistry;

impl BlogServiceRegistry {
    pub fn attach_all_services(rocket: Rocket<Build>) -> Rocket<Build> {
        log::info!("Registering blog application services...");
        
        // Create AI provider service and add providers
        log::debug!("Creating AI provider service with OpenAI and Ollama providers");
        let mut ai_service = AIProviderService::new();
        ai_service.add_provider(Box::new(OpenAIService::new()));
        ai_service.add_provider(Box::new(OllamaService::new()));
        
        log::debug!("Attaching blog services: Auth, Blog, Comment, OpenAI, Ollama, AIProvider, Reaction, Settings, Tag, Coordinator, YouTube, BackgroundJob");
        
        rocket
            .manage(AuthService::new())
            .manage(BlogService::new())
            .manage(CommentService::new())
            .manage(OpenAIService::new()) // Keep for backwards compatibility
            .manage(OllamaService::new())
            .manage(ai_service)
            .manage(ReactionService::new())
            .manage(SettingsService::new())
            .manage(TagService::new())
            .manage(CoordinatorService::new())
            .manage(YoutubeDownloadService::new())
            .manage(BackgroundJobService::new())
    }
}

/// Blog-specific controller registry  
pub struct BlogControllerRegistry;

impl BlogControllerRegistry {
    pub fn attach_all_controllers(rocket: Rocket<Build>) -> Rocket<Build> {
        log::info!("Registering blog application controllers...");
        log::debug!("Attaching controllers: Index (/), Auth (/auth), Blog (/blog), Comment (/comment), Feed (/feed), Settings (/settings), SEO (/)");
        
        rocket
            .attach(controllers::IndexController::new("/".to_owned()))
            .attach(controllers::AuthController::new("/auth".to_owned()))
            .attach(controllers::BlogController::new("/blog".to_owned()))
            .attach(controllers::CommentController::new("/comment".to_owned()))
            .attach(controllers::FeedController::new("/feed".to_owned()))
            .attach(controllers::SettingsController::new("/settings".to_owned()))
            .attach(controllers::SeoController::new("/".to_owned()))
    }
}

#[launch]
async fn rocket() -> Rocket<Build> {
    log::info!("Starting Rocket Blog application...");
    log::debug!("Development mode: {}", Features::is_development());
    log::debug!("Seeding enabled: {}", Features::enable_seeding());
    log::debug!("Log level: {:?}", Features::log_level());
    
    // Parse command line arguments for database configuration
    let db_config = parse_database_args_with_fallback();
    log::info!("Database configuration: {:?}", db_config);
    
    // Build the base rocket instance with database auto-detection
    log::info!("Building Blog Rocket instance and configuring database...");
    let mut rocket = create_base_rocket_with_database(db_config).await
        .register("/", catchers![catch_default])
        .attach(template_config::create_template_fairing());
    
    // Attach blog-specific services
    rocket = BlogServiceRegistry::attach_all_services(rocket);
    
    // Only attach seeding in debug builds (development mode)
    if Features::enable_seeding() {
        log::info!("Attaching database seeding middleware");
        rocket = rocket.attach(middleware::Seeding::new(Some(0), 50));
    }
    
    log::info!("Attaching blog controllers and static file server");
    // Attach blog controllers
    BlogControllerRegistry::attach_all_controllers(rocket)
        .mount("/static", FileServer::from("./static/"))
}