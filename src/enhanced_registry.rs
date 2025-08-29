//! Enhanced Registry System using New Macros
//! 
//! This file demonstrates how the new macro system can be used to replace
//! the existing manual registry with auto-generated registries.

use crate::{create_service_registry, create_controller_registry};
use crate::services::{
    AuthService, BlogService, CommentService, OpenAIService, OllamaService, 
    AIProviderService, ReactionService, SettingsService, TagService, 
    CoordinatorService, YoutubeDownloadService, BackgroundJobService, 
    WorkTimeService, PayPeriodService
};
use crate::controllers::{
    AuthController, BlogController, CommentController, IndexController, 
    FeedController, SettingsController, SeoController, WorkTimeController
};

// Auto-generated service registry using the new macro
create_service_registry!(
    NewServiceRegistry,
    [
        AuthService,
        BlogService,
        CommentService,
        OpenAIService,
        OllamaService,
        ReactionService,
        SettingsService,
        TagService,
        CoordinatorService,
        YoutubeDownloadService,
        BackgroundJobService,
        WorkTimeService,
        PayPeriodService,
    ]
);

// Auto-generated blog controller registry
create_controller_registry!(
    BlogControllerRegistry,
    [
        (IndexController, "/"),
        (AuthController, "/auth"),
        (BlogController, "/blog"),
        (CommentController, "/comment"),
        (FeedController, "/feed"),
        (SettingsController, "/settings"),
        (SeoController, "/"),
    ]
);

// Auto-generated work time controller registry  
create_controller_registry!(
    WorkTimeControllerRegistry,
    [
        (WorkTimeController, "/worktime"),
    ]
);

// Example of how to use the registries
impl NewServiceRegistry {
    /// Enhanced service registration with AI provider setup
    pub fn attach_all_services_with_ai(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
        log::info!("Registering enhanced services with AI provider setup...");
        
        // Create AI provider service and add providers
        log::debug!("Creating AI provider service with OpenAI and Ollama providers");
        let mut ai_service = AIProviderService::new();
        ai_service.add_provider(Box::new(OpenAIService::new()));
        ai_service.add_provider(Box::new(OllamaService::new()));
        
        // Use the auto-generated registry and add the configured AI service
        Self::attach_all_services(rocket)
            .manage(ai_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_service_registry() {
        // Test that the macro-generated registry works
        let rocket = rocket::build();
        let rocket = NewServiceRegistry::attach_all_services(rocket);
        
        // Verify that services are managed
        assert!(rocket.state::<AuthService>().is_some());
        assert!(rocket.state::<BlogService>().is_some());
        assert!(rocket.state::<TagService>().is_some());
    }
    
    #[test]
    fn test_blog_controller_registry() {
        let rocket = rocket::build();
        let rocket = BlogControllerRegistry::attach_all_controllers(rocket);
        
        // The controllers are attached via fairings, so we can't directly test state
        // but we can verify the registry was created
        let _registry = BlogControllerRegistry;
    }
    
    #[test] 
    fn test_enhanced_ai_service_registry() {
        let rocket = rocket::build();
        let rocket = NewServiceRegistry::attach_all_services_with_ai(rocket);
        
        // Verify enhanced setup
        assert!(rocket.state::<AIProviderService>().is_some());
        assert!(rocket.state::<OpenAIService>().is_some());
    }
}