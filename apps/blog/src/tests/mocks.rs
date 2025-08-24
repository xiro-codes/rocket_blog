// Test data factory for creating consistent test data

pub mod test_data {
    use chrono::Local;
    use uuid::Uuid;
    
    pub fn mock_uuid() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    }
    
    pub fn mock_timestamp() -> chrono::NaiveDateTime {
        Local::now().naive_local()
    }
    
    pub fn mock_user_id() -> Uuid {
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
    }
    
    pub fn mock_post_id() -> Uuid {
        Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()
    }
    
    pub fn mock_tag_id() -> Uuid {
        Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap()
    }
}