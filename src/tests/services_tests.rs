#[cfg(test)]
mod tests {
    use crate::services::{AuthService, BaseService, BlogService, CommentService, TagService};
    use crate::tests::mocks::{create_mock_db, test_data};
    use sea_orm::{DbErr, MockDatabase, DatabaseBackend};
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
            let result: Result<String, DbErr> = BaseService::handle_not_found(None, "TestEntity");
            
            assert!(result.is_err());
            match result.unwrap_err() {
                DbErr::RecordNotFound(msg) => assert_eq!(msg, "TestEntity not found"),
                _ => panic!("Expected RecordNotFound error"),
            }
        }
    }

    mod auth_service_tests {
        use super::*;
        use tokio_test;

        #[test]
        fn test_auth_service_new() {
            let service = AuthService::new();
            // Should create successfully without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<AuthService>());
        }

        #[tokio::test]
        async fn test_login_with_invalid_credentials() {
            let service = AuthService::new();
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([
                    // Empty result set for account lookup
                    vec![]
                ])
                .into_connection();

            let form_data = models::dto::AccountFormDTO {
                username: "nonexistent_user".to_string(),
                password: "wrong_password".to_string(),
            };

            let result = service.login(&db, form_data).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_check_token_with_invalid_token() {
            let service = AuthService::new();
            let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
            
            let invalid_token = test_data::mock_uuid();
            let result = service.check_token(&db, invalid_token).await;
            
            assert!(result.is_none());
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

        #[tokio::test]
        async fn test_find_by_seq_id_not_found() {
            let service = BlogService::new();
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([
                    // Empty result set for post lookup
                    vec![]
                ])
                .into_connection();

            let result = service.find_by_seq_id(&db, 999).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_delete_by_id() {
            let service = BlogService::new();
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_exec_results([
                    sea_orm::MockExecResult {
                        last_insert_id: 1,
                        rows_affected: 1,
                    }
                ])
                .into_connection();

            let result = service.delete_by_id(&db, test_data::mock_post_id()).await;
            assert!(result.is_ok());
        }
    }

    mod comment_service_tests {
        use super::*;

        #[test]
        fn test_comment_service_new() {
            let service = CommentService::new();
            // Should create successfully without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<CommentService>());
        }

        #[tokio::test]
        async fn test_delete_by_id() {
            let service = CommentService::new();
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_exec_results([
                    sea_orm::MockExecResult {
                        last_insert_id: 1,
                        rows_affected: 1,
                    }
                ])
                .into_connection();

            let result = service.delete_by_id(&db, test_data::mock_uuid()).await;
            assert!(result.is_ok());
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

        #[tokio::test]
        async fn test_find_or_create_tag() {
            let service = TagService::new();
            let db = MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results([
                    // Empty result set (tag not found)
                    vec![]
                ])
                .append_query_results([
                    // Mock created tag result
                    vec![models::tag::Model {
                        id: test_data::mock_tag_id(),
                        name: "Test Tag".to_string(),
                        slug: "test-tag".to_string(),
                        color: Some("#FF0000".to_string()),
                        created_at: test_data::mock_timestamp().and_utc(),
                    }]
                ])
                .into_connection();

            let result = service.find_or_create_tag(&db, "Test Tag").await;
            assert!(result.is_ok());
            let tag = result.unwrap();
            assert_eq!(tag.name, "Test Tag");
            assert_eq!(tag.slug, "test-tag");
        }
    }
}