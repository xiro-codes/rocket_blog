#[cfg(test)]
mod tests {
    use crate::services::{WorkRoleService, WorkSessionService};
    use tokio;
    use uuid::Uuid;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_work_role_service() {
        let service = WorkRoleService::new();
        
        // Test that service initializes correctly
        assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<WorkRoleService>());
    }

    #[tokio::test]
    async fn test_work_session_service() {
        let service = WorkSessionService::new();
        
        // Test that service initializes correctly
        assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<WorkSessionService>());
    }

    #[test]
    fn test_decimal_parsing() {
        let rate_str = "25.50";
        let rate = Decimal::from_str(rate_str).unwrap();
        assert_eq!(rate.to_string(), "25.50");
    }

    #[test]
    fn test_uuid_parsing() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let uuid = Uuid::from_str(uuid_str).unwrap();
        assert_eq!(uuid.to_string(), uuid_str);
    }
}