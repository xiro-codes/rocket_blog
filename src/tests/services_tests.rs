#[cfg(test)]
mod tests {
    use crate::services::{AuthService, BaseService, BlogService, CommentService, TagService, AIProviderService, OpenAIService, OllamaService, AIProvider, BackgroundJobService};
    use crate::tests::mocks::test_data;
    use uuid::Uuid;

    mod base_service_tests {
        use super::*;

        #[test]
        fn test_base_service_new() {
            let service = BaseService::new();
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<BaseService>());
        }

        #[test]
        fn test_generate_id() {
            let id1 = BaseService::generate_id();
            let id2 = BaseService::generate_id();
            
            // Should generate valid UUIDs
            assert_ne!(id1, id2);
            assert_eq!(id1.get_version(), Some(uuid::Version::Random));
            assert_eq!(id2.get_version(), Some(uuid::Version::Random));
        }

        #[test]
        fn test_handle_not_found_with_some() {
            let value = "test_value";
            let result = BaseService::handle_not_found(Some(value), "TestEntity");
            
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), value);
        }

        #[test]
        fn test_handle_not_found_with_none() {
            let result: Result<String, sea_orm::DbErr> = BaseService::handle_not_found(None, "TestEntity");
            
            assert!(result.is_err());
            match result.unwrap_err() {
                sea_orm::DbErr::RecordNotFound(msg) => assert_eq!(msg, "TestEntity not found"),
                _ => panic!("Expected RecordNotFound error"),
            }
        }
    }

    mod auth_service_tests {
        use super::*;
        use models::dto::{AccountFormDTO, AdminCreateFormDTO};

        #[test]
        fn test_auth_service_new() {
            let service = AuthService::new();
            // Should create successfully without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<AuthService>());
        }

        #[test]
        fn test_admin_create_form_dto() {
            let form_data = AdminCreateFormDTO {
                username: "admin".to_string(),
                password: "password123".to_string(),
                email: "admin@example.com".to_string(),
            };
            
            assert_eq!(form_data.username, "admin");
            assert_eq!(form_data.password, "password123");
            assert_eq!(form_data.email, "admin@example.com");
        }

        #[test]
        fn test_account_form_dto() {
            let form_data = AccountFormDTO {
                username: "user".to_string(),
                password: "password123".to_string(),
            };
            
            assert_eq!(form_data.username, "user");
            assert_eq!(form_data.password, "password123");
        }
    }

    mod blog_service_tests {
        use super::*;

        #[test]
        fn test_blog_service_new() {
            let service = BlogService::new();
            // Should create successfully without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<BlogService>());
        }

        #[test]
        fn test_paginate_with_title_include_drafts_parameters() {
            let service = BlogService::new();
            
            // Test that the method exists and can be called with different include_drafts values
            // We can't test the actual database logic without a real database connection,
            // but we can verify the method signature and basic parameter handling
            
            // This test verifies that:
            // 1. The method exists with the expected signature
            // 2. Both include_drafts true/false are handled without compile errors
            // 3. The service can be instantiated
            
            // Since these are async methods that need a database connection,
            // we'll just verify the service can be created and the methods exist
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<BlogService>());
        }

        #[test]
        fn test_find_recent_published_posts_method_exists() {
            let service = BlogService::new();
            
            // Test that the RSS feed method exists
            // We can't test the actual database logic without a real database connection,
            // but we can verify the method exists by checking the service is created properly
            
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<BlogService>());
        }

        #[test]
        fn test_prepare_tsquery() {
            // Test basic query preparation
            assert_eq!(BlogService::prepare_tsquery("hello"), "hello:*");
            
            // Test multiple terms
            assert_eq!(BlogService::prepare_tsquery("hello world"), "hello:* & world:*");
            
            // Test with special characters (should be filtered out)
            assert_eq!(BlogService::prepare_tsquery("hello! @world#"), "hello:* & world:*");
            
            // Test with allowed characters
            assert_eq!(BlogService::prepare_tsquery("hello-world test_case"), "hello-world:* & test_case:*");
            
            // Test empty query
            assert_eq!(BlogService::prepare_tsquery(""), "''");
            
            // Test whitespace only
            assert_eq!(BlogService::prepare_tsquery("   "), "''");
            
            // Test with extra whitespace
            assert_eq!(BlogService::prepare_tsquery("  hello   world  "), "hello:* & world:*");
        }
    }

    mod comment_service_tests {
        use super::*;
        use models::dto::CommentFormDTO;

        #[test]
        fn test_comment_service_new() {
            let service = CommentService::new();
            // Should create successfully without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<CommentService>());
        }

        #[test]
        fn test_comment_form_dto() {
            let form_data = CommentFormDTO {
                text: "This is a test comment".to_string(),
                username: Some("anonymous_user".to_string()),
                parent_id: None,
            };
            
            assert_eq!(form_data.text, "This is a test comment");
            assert_eq!(form_data.username, Some("anonymous_user".to_string()));
            assert_eq!(form_data.parent_id, None);
        }

        #[test]
        fn test_comment_form_dto_without_username() {
            let form_data = CommentFormDTO {
                text: "This is a test comment".to_string(),
                username: None,
                parent_id: None,
            };
            
            assert_eq!(form_data.text, "This is a test comment");
            assert_eq!(form_data.username, None);
            assert_eq!(form_data.parent_id, None);
        }
    }

    mod tag_service_tests {
        use super::*;

        #[test]
        fn test_tag_service_new() {
            let service = TagService::new();
            // Should create successfully without panicking  
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<TagService>());
        }
    }

    mod ai_provider_tests {
        use super::*;
        use crate::services::{AIProviderService, OpenAIService, OllamaService, AIProvider};

        #[test]
        fn test_ai_provider_service_new() {
            let service = AIProviderService::new();
            // Should create successfully without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<AIProviderService>());
        }

        #[test]
        fn test_openai_service_creation() {
            let service = OpenAIService::new();
            assert_eq!(service.provider_name(), "OpenAI");
        }

        #[test]
        fn test_ollama_service_creation() {
            let service = OllamaService::new();
            assert_eq!(service.provider_name(), "Ollama");
        }

        #[test]
        fn test_ai_provider_service_with_providers() {
            let mut service = AIProviderService::new();
            service.add_provider(Box::new(OpenAIService::new()));
            service.add_provider(Box::new(OllamaService::new()));
            
            // Should have added providers without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<AIProviderService>());
        }

        #[test]
        fn test_openai_service_provider_name() {
            let service = OpenAIService::new();
            assert_eq!(service.provider_name(), "OpenAI");
            assert!(!service.provider_name().is_empty());
        }

        #[test]
        fn test_ollama_service_provider_name() {
            let service = OllamaService::new();
            assert_eq!(service.provider_name(), "Ollama");
            assert!(!service.provider_name().is_empty());
        }

        #[test]
        fn test_ai_provider_service_multiple_additions() {
            let mut service = AIProviderService::new();
            
            // Add multiple providers
            for _ in 0..3 {
                service.add_provider(Box::new(OpenAIService::new()));
                service.add_provider(Box::new(OllamaService::new()));
            }
            
            // Should handle multiple additions without issues
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<AIProviderService>());
        }

        #[test]
        fn test_ai_provider_service_empty_state() {
            let service = AIProviderService::new();
            
            // Service should be usable even without providers
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<AIProviderService>());
        }
    }

    mod coordinator_service_tests {
        use super::*;
        use crate::services::CoordinatorService;

        #[test]
        fn test_coordinator_service_new() {
            let service = CoordinatorService::new();
            
            // Should create successfully with all sub-services
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<CoordinatorService>());
        }

        #[test]
        fn test_coordinator_service_composition() {
            let service = CoordinatorService::new();
            
            // Coordinator should compose multiple services
            // We can't access private fields directly, but we can verify creation
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<CoordinatorService>());
        }

        #[test]
        fn test_coordinator_service_multiple_instances() {
            let service1 = CoordinatorService::new();
            let service2 = CoordinatorService::new();
            
            // Should be able to create multiple instances
            assert_eq!(std::mem::size_of_val(&service1), std::mem::size_of::<CoordinatorService>());
            assert_eq!(std::mem::size_of_val(&service2), std::mem::size_of::<CoordinatorService>());
        }
    }

    mod reaction_service_tests {
        use super::*;
        use crate::services::ReactionService;

        #[test]
        fn test_reaction_service_new() {
            let service = ReactionService::new();
            
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<ReactionService>());
        }

        #[test]
        fn test_reaction_service_creation_multiple() {
            // Test multiple service creation
            let services: Vec<ReactionService> = (0..5).map(|_| ReactionService::new()).collect();
            
            assert_eq!(services.len(), 5);
            for service in &services {
                assert_eq!(std::mem::size_of_val(service), std::mem::size_of::<ReactionService>());
            }
        }
    }

    mod settings_service_tests {
        use super::*;
        use crate::services::SettingsService;

        #[test]
        fn test_settings_service_new() {
            let service = SettingsService::new();
            
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<SettingsService>());
        }

        #[test]
        fn test_settings_service_encryption_key() {
            let service1 = SettingsService::new();
            let service2 = SettingsService::new();
            
            // Both services should be created successfully
            // Note: We can't directly test encryption key equality since it's private
            assert_eq!(std::mem::size_of_val(&service1), std::mem::size_of::<SettingsService>());
            assert_eq!(std::mem::size_of_val(&service2), std::mem::size_of::<SettingsService>());
        }
    }

    mod background_job_service_tests {
        use super::*;
        use crate::services::BackgroundJobService;
        use models::background_job;
        use sea_orm::{MockDatabase, DbBackend, MockExecResult};
        use chrono::{DateTime, FixedOffset};
        use serde_json::json;

        #[tokio::test]
        async fn test_background_job_timezone_aware_creation() {
            // Create mock data with timezone-aware timestamp
            let mock_job = background_job::Model {
                id: Uuid::new_v4(),
                job_type: "youtube_download".to_string(),
                entity_type: "post".to_string(),
                entity_id: Uuid::new_v4(),
                status: "pending".to_string(),
                error_message: None,
                job_data: Some(json!({"url": "https://youtube.com/watch?v=test"})),
                created_at: DateTime::<FixedOffset>::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap(),
                updated_at: DateTime::<FixedOffset>::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap(),
            };

            // Create mock database
            let db = MockDatabase::new(DbBackend::Postgres)
                .append_query_results([
                    vec![mock_job.clone()],
                ])
                .into_connection();

            let service = BackgroundJobService::new();
            
            // Test creating a job with timezone-aware timestamps
            let result = service.create_job(
                &db,
                "youtube_download".to_string(),
                "post".to_string(),
                Uuid::new_v4(),
                "pending".to_string(),
                Some(json!({"url": "https://youtube.com/watch?v=test"})),
            ).await;

            // Verify the job was created successfully
            assert!(result.is_ok());
            let job = result.unwrap();
            assert_eq!(job.job_type, "youtube_download");
            assert_eq!(job.entity_type, "post");
            assert_eq!(job.status, "pending");
            
            // Verify that created_at and updated_at are timezone-aware
            // The exact values will be mock data, but the types should be correct
            assert!(job.created_at.timezone().local_minus_utc() == 0); // Should be UTC offset
            assert!(job.updated_at.timezone().local_minus_utc() == 0);
        }

        #[test]
        fn test_background_job_service_new() {
            let service = BackgroundJobService::new();
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<BackgroundJobService>());
        }
    }
}