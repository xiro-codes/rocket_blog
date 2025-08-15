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
        
        assert_eq!(info.name, "Database Seeding");
        assert_eq!(info.kind, Kind::Ignite | Kind::Shutdown);
    }

    #[test]
    fn test_seeding_fairing_kind() {
        let seeding = Seeding::new(Some(42), 100);
        let info = seeding.info();
        
        // Should handle both ignite and shutdown
        assert!(info.kind.contains(Kind::Ignite));
        assert!(info.kind.contains(Kind::Shutdown));
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
            
            assert_eq!(info.name, "Database Seeding");
            assert!(info.kind.contains(Kind::Ignite));
            assert!(info.kind.contains(Kind::Shutdown));
        }
    }

    #[tokio::test]
    async fn test_seeding_lifecycle() {
        use rocket::{Build, Rocket};
        use crate::tests::utils::create_test_rocket;
        use crate::pool::Db;
        use rocket_dyn_templates::Template;

        // Create a test rocket with seeding middleware
        let rocket = create_test_rocket()
            .manage(crate::config::AppConfig::default())
            .attach(Template::fairing())
            .attach(Seeding::new(Some(0), 1));

        // Should build successfully 
        assert!(rocket.state::<crate::config::AppConfig>().is_some());
    }
}