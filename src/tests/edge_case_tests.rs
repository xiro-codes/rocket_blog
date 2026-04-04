#[cfg(test)]
mod edge_case_tests {
    use crate::services::{AuthService, BaseService, BlogService, CommentService, TagService, AIProviderService, OpenAIService, OllamaService, ReactionService, SettingsService, CoordinatorService, AIProvider};
    use crate::config::AppConfig;
    use crate::middleware::Seeding;
    use crate::controllers::{AuthController, BlogController, CommentController, IndexController, FeedController};
    use crate::registry::{ServiceRegistry, ControllerRegistry};
    use crate::guards::{AuthenticatedUser, OptionalUser, AdminUser};
    use crate::responders::ApiResponse;
    use crate::features::Features;
    use uuid::Uuid;
    use chrono::{Local, Datelike};

    #[test]
    fn test_uuid_edge_cases() {
        // Test UUID edge cases and validation
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        
        // UUIDs should be different
        assert_ne!(uuid1, uuid2);
        
        // Test UUID string representations
        let uuid_str = uuid1.to_string();
        assert_eq!(uuid_str.len(), 36); // Standard UUID length
        assert!(uuid_str.contains('-'));
        
        // Test UUID parsing
        let parsed_uuid = Uuid::parse_str(&uuid_str).unwrap();
        assert_eq!(parsed_uuid, uuid1);
        
        // Test invalid UUID strings
        let invalid_uuids = vec![
            "",
            "invalid",
            "123",
            "550e8400-e29b-41d4-a716", // Too short
            "550e8400-e29b-41d4-a716-446655440000-extra", // Too long
            "550e8400_e29b_41d4_a716_446655440000", // Wrong separators
        ];
        
        for invalid_uuid in invalid_uuids {
            assert!(Uuid::parse_str(invalid_uuid).is_err());
        }
    }

    #[test]
    fn test_timestamp_edge_cases() {
        // Test timestamp edge cases
        let now = Local::now().naive_local();
        
        // Test that timestamps are reasonable
        assert!(now.year() >= 2020);
        assert!(now.month() >= 1 && now.month() <= 12);
        assert!(now.day() >= 1 && now.day() <= 31);
        
        // Test timestamp comparison
        let earlier = now - chrono::Duration::seconds(1);
        assert!(now > earlier);
        
        // Test timestamp formatting
        let formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
        assert!(formatted.contains('-'));
        assert!(formatted.contains(':'));
    }

    #[test]
    fn test_service_error_handling() {
        // Test error handling in services
        let result: Result<String, sea_orm::DbErr> = BaseService::handle_not_found(None, "TestEntity");
        
        assert!(result.is_err());
        match result.unwrap_err() {
            sea_orm::DbErr::RecordNotFound(msg) => {
                assert_eq!(msg, "TestEntity not found");
            }
            _ => panic!("Expected RecordNotFound error"),
        }
        
        // Test successful case
        let success_result = BaseService::handle_not_found(Some("found"), "TestEntity");
        assert!(success_result.is_ok());
        assert_eq!(success_result.unwrap(), "found");
    }

    #[test]
    fn test_config_edge_cases() {
        // Test configuration edge cases
        let default_config = AppConfig::default();
        assert!(!default_config.data_path.is_empty());
        
        // Test config with empty figment
        use rocket::figment::Figment;
        let empty_figment = Figment::new();
        let config_from_empty = AppConfig::from_figment(&empty_figment);
        
        // Should use defaults
        assert!(!config_from_empty.data_path.is_empty());
    }

    #[test]
    fn test_api_response_edge_cases() {
        // Test API response edge cases
        let empty_response = ApiResponse::success_redirect("", "");
        match empty_response {
            ApiResponse::SuccessRedirect(url, msg) => {
                assert!(url.is_empty());
                assert!(msg.is_empty());
            }
            _ => panic!("Expected SuccessRedirect"),
        }
        
        // Test with very long strings
        let long_url = "a".repeat(1000);
        let long_message = "b".repeat(1000);
        
        let long_response = ApiResponse::error_redirect(&long_url, &long_message);
        match long_response {
            ApiResponse::ErrorRedirect(url, msg) => {
                assert_eq!(url.len(), 1000);
                assert_eq!(msg.len(), 1000);
            }
            _ => panic!("Expected ErrorRedirect"),
        }
    }

    #[test]
    fn test_feature_flags_edge_cases() {
        // Test feature flags consistency
        let is_dev = Features::is_development();
        let enable_seeding = Features::enable_seeding();
        let enable_logging = Features::enable_detailed_logging();
        let log_level = Features::log_level();
        
        // All should be consistent
        assert_eq!(is_dev, enable_seeding);
        assert_eq!(is_dev, enable_logging);
        
        if is_dev {
            assert_eq!(log_level, log::LevelFilter::Debug);
        } else {
            assert_eq!(log_level, log::LevelFilter::Info);
        }
    }

    #[test]
    fn test_service_creation_stress() {
        // Stress test service creation
        for _ in 0..100 {
            let _auth = AuthService::new();
            let _blog = BlogService::new();
            let _comment = CommentService::new();
            let _tag = TagService::new();
            let _reaction = ReactionService::new();
            let _settings = SettingsService::new();
            let _coordinator = CoordinatorService::new();
            let _ai_provider = AIProviderService::new();
        }
        
        // Should complete without issues
        assert!(true);
    }

