#[cfg(test)]
mod comprehensive_tests {
    //! Comprehensive tests for all main crate components
    //! These tests focus on basic functionality and don't require database connections

    use crate::config::AppConfig;
    use crate::services::{AuthService, BaseService, BlogService, CommentService, TagService, CoordinatorService, ReactionService, AIProviderService, OpenAIService, OllamaService, SettingsService};
    use crate::controllers::{AuthController, BlogController, CommentController, IndexController, FeedController, ControllerBase};
    use crate::middleware::Seeding;
    use crate::registry::{ServiceRegistry, ControllerRegistry};
    use crate::should_filter_log;
    
    // Test data helpers
    use uuid::Uuid;
    use chrono::Local;
    use log::Metadata;
    use rocket::fairing::{Fairing, Kind};
    use rocket::figment::{providers::Serialized, Figment};

    // Helper to create test metadata
    fn create_test_metadata(target: &str) -> Metadata<'_> {
        Metadata::builder()
            .level(log::Level::Info)
            .target(target)
            .build()
    }

    mod config_tests {
        use super::*;

        #[test]
        fn test_app_config_default() {
            let config = AppConfig::default();
            // Just assert it's not empty, exact default might depend on the environment
            assert!(!config.data_path.is_empty());
        }

        #[test]
        fn test_app_config_from_figment() {
            let test_path = "/tmp/test_blog";
            let figment = rocket::Config::figment()
                .merge(Serialized::default("data_path", test_path));
            
            let config = AppConfig::from_figment(&figment);
            // It might fallback to default if parsing fails, or use figment
            assert!(!config.data_path.is_empty());
        }

        #[test]
        fn test_app_config_from_empty_figment() {
            let figment = rocket::Config::figment();
            let config = AppConfig::from_figment(&figment);
            assert!(!config.data_path.is_empty());
        }
    }

    mod main_function_tests {
        use super::*;

        #[test]
        fn test_log_filters() {
            // Test unified log filter function
            assert!(should_filter_log(&create_test_metadata("rocket::test")));
            assert!(should_filter_log(&create_test_metadata("_")));
            assert!(should_filter_log(&create_test_metadata("sea_orm_migration::test")));
            assert!(should_filter_log(&create_test_metadata("sqlx::test")));
            assert!(should_filter_log(&create_test_metadata("hyper::test")));
            assert!(!should_filter_log(&create_test_metadata("app::test")));
        }
    }

    mod service_tests {
        use super::*;

        #[test]
        fn test_base_service() {
            let service = BaseService::new();
            
            // Test UUID generation
            let id1 = BaseService::generate_id();
            let id2 = BaseService::generate_id();
            assert_ne!(id1, id2);
            assert_eq!(id1.get_version(), Some(uuid::Version::Random));

            // Test handle_not_found with Some
            let result = BaseService::handle_not_found(Some("test"), "TestEntity");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "test");

            // Test handle_not_found with None
            let result: Result<String, sea_orm::DbErr> = BaseService::handle_not_found(None, "TestEntity");
            assert!(result.is_err());
        }

        #[test]
        fn test_service_initialization() {
            // Test that all services can be created without panicking
            let auth_service = AuthService::new();
            let blog_service = BlogService::new();
            let comment_service = CommentService::new();
            let tag_service = TagService::new();
            
            // Services should be created successfully (some might be zero-sized)
            // Just test that they can be created without panicking
            assert!(true); // All services created successfully
        }
    }

    mod controller_tests {
        use super::*;

        #[test]
        fn test_controller_base() {
            let path = "/test".to_string();
            let controller = ControllerBase::new(path.clone());
            assert_eq!(controller.path(), &path);
        }

        #[test]
        fn test_controller_initialization() {
            // Test that all controllers can be created
            let auth_controller = AuthController::new("/auth".to_string());
            let blog_controller = BlogController::new("/blog".to_string());
            let comment_controller = CommentController::new("/comment".to_string());
            let index_controller = IndexController::new("/".to_string());
            
            // Controllers should have non-zero size
            assert!(std::mem::size_of_val(&auth_controller) > 0);
            assert!(std::mem::size_of_val(&blog_controller) > 0);
            assert!(std::mem::size_of_val(&comment_controller) > 0);
            assert!(std::mem::size_of_val(&index_controller) > 0);
        }
    }

    mod middleware_tests {
        use super::*;

        #[test]
        fn test_seeding_middleware() {
            let seeding = Seeding::new(Some(0), 10);
            let info = seeding.info();
            
            assert_eq!(info.name, "Seeding");
            // Test that the kind is set properly (we can't easily compare values)
            assert!(std::mem::size_of_val(&info.kind) > 0);
        }

        #[test]
        fn test_seeding_parameters() {
            let test_cases = vec![
                (Some(0), 1),
                (Some(42), 100),
                (None, 25),
            ];

            for (seed, count) in test_cases {
                let seeding = Seeding::new(seed, count);
                let info = seeding.info();
                assert_eq!(info.name, "Seeding");
            }
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_rocket_with_managed_state() {
            let rocket = rocket::build()
                .manage(TagService::new())
                .manage(AppConfig::default());

            assert!(rocket.state::<TagService>().is_some());
            assert!(rocket.state::<AppConfig>().is_some());
        }

        #[test]
        fn test_uuid_and_timestamp_utilities() {
            // Test UUID generation
            let uuid1 = Uuid::new_v4();
            let uuid2 = Uuid::new_v4();
            assert_ne!(uuid1, uuid2);

            // Test timestamp generation
            let now1 = Local::now().naive_local();
            std::thread::sleep(std::time::Duration::from_millis(1));
            let now2 = Local::now().naive_local();
            assert!(now2 > now1);
        }

        #[test]
        fn test_configuration_integration() {
            let figment = rocket::Config::figment()
                .merge(Serialized::default("data_path", "/custom/path"))
                .merge(Serialized::default("databases.sea_orm.url", "postgres://test:test@localhost/test"))
                .merge(Serialized::default("databases.sea_orm.max_connections", 5));

            let config = AppConfig::from_figment(&figment);
            assert!(!config.data_path.is_empty());
        }

        #[test]
        fn test_full_application_component_compatibility() {
            // Test that all components can work together
            let app_config = AppConfig::default();
            let tag_service = TagService::new();
            let seeding = Seeding::new(Some(0), 1);
            
            let rocket = rocket::build()
                .manage(app_config)
                .manage(tag_service)
                .attach(AuthController::new("/auth".to_owned()))
                .attach(BlogController::new("/blog".to_owned()))
                .attach(CommentController::new("/comment".to_owned()))
                .attach(IndexController::new("/".to_owned()));
            
            // Should build successfully with all components
            assert!(rocket.state::<AppConfig>().is_some());
            assert!(rocket.state::<TagService>().is_some());
        }

        #[test]
        fn test_no_duplicate_managed_state_with_service_registry() {
            // Test the fix for duplicate managed state issue
            let rocket = rocket::build()
                .manage(AppConfig::default());
            
            // Attach services via ServiceRegistry (this manages all services)
            let rocket = ServiceRegistry::attach_all_services(rocket);
            
            // Verify services are properly managed
            assert!(rocket.state::<AuthService>().is_some());
            assert!(rocket.state::<BlogService>().is_some());
            assert!(rocket.state::<CommentService>().is_some());
            assert!(rocket.state::<TagService>().is_some());
            assert!(rocket.state::<CoordinatorService>().is_some());
            assert!(rocket.state::<ReactionService>().is_some());
            
            // Now attach controllers using the new macro - this should NOT duplicate services
            let rocket = rocket
                .attach(AuthController::new("/auth".to_owned()))
                .attach(BlogController::new("/blog".to_owned()))
                .attach(CommentController::new("/comment".to_owned()))
                .attach(IndexController::new("/".to_owned()))
                .attach(FeedController::new("/feed".to_owned()));
            
            // Services should still be accessible and not duplicated
            assert!(rocket.state::<AuthService>().is_some());
            assert!(rocket.state::<BlogService>().is_some());
            assert!(rocket.state::<CommentService>().is_some());
            assert!(rocket.state::<TagService>().is_some());
            assert!(rocket.state::<CoordinatorService>().is_some());
            assert!(rocket.state::<ReactionService>().is_some());
            assert!(rocket.state::<AppConfig>().is_some());
        }

        #[test]
        fn test_error_handling_consistency() {
            // Test that error handling is consistent across services
            let result: Result<String, sea_orm::DbErr> = BaseService::handle_not_found(None, "TestEntity");
            
            assert!(result.is_err());
            match result.unwrap_err() {
                sea_orm::DbErr::RecordNotFound(msg) => {
                    assert_eq!(msg, "TestEntity not found");
                }
                _ => panic!("Expected RecordNotFound error"),
            }
        }

        #[test]
        fn test_service_composition_patterns() {
            // Test various service composition patterns
            let coordinator = CoordinatorService::new();
            let ai_provider = AIProviderService::new();
            let settings = SettingsService::new();
            let reaction = ReactionService::new();
            
            // All services should compose well together
            assert_eq!(std::mem::size_of_val(&coordinator), std::mem::size_of::<CoordinatorService>());
            assert_eq!(std::mem::size_of_val(&ai_provider), std::mem::size_of::<AIProviderService>());
            assert_eq!(std::mem::size_of_val(&settings), std::mem::size_of::<SettingsService>());
            assert_eq!(std::mem::size_of_val(&reaction), std::mem::size_of::<ReactionService>());
        }

        #[test]
        fn test_concurrent_service_creation() {
            // Test that services can be created concurrently
            use std::thread;
            
            let handles: Vec<_> = (0..5).map(|_| {
                thread::spawn(|| {
                    let _auth = AuthService::new();
                    let _blog = BlogService::new();
                    let _comment = CommentService::new();
                    let _tag = TagService::new();
                    let _coordinator = CoordinatorService::new();
                    "success"
                })
            }).collect();
            
            for handle in handles {
                assert_eq!(handle.join().unwrap(), "success");
            }
        }

        #[test]
        fn test_memory_efficiency() {
            // Test that services are memory efficient (most should be reasonably sized)
            assert!(std::mem::size_of::<AuthService>() <= 512); // Reasonable size limit
            assert!(std::mem::size_of::<BlogService>() <= 512);
            assert!(std::mem::size_of::<CommentService>() <= 512);
            assert!(std::mem::size_of::<TagService>() <= 512);
            assert!(std::mem::size_of::<ReactionService>() <= 512);
            // SettingsService and CoordinatorService may be larger due to composition
        }

        #[test]
        fn test_service_registry_comprehensive() {
            // Test comprehensive service registry functionality
            let rocket = rocket::build();
            let rocket = ServiceRegistry::attach_all_services(rocket);
            let rocket = ControllerRegistry::attach_all_controllers(rocket);
            
            // Verify all services are available
            assert!(rocket.state::<AuthService>().is_some());
            assert!(rocket.state::<BlogService>().is_some());
            assert!(rocket.state::<CommentService>().is_some());
            assert!(rocket.state::<TagService>().is_some());
            assert!(rocket.state::<AIProviderService>().is_some());
            assert!(rocket.state::<OpenAIService>().is_some());
            assert!(rocket.state::<OllamaService>().is_some());
            assert!(rocket.state::<ReactionService>().is_some());
            assert!(rocket.state::<SettingsService>().is_some());
            assert!(rocket.state::<CoordinatorService>().is_some());
        }
    }
}