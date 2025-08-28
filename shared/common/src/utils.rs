use std::time::SystemTime;
use fern;
use humantime;
use log;
use chrono::{NaiveDateTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;

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

/// Generic utility functions for common operations across applications
pub struct Utils;

impl Utils {
    /// Format minutes into hours and minutes display
    pub fn format_duration_display(total_minutes: i32) -> (i32, i32) {
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        (hours, minutes)
    }

    /// Calculate hours from minutes as decimal
    pub fn minutes_to_hours_decimal(minutes: i32) -> Decimal {
        Decimal::from(minutes) / Decimal::from(60)
    }

    /// Parse decimal from string with error handling
    pub fn parse_decimal(value: &str, field_name: &str) -> Result<Decimal, String> {
        Decimal::from_str(value)
            .map_err(|_| format!("Invalid {} format", field_name))
    }

    /// Get current UTC timestamp
    pub fn now_utc() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    /// Format currency for display
    pub fn format_currency(amount: Decimal) -> String {
        format!("{:.2}", amount)
    }

    /// Calculate percentage
    pub fn calculate_percentage(part: Decimal, total: Decimal) -> Decimal {
        if total.is_zero() {
            Decimal::ZERO
        } else {
            (part / total) * Decimal::from(100)
        }
    }

    /// Validate required string field
    pub fn validate_required_string(value: &str, field_name: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            Err(format!("{} is required", field_name))
        } else {
            Ok(())
        }
    }

    /// Validate positive decimal
    pub fn validate_positive_decimal(value: Decimal, field_name: &str) -> Result<(), String> {
        if value <= Decimal::ZERO {
            Err(format!("{} must be greater than zero", field_name))
        } else {
            Ok(())
        }
    }
}

/// Generic traits for common validation patterns
pub trait Validatable {
    fn validate(&self) -> Result<(), Vec<String>>;
}

/// Generic error handling for services
#[derive(Debug)]
pub enum ServiceError {
    DatabaseError(String),
    ValidationError(Vec<String>),
    NotFound(String),
    InvalidData(String),
    BusinessLogicError(String),
}

impl From<sea_orm::DbErr> for ServiceError {
    fn from(err: sea_orm::DbErr) -> Self {
        match err {
            sea_orm::DbErr::RecordNotFound(msg) => ServiceError::NotFound(msg),
            _ => ServiceError::DatabaseError(err.to_string()),
        }
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ServiceError::ValidationError(errors) => write!(f, "Validation errors: {}", errors.join(", ")),
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ServiceError::BusinessLogicError(msg) => write!(f, "Business logic error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}