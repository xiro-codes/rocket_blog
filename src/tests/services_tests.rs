#[cfg(test)]
mod tests {
    use crate::services::{AuthService, BaseService, BlogService, CommentService, TagService};
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

        #[test]
        fn test_auth_service_new() {
            let service = AuthService::new();
            // Should create successfully without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<AuthService>());
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
    }

    mod comment_service_tests {
        use super::*;

        #[test]
        fn test_comment_service_new() {
            let service = CommentService::new();
            // Should create successfully without panicking
            assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<CommentService>());
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
}