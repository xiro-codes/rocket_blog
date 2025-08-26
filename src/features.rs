/// Feature flags for different build configurations
pub struct Features;

impl Features {
    /// Check if we're in development mode
    pub const fn is_development() -> bool {
        cfg!(debug_assertions)
    }
    
    /// Check if seeding should be enabled
    pub const fn enable_seeding() -> bool {
        cfg!(debug_assertions)
    }
    
    /// Check if detailed logging should be enabled
    pub const fn enable_detailed_logging() -> bool {
        cfg!(debug_assertions)
    }
    
    /// Get the log level based on build configuration
    pub const fn log_level() -> log::LevelFilter {
        if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        }
    }
}