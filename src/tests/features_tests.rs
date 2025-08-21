#[cfg(test)]
mod tests {
    use crate::features::Features;

    #[test]
    fn test_features_is_development() {
        // Test that is_development() returns the expected value based on build configuration
        let is_dev = Features::is_development();
        
        // In debug build, should be true; in release build, should be false
        #[cfg(debug_assertions)]
        assert!(is_dev);
        
        #[cfg(not(debug_assertions))]
        assert!(!is_dev);
    }

    #[test]
    fn test_features_enable_seeding() {
        // Test that enable_seeding() follows the same pattern as is_development()
        let enable_seeding = Features::enable_seeding();
        let is_dev = Features::is_development();
        
        // Should match development mode setting
        assert_eq!(enable_seeding, is_dev);
    }

    #[test]
    fn test_features_enable_detailed_logging() {
        // Test that enable_detailed_logging() follows the same pattern as is_development()
        let enable_logging = Features::enable_detailed_logging();
        let is_dev = Features::is_development();
        
        // Should match development mode setting
        assert_eq!(enable_logging, is_dev);
    }

    #[test]
    fn test_features_log_level() {
        // Test that log_level() returns appropriate level based on build configuration
        let log_level = Features::log_level();
        
        #[cfg(debug_assertions)]
        assert_eq!(log_level, log::LevelFilter::Debug);
        
        #[cfg(not(debug_assertions))]
        assert_eq!(log_level, log::LevelFilter::Info);
    }

    #[test]
    fn test_features_consistency() {
        // Test that all feature flags are consistent with each other
        let is_dev = Features::is_development();
        let enable_seeding = Features::enable_seeding();
        let enable_logging = Features::enable_detailed_logging();
        let log_level = Features::log_level();
        
        // All boolean flags should match is_development
        assert_eq!(is_dev, enable_seeding);
        assert_eq!(is_dev, enable_logging);
        
        // Log level should be consistent with development mode
        if is_dev {
            assert_eq!(log_level, log::LevelFilter::Debug);
        } else {
            assert_eq!(log_level, log::LevelFilter::Info);
        }
    }
}