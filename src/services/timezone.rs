use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::{Tz, America, Europe, Asia, Australia};
use std::str::FromStr;

/// Timezone utility service for handling timezone conversions and preferences
pub struct TimezoneService;

impl TimezoneService {
    /// Get a list of common timezones for user selection
    pub fn get_common_timezones() -> Vec<(String, String)> {
        vec![
            // UTC
            ("UTC".to_string(), "UTC".to_string()),
            
            // North America
            ("America/New_York".to_string(), "Eastern Time (New York)".to_string()),
            ("America/Chicago".to_string(), "Central Time (Chicago)".to_string()),
            ("America/Denver".to_string(), "Mountain Time (Denver)".to_string()),
            ("America/Los_Angeles".to_string(), "Pacific Time (Los Angeles)".to_string()),
            ("America/Toronto".to_string(), "Eastern Time (Toronto)".to_string()),
            ("America/Vancouver".to_string(), "Pacific Time (Vancouver)".to_string()),
            
            // Europe
            ("Europe/London".to_string(), "British Time (London)".to_string()),
            ("Europe/Paris".to_string(), "Central European Time (Paris)".to_string()),
            ("Europe/Berlin".to_string(), "Central European Time (Berlin)".to_string()),
            ("Europe/Rome".to_string(), "Central European Time (Rome)".to_string()),
            ("Europe/Madrid".to_string(), "Central European Time (Madrid)".to_string()),
            ("Europe/Amsterdam".to_string(), "Central European Time (Amsterdam)".to_string()),
            ("Europe/Stockholm".to_string(), "Central European Time (Stockholm)".to_string()),
            ("Europe/Moscow".to_string(), "Moscow Time".to_string()),
            
            // Asia
            ("Asia/Tokyo".to_string(), "Japan Time (Tokyo)".to_string()),
            ("Asia/Shanghai".to_string(), "China Time (Shanghai)".to_string()),
            ("Asia/Seoul".to_string(), "Korea Time (Seoul)".to_string()),
            ("Asia/Hong_Kong".to_string(), "Hong Kong Time".to_string()),
            ("Asia/Singapore".to_string(), "Singapore Time".to_string()),
            ("Asia/Kolkata".to_string(), "India Time (Kolkata)".to_string()),
            ("Asia/Dubai".to_string(), "UAE Time (Dubai)".to_string()),
            
            // Australia
            ("Australia/Sydney".to_string(), "Australian Eastern Time (Sydney)".to_string()),
            ("Australia/Melbourne".to_string(), "Australian Eastern Time (Melbourne)".to_string()),
            ("Australia/Perth".to_string(), "Australian Western Time (Perth)".to_string()),
            
            // Others
            ("Pacific/Auckland".to_string(), "New Zealand Time (Auckland)".to_string()),
            ("Africa/Cairo".to_string(), "Egypt Time (Cairo)".to_string()),
        ]
    }

    /// Convert UTC DateTime to user's timezone
    pub fn convert_to_user_timezone(utc_time: DateTime<Utc>, user_timezone: &str) -> Result<DateTime<Tz>, String> {
        let tz = Tz::from_str(user_timezone)
            .map_err(|_| format!("Invalid timezone: {}", user_timezone))?;
        
        Ok(utc_time.with_timezone(&tz))
    }

    /// Format datetime for display in user's timezone
    pub fn format_for_display(utc_time: DateTime<Utc>, user_timezone: &str, format: &str) -> Result<String, String> {
        let user_time = Self::convert_to_user_timezone(utc_time, user_timezone)?;
        Ok(user_time.format(format).to_string())
    }

    /// Format datetime for display with timezone abbreviation
    pub fn format_with_timezone(utc_time: DateTime<Utc>, user_timezone: &str) -> Result<String, String> {
        let user_time = Self::convert_to_user_timezone(utc_time, user_timezone)?;
        Ok(user_time.format("%Y-%m-%d %H:%M:%S %Z").to_string())
    }

    /// Format date only in user's timezone
    pub fn format_date_only(utc_time: DateTime<Utc>, user_timezone: &str) -> Result<String, String> {
        Self::format_for_display(utc_time, user_timezone, "%Y-%m-%d")
    }

    /// Format time only in user's timezone
    pub fn format_time_only(utc_time: DateTime<Utc>, user_timezone: &str) -> Result<String, String> {
        Self::format_for_display(utc_time, user_timezone, "%H:%M:%S")
    }

    /// Format datetime for HTML datetime-local input
    pub fn format_for_datetime_local(utc_time: DateTime<Utc>, user_timezone: &str) -> Result<String, String> {
        Self::format_for_display(utc_time, user_timezone, "%Y-%m-%dT%H:%M")
    }

    /// Format datetime for short table display (date and time separated)
    pub fn format_short_datetime(utc_time: DateTime<Utc>, user_timezone: &str) -> Result<(String, String), String> {
        let user_time = Self::convert_to_user_timezone(utc_time, user_timezone)?;
        let date = user_time.format("%m/%d").to_string();
        let time = user_time.format("%H:%M").to_string();
        Ok((date, time))
    }

    /// Format datetime for compact display (one line, shorter)
    pub fn format_compact(utc_time: DateTime<Utc>, user_timezone: &str) -> Result<String, String> {
        let user_time = Self::convert_to_user_timezone(utc_time, user_timezone)?;
        Ok(user_time.format("%m/%d %H:%M").to_string())
    }

    /// Validate if a timezone string is valid
    pub fn is_valid_timezone(timezone: &str) -> bool {
        Tz::from_str(timezone).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_timezone_conversion() {
        let utc_time = Utc.with_ymd_and_hms(2024, 12, 15, 14, 30, 0).unwrap();
        
        // Test New York timezone (EST/EDT)
        let ny_time = TimezoneService::convert_to_user_timezone(utc_time, "America/New_York").unwrap();
        assert_eq!(ny_time.timezone().name(), "America/New_York");
        
        // Test UTC conversion (should be the same)
        let utc_converted = TimezoneService::convert_to_user_timezone(utc_time, "UTC").unwrap();
        assert_eq!(utc_converted.naive_utc(), utc_time.naive_utc());
    }

    #[test]
    fn test_format_with_timezone() {
        let utc_time = Utc.with_ymd_and_hms(2024, 12, 15, 14, 30, 0).unwrap();
        
        let formatted = TimezoneService::format_with_timezone(utc_time, "America/New_York").unwrap();
        assert!(formatted.contains("2024-12-15"));
        assert!(formatted.contains("EST") || formatted.contains("EDT"));
    }

    #[test]
    fn test_invalid_timezone() {
        let utc_time = Utc.with_ymd_and_hms(2024, 12, 15, 14, 30, 0).unwrap();
        
        let result = TimezoneService::convert_to_user_timezone(utc_time, "Invalid/Timezone");
        assert!(result.is_err());
    }

    #[test]
    fn test_timezone_validation() {
        assert!(TimezoneService::is_valid_timezone("America/New_York"));
        assert!(TimezoneService::is_valid_timezone("UTC"));
        assert!(TimezoneService::is_valid_timezone("Europe/London"));
        assert!(!TimezoneService::is_valid_timezone("Invalid/Timezone"));
        assert!(!TimezoneService::is_valid_timezone(""));
    }

    #[test]
    fn test_common_timezones_list() {
        let timezones = TimezoneService::get_common_timezones();
        assert!(!timezones.is_empty());
        
        // Verify all returned timezones are valid
        for (tz_id, _display_name) in timezones {
            assert!(TimezoneService::is_valid_timezone(&tz_id), "Invalid timezone: {}", tz_id);
        }
    }
}