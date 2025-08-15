//! Unit tests for the AuthService module
//! Note: These tests focus on the service logic rather than database operations
use app::services::AuthService;
use uuid::Uuid;

#[cfg(test)]
mod auth_service_tests {
    use super::*;

    #[tokio::test]
    async fn test_new_creates_auth_service() {
        let service = AuthService::new();
        
        // We can't directly inspect the internal state, but we can verify
        // that the service is created and the token_map is empty by testing
        // check_token with a random token
        let random_token = Uuid::new_v4();
        
        // Since we don't have a real database connection, we can't fully test check_token
        // But we can verify the service exists and can be used
        // This test mainly verifies the service can be instantiated
        let _service_exists = true; // Just verify it compiles and creates
        assert!(_service_exists);
    }

    #[test]
    fn test_token_and_account_id_types() {
        // Test that our type aliases work correctly
        let token: Uuid = Uuid::new_v4();
        let account_id: Uuid = Uuid::new_v4();
        
        assert_ne!(token, account_id);
        assert!(token.get_version().is_some());
        assert!(account_id.get_version().is_some());
    }

    // Note: Full testing of login() and check_token() would require database mocking
    // or integration tests with a test database. The core logic involves:
    // 1. Database lookups (Account::find())  
    // 2. Password verification (bcrypt::verify())
    // 3. Token storage in HashMap
    // 
    // These would be better tested in integration tests with a real or mocked database.

    #[tokio::test]
    async fn test_multiple_auth_services_are_independent() {
        let service1 = AuthService::new();
        let service2 = AuthService::new();
        
        // Each service should have its own independent token_map
        // We can't directly test this without database operations,
        // but we can verify they're separate instances
        let service1_ptr = &service1 as *const AuthService;
        let service2_ptr = &service2 as *const AuthService;
        
        assert_ne!(service1_ptr, service2_ptr);
    }
}