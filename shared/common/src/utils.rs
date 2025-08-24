use std::time::SystemTime;
use fern;
use humantime;
use log;

/// Setup application logging with clean filtering
pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let timestamp = humantime::format_rfc3339_seconds(SystemTime::now());
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                timestamp,
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("rocket", log::LevelFilter::Warn)
        .level_for("sea_orm_migration", log::LevelFilter::Warn)
        .level_for("sqlx", log::LevelFilter::Warn)
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("_", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .apply()
        .map_err(|e| fern::InitError::SetLoggerError(e))
}

/// Unified log filter for noisy dependencies
pub fn should_filter_log(meta: &log::Metadata) -> bool {
    let target = meta.target();
    // Filter out noisy log targets
    target.starts_with("rocket") || 
    target.starts_with("sea_orm_migration") || 
    target.starts_with("sqlx") || 
    target.starts_with("hyper") || 
    target.eq("_")
}