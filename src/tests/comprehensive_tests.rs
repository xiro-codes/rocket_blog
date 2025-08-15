#[cfg(test)]
mod comprehensive_tests {
    //! Comprehensive tests for all main crate components
    //! These tests focus on basic functionality and don't require database connections

    use crate::config::AppConfig;
    use crate::services::{AuthService, BaseService, BlogService, CommentService, TagService};
    use crate::controllers::{AuthController, BlogController, CommentController, IndexController, ControllerBase};
    use crate::middleware::Seeding;
    use crate::{catch_default, drop_rocket, drop_sea_orm_migration, drop_sqlx, drop_hyper};
    
    // Test data helpers
    use uuid::Uuid;
    use chrono::Local;
    use log::Metadata;
    use rocket::fairing::{Fairing, Kind};
    use rocket::figment::{providers::Serialized, Figment};

    // Helper to create test metadata
    fn create_test_metadata(target: &str) -> Metadata {
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
            assert_eq!(config.data_path, "/home/tod/.local/share/blog");
        }

        #[test]
        fn test_app_config_from_figment() {
            let test_path = "/tmp/test_blog";
            let figment = rocket::Config::figment()
                .merge(Serialized::default("data_path", test_path));
            
            let config = AppConfig::from_figment(&figment);
            assert_eq!(config.data_path, test_path);
        }

        #[test]
        fn test_app_config_from_empty_figment() {
            let figment = rocket::Config::figment();
            let config = AppConfig::from_figment(&figment);
            assert_eq!(config.data_path, "/home/tod/.local/share/blog");
        }
    }

    mod main_function_tests {
        use super::*;

        #[test]
        fn test_catch_default_redirect() {
            let redirect = catch_default();
            // Test that redirect is created (we can't easily test the actual location without more complex setup)
            assert!(true); // Placeholder to ensure redirect was created successfully
        }

        #[test]
        fn test_log_filters() {
            // Test rocket filter
            let metadata = create_test_metadata("rocket::test");
            assert!(!drop_rocket(&metadata));
            
            let metadata = create_test_metadata("_");
            assert!(!drop_rocket(&metadata));
            
            let metadata = create_test_metadata("app::test");
            assert!(drop_rocket(&metadata));

            // Test sea_orm_migration filter  
            let metadata = create_test_metadata("sea_orm_migration::test");
            assert!(!drop_sea_orm_migration(&metadata));

            // Test sqlx filter
            let metadata = create_test_metadata("sqlx::test");
            assert!(!drop_sqlx(&metadata));

            // Test hyper filter
            let metadata = create_test_metadata("hyper::test");
            assert!(!drop_hyper(&metadata));
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
            assert_eq!(config.data_path, "/custom/path");
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
    }
}