#[cfg(test)]
mod tests {
    use crate::{should_filter_log, setup_logger};
    use log::Metadata;

    #[test]
    fn test_unified_log_filter() {
        // Test rocket logs are filtered
        let rocket_metadata = create_mock_metadata("rocket::test");
        assert!(should_filter_log(&rocket_metadata));
        
        // Test sea_orm_migration logs are filtered
        let sea_orm_metadata = create_mock_metadata("sea_orm_migration::test");
        assert!(should_filter_log(&sea_orm_metadata));
        
        // Test sqlx logs are filtered
        let sqlx_metadata = create_mock_metadata("sqlx::test");
        assert!(should_filter_log(&sqlx_metadata));
        
        // Test hyper logs are filtered
        let hyper_metadata = create_mock_metadata("hyper::test");
        assert!(should_filter_log(&hyper_metadata));
        
        // Test underscore target is filtered
        let underscore_metadata = create_mock_metadata("_");
        assert!(should_filter_log(&underscore_metadata));
        
        // Test non-noisy target is not filtered
        let app_metadata = create_mock_metadata("app::test");
        assert!(!should_filter_log(&app_metadata));
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
    fn create_mock_metadata(target: &str) -> Metadata<'_> {
        Metadata::builder()
            .level(log::Level::Info)
            .target(target)
            .build()
    }
}