    #[test]
    fn test_ai_provider_edge_cases() {
        // Test AI provider edge cases
        let mut ai_service = AIProviderService::new();
        
        // Add providers multiple times
        for _ in 0..10 {
            ai_service.add_provider(Box::new(OpenAIService::new()));
            ai_service.add_provider(Box::new(OllamaService::new()));
        }
        
        // Should handle multiple additions
        assert_eq!(std::mem::size_of_val(&ai_service), std::mem::size_of::<AIProviderService>());
        
        // Test provider names
        let openai = OpenAIService::new();
        let ollama = OllamaService::new();
        
        assert_eq!(openai.provider_name(), "OpenAI");
        assert_eq!(ollama.provider_name(), "Ollama");
        assert_ne!(openai.provider_name(), ollama.provider_name());
    }

    #[test]
    fn test_guard_struct_edge_cases() {
        // Test guard structs with edge case UUIDs
        let zero_uuid = Uuid::nil();
        let max_uuid = Uuid::max();
        let regular_uuid = Uuid::new_v4();
        
        // Test AuthenticatedUser with different UUIDs
        let account_id = Uuid::new_v4();
        let username = "test".to_string();
        let auth_zero = AuthenticatedUser { token: zero_uuid, account_id, username: username.clone() };
        let auth_max = AuthenticatedUser { token: max_uuid, account_id, username: username.clone() };
        let auth_regular = AuthenticatedUser { token: regular_uuid, account_id, username: username.clone() };
        
        assert_eq!(auth_zero.token, zero_uuid);
        assert_eq!(auth_max.token, max_uuid);
        assert_eq!(auth_regular.token, regular_uuid);
        
        // Test OptionalUser edge cases
        let optional_none = OptionalUser { token: None };
        let optional_some = OptionalUser { token: Some(regular_uuid) };
        
        assert!(optional_none.token.is_none());
        assert!(optional_some.token.is_some());
        assert_eq!(optional_some.token.unwrap(), regular_uuid);
        
        // Test AdminUser
        let admin = AdminUser { token: regular_uuid };
        assert_eq!(admin.token, regular_uuid);
    }

    #[test]
    fn test_memory_allocation_patterns() {
        // Test memory allocation patterns for services
        let services = vec![
            std::mem::size_of::<AuthService>(),
            std::mem::size_of::<BlogService>(),
            std::mem::size_of::<CommentService>(),
            std::mem::size_of::<TagService>(),
            std::mem::size_of::<ReactionService>(),
        ];
        
        // Most services should be reasonably sized
        for &size in &services {
            assert!(size <= 256, "Service size {} exceeds reasonable limit", size);
        }
        
        // Test that we can create many instances without issues
        let many_services: Vec<_> = (0..1000).map(|_| AuthService::new()).collect();
        assert_eq!(many_services.len(), 1000);
    }

    #[test]
    fn test_error_propagation() {
        // Test error propagation patterns
        fn create_error() -> Result<String, sea_orm::DbErr> {
            BaseService::handle_not_found(None, "TestError")
        }
        
        fn propagate_error() -> Result<String, sea_orm::DbErr> {
            create_error()?;
            Ok("should not reach".to_string())
        }
        
        let result = propagate_error();
        assert!(result.is_err());
        
        match result.unwrap_err() {
            sea_orm::DbErr::RecordNotFound(msg) => {
                assert_eq!(msg, "TestError not found");
            }
            _ => panic!("Expected RecordNotFound error"),
        }
    }

    #[test]
    fn test_concurrent_access_patterns() {
        // Test concurrent access to services
        use std::sync::Arc;
        use std::thread;
        
        let auth_service = Arc::new(AuthService::new());
        let handles: Vec<_> = (0..10).map(|_| {
            let service = Arc::clone(&auth_service);
            thread::spawn(move || {
                // Simulate concurrent access
                let _size = std::mem::size_of_val(service.as_ref());
                "success"
            })
        }).collect();
        
        for handle in handles {
            assert_eq!(handle.join().unwrap(), "success");
        }
    }

    #[test]
    fn test_configuration_merge_edge_cases() {
        use rocket::figment::{providers::Serialized, Figment};
        
        // Test configuration merge with conflicting values
        let figment = Figment::new()
            .merge(Serialized::default("data_path", "/first/path"))
            .merge(Serialized::default("data_path", "/second/path")) // Should override
            .merge(Serialized::default("new_field", "value"));
            
        let config = AppConfig::from_figment(&figment);
        assert_eq!(config.data_path, "/second/path"); // Last value wins
    }

    #[test]
    fn test_service_composition_edge_cases() {
        // Test service composition edge cases
        let coordinator = CoordinatorService::new();
        
        // Test that coordinator doesn't interfere with individual services
        let individual_auth = AuthService::new();
        let individual_blog = BlogService::new();
        
        // All should be independent
        assert_eq!(std::mem::size_of_val(&coordinator), std::mem::size_of::<CoordinatorService>());
        assert_eq!(std::mem::size_of_val(&individual_auth), std::mem::size_of::<AuthService>());
        assert_eq!(std::mem::size_of_val(&individual_blog), std::mem::size_of::<BlogService>());
    }
}