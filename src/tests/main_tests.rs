#[cfg(test)]
mod tests {
    use crate::{catch_default, drop_rocket, drop_sea_orm_migration, drop_sqlx, drop_hyper, setup_logger};
    use log::Metadata;
    use rocket::response::Redirect;

    #[test]
    fn test_catch_default() {
        let redirect = catch_default();
        // Should redirect to root path
        assert_eq!(redirect.location(), "/");
    }

    #[test]
    fn test_drop_rocket_filter() {
        // Create mock metadata for rocket module
        let target = "rocket::test";
        let metadata = create_mock_metadata(target);
        
        // Should filter out rocket logs
        assert!(!drop_rocket(&metadata));
        
        // Test with underscore target
        let target = "_";
        let metadata = create_mock_metadata(target);
        assert!(!drop_rocket(&metadata));
        
        // Test with non-rocket target
        let target = "app::test";
        let metadata = create_mock_metadata(target);
        assert!(drop_rocket(&metadata));
    }

    #[test]
    fn test_drop_sea_orm_migration_filter() {
        // Create mock metadata for sea_orm_migration module
        let target = "sea_orm_migration::test";
        let metadata = create_mock_metadata(target);
        
        // Should filter out sea_orm_migration logs
        assert!(!drop_sea_orm_migration(&metadata));
        
        // Test with underscore target
        let target = "_";
        let metadata = create_mock_metadata(target);
        assert!(!drop_sea_orm_migration(&metadata));
        
        // Test with non-sea_orm_migration target
        let target = "app::test";
        let metadata = create_mock_metadata(target);
        assert!(drop_sea_orm_migration(&metadata));
    }

    #[test]
    fn test_drop_sqlx_filter() {
        // Create mock metadata for sqlx module
        let target = "sqlx::test";
        let metadata = create_mock_metadata(target);
        
        // Should filter out sqlx logs
        assert!(!drop_sqlx(&metadata));
        
        // Test with underscore target
        let target = "_";
        let metadata = create_mock_metadata(target);
        assert!(!drop_sqlx(&metadata));
        
        // Test with non-sqlx target
        let target = "app::test";
        let metadata = create_mock_metadata(target);
        assert!(drop_sqlx(&metadata));
    }

    #[test]
    fn test_drop_hyper_filter() {
        // Create mock metadata for hyper module
        let target = "hyper::test";
        let metadata = create_mock_metadata(target);
        
        // Should filter out hyper logs
        assert!(!drop_hyper(&metadata));
        
        // Test with underscore target
        let target = "_";
        let metadata = create_mock_metadata(target);
        assert!(!drop_hyper(&metadata));
        
        // Test with non-hyper target
        let target = "app::test";
        let metadata = create_mock_metadata(target);
        assert!(drop_hyper(&metadata));
    }

    #[test]
    fn test_setup_logger() {
        // Test that logger setup doesn't panic
        // Note: This is a basic test since the actual logger setup 
        // requires file system access and may conflict with other tests
        let result = setup_logger();
        // If it fails due to already initialized logger, that's expected in test environment
        match result {
            Ok(()) => { /* Success */ }
            Err(fern::InitError::SetLoggerError(_)) => { /* Logger already set, expected in tests */ }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    // Helper function to create mock metadata
    fn create_mock_metadata(target: &str) -> Metadata {
        Metadata::builder()
            .level(log::Level::Info)
            .target(target)
            .build()
    }
}