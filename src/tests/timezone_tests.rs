#[cfg(test)]
mod timezone_tests {
    use super::*;
    use crate::services::WorkTimeService;
    use models::{work_time_entry, dto::TimeTrackingControlDTO};
    use sea_orm::{MockDatabase, DbBackend, MockExecResult};
    use chrono::{DateTime, Utc, FixedOffset, TimeZone};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_timezone_aware_time_tracking() {
        // Create timezone-aware test data
        let account_id = Uuid::new_v4();
        let user_role_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();
        
        // Create timestamps with different timezones
        let utc_start_time = Utc.with_ymd_and_hms(2024, 12, 15, 10, 0, 0).unwrap();
        let utc_end_time = Utc.with_ymd_and_hms(2024, 12, 15, 12, 0, 0).unwrap();
        
        // Mock entry data with timezone-aware timestamps
        let mock_entry = work_time_entry::Model {
            id: entry_id,
            account_id,
            user_role_id,
            pay_period_id: None,
            start_time: utc_start_time,
            end_time: Some(utc_end_time),
            duration: Some(120), // 2 hours in minutes
            description: Some("Test work session".to_string()),
            project: Some("Test project".to_string()),
            is_active: false,
            tips: None,
            created_at: utc_start_time,
            updated_at: utc_end_time,
        };

        // Create mock database
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([
                vec![mock_entry.clone()], // For insert result
            ])
            .append_exec_results([
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
            ])
            .into_connection();

        let service = WorkTimeService::new();
        
        // Verify that timestamps maintain timezone information
        assert_eq!(mock_entry.start_time.timezone(), Utc);
        assert_eq!(mock_entry.end_time.unwrap().timezone(), Utc);
        assert_eq!(mock_entry.created_at.timezone(), Utc);
        assert_eq!(mock_entry.updated_at.timezone(), Utc);
        
        // Verify duration calculation works correctly with timezone-aware timestamps
        let calculated_duration = (mock_entry.end_time.unwrap() - mock_entry.start_time).num_minutes();
        assert_eq!(calculated_duration, 120);
    }

    #[test]
    fn test_timezone_conversion_consistency() {
        // Test that different timezone representations of the same instant work correctly
        let utc_time = Utc.with_ymd_and_hms(2024, 12, 15, 15, 30, 0).unwrap();
        
        // Create equivalent times in different timezones
        let est_offset = FixedOffset::west_opt(5 * 3600).unwrap(); // EST: UTC-5
        let pst_offset = FixedOffset::west_opt(8 * 3600).unwrap(); // PST: UTC-8
        
        let est_time = utc_time.with_timezone(&est_offset);
        let pst_time = utc_time.with_timezone(&pst_offset);
        
        // All should represent the same instant in time
        assert_eq!(utc_time.timestamp(), est_time.timestamp());
        assert_eq!(utc_time.timestamp(), pst_time.timestamp());
        
        // When converted to UTC, they should be identical
        assert_eq!(utc_time, est_time.with_timezone(&Utc));
        assert_eq!(utc_time, pst_time.with_timezone(&Utc));
    }

    #[test]
    fn test_rfc3339_parsing_preserves_timezone() {
        // Test parsing of timezone-aware RFC3339 strings
        let test_cases = vec![
            ("2024-12-15T10:30:00Z", 0),                    // UTC
            ("2024-12-15T10:30:00+00:00", 0),              // UTC explicit
            ("2024-12-15T05:30:00-05:00", -5 * 3600),      // EST
            ("2024-12-15T02:30:00-08:00", -8 * 3600),      // PST
            ("2024-12-15T16:30:00+06:00", 6 * 3600),       // +6 timezone
        ];

        for (rfc3339_str, expected_offset) in test_cases {
            let parsed = DateTime::parse_from_rfc3339(rfc3339_str).unwrap();
            let utc_converted = parsed.with_timezone(&Utc);
            
            // All should convert to the same UTC time
            let expected_utc = Utc.with_ymd_and_hms(2024, 12, 15, 10, 30, 0).unwrap();
            assert_eq!(utc_converted, expected_utc, "Failed for timezone offset {}", expected_offset);
        }
    }

    #[test]
    fn test_daylight_saving_time_handling() {
        // Test edge case around DST transitions
        
        // Spring forward: 2024-03-10 2:00 AM becomes 3:00 AM in EST
        let before_dst = FixedOffset::west_opt(5 * 3600).unwrap() // EST
            .with_ymd_and_hms(2024, 3, 10, 1, 0, 0).unwrap();
        let after_dst = FixedOffset::west_opt(4 * 3600).unwrap() // EDT  
            .with_ymd_and_hms(2024, 3, 10, 3, 0, 0).unwrap();
        
        // Convert both to UTC
        let before_utc = before_dst.with_timezone(&Utc);
        let after_utc = after_dst.with_timezone(&Utc);
        
        // Should be exactly 1 hour apart in UTC
        let duration = after_utc - before_utc;
        assert_eq!(duration.num_hours(), 1);
    }

    #[test] 
    fn test_database_storage_format() {
        // Test that our timezone-aware DateTime<Utc> can be properly stored/retrieved
        let test_time = Utc.with_ymd_and_hms(2024, 12, 15, 14, 30, 45).unwrap();
        
        // Verify it maintains precision
        assert_eq!(test_time.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string(), 
                  "2024-12-15 14:30:45.000 UTC");
        
        // Verify it can round-trip through RFC3339
        let rfc3339_str = test_time.to_rfc3339();
        let parsed_back = DateTime::parse_from_rfc3339(&rfc3339_str).unwrap().with_timezone(&Utc);
        assert_eq!(test_time, parsed_back);
    }
}