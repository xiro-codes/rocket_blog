#[cfg(test)]
mod integration_tests {
    use crate::{config::AppConfig, services::TagService};
    use rocket::figment::{providers::Serialized, Figment};

    #[test]
    fn test_app_config_integration() {
        let figment = rocket::Config::figment()
            .merge(Serialized::default("data_path", "/tmp/test_blog"));
            
        let config = AppConfig::from_figment(&figment);
        assert!(!config.data_path.is_empty());
    }

    #[test]
    fn test_services_initialization() {
        // Test that all services can be created
        let tag_service = TagService::new();
        let app_config = AppConfig::default();
        
        // Services should initialize without panicking (some might be zero-sized)
        assert!(true); // Services created successfully
        assert!(!app_config.data_path.is_empty());
    }

    #[test]
    fn test_rocket_configuration() {
        // Test basic rocket configuration
        let rocket = rocket::build()
            .manage(TagService::new())
            .manage(AppConfig::default());

        // Should have managed state
        assert!(rocket.state::<TagService>().is_some());
        assert!(rocket.state::<AppConfig>().is_some());
    }

    #[test]
    fn test_uuid_generation() {
        use uuid::Uuid;
        
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.get_version(), Some(uuid::Version::Random));
        assert_eq!(id2.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_chrono_timestamp() {
        use chrono::Local;
        
        let now1 = Local::now().naive_local();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let now2 = Local::now().naive_local();
        
        assert!(now2 > now1);
    }
}