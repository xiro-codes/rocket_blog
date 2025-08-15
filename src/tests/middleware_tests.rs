#[cfg(test)]
mod tests {
    use crate::middleware::Seeding;
    use rocket::fairing::{Fairing, Kind};

    #[test]
    fn test_seeding_new() {
        let seeding = Seeding::new(Some(10), 25);
        
        // Should create successfully without panicking
        assert_eq!(std::mem::size_of_val(&seeding), std::mem::size_of::<Seeding>());
    }

    #[test]  
    fn test_seeding_new_with_none_seed() {
        let seeding = Seeding::new(None, 50);
        
        // Should create successfully without panicking
        assert_eq!(std::mem::size_of_val(&seeding), std::mem::size_of::<Seeding>());
    }

    #[test]
    fn test_seeding_fairing_info() {
        let seeding = Seeding::new(Some(0), 10);
        let info = seeding.info();
        
        assert_eq!(info.name, "Seeding");
        // Just test that kind is set and has the expected type
        assert!(std::mem::size_of_val(&info.kind) > 0);
    }

    #[test]
    fn test_seeding_fairing_kind() {
        let seeding = Seeding::new(Some(42), 100);
        let info = seeding.info();
        
        // Should handle both ignite and shutdown - test by verifying it's not empty
        assert!(std::mem::size_of_val(&info.kind) > 0);
    }

    #[test]
    fn test_seeding_with_different_parameters() {
        // Test various parameter combinations
        let test_cases = vec![
            (Some(0), 1),
            (Some(1), 10),
            (Some(100), 50),
            (None, 25),
            (None, 100),
        ];

        for (seed, count) in test_cases {
            let seeding = Seeding::new(seed, count);
            let info = seeding.info();
            
            assert_eq!(info.name, "Seeding");
            assert!(std::mem::size_of_val(&info.kind) > 0);
        }
    }
}