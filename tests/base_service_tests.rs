//! Unit tests for the BaseService module
use app::services::BaseService;
use sea_orm::DbErr;
use uuid::Uuid;

#[cfg(test)]
mod base_service_tests {
    use super::*;

    #[test]
    fn test_new_creates_base_service() {
        let service = BaseService::new();
        // BaseService doesn't have fields to check, but we verify it can be created
        assert!(std::mem::size_of_val(&service) == 0); // Zero-sized type
    }

    #[test]
    fn test_generate_id_creates_valid_uuid() {
        let id1 = BaseService::generate_id();
        let id2 = BaseService::generate_id();
        
        // Verify both are valid UUIDs
        assert!(id1.get_version().is_some());
        assert!(id2.get_version().is_some());
        
        // Verify they are different (extremely unlikely to be the same)
        assert_ne!(id1, id2);
        
        // Verify they are version 4 UUIDs
        assert_eq!(id1.get_version(), Some(uuid::Version::Random));
        assert_eq!(id2.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_handle_not_found_with_some_value() {
        let test_value = "test_data";
        let result = BaseService::handle_not_found(Some(test_value), "TestEntity");
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_value);
    }

    #[test]
    fn test_handle_not_found_with_none_value() {
        let result: Result<&str, DbErr> = BaseService::handle_not_found(None, "TestEntity");
        
        assert!(result.is_err());
        match result.unwrap_err() {
            DbErr::RecordNotFound(msg) => {
                assert_eq!(msg, "TestEntity not found");
            }
            _ => panic!("Expected RecordNotFound error"),
        }
    }

    #[test]
    fn test_handle_not_found_with_different_entity_names() {
        let result1: Result<&str, DbErr> = BaseService::handle_not_found(None, "User");
        let result2: Result<&str, DbErr> = BaseService::handle_not_found(None, "Post");
        
        assert!(result1.is_err());
        assert!(result2.is_err());
        
        if let DbErr::RecordNotFound(msg1) = result1.unwrap_err() {
            if let DbErr::RecordNotFound(msg2) = result2.unwrap_err() {
                assert_eq!(msg1, "User not found");
                assert_eq!(msg2, "Post not found");
            } else {
                panic!("Expected RecordNotFound error for Post");
            }
        } else {
            panic!("Expected RecordNotFound error for User");
        }
    }

    #[test]
    fn test_multiple_generate_id_calls_are_unique() {
        let mut ids = std::collections::HashSet::new();
        
        // Generate 100 UUIDs and ensure they're all unique
        for _ in 0..100 {
            let id = BaseService::generate_id();
            assert!(ids.insert(id), "Generated duplicate UUID: {}", id);
        }
        
        assert_eq!(ids.len(), 100);
    }
}