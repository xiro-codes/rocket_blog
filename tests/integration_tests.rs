//! Integration tests for the complete application setup
//! These tests verify that the application can be built and configured properly

use rocket::local::blocking::Client;
use rocket::http::Status;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_minimal_test_rocket() -> rocket::Rocket<rocket::Build> {
        use app::controllers;
        use app::{catch_default, config::AppConfig};
        use rocket_dyn_templates::Template;

        // Create a minimal rocket instance for testing
        // We exclude database and seeding to avoid external dependencies
        let figment = rocket::Config::figment();
        let app_config = AppConfig::from_figment(&figment);
        
        rocket::build()
            .register("/", catchers![catch_default])
            .attach(Template::fairing())
            .manage(app_config)
            .attach(controllers::IndexController::new("/".to_owned()))
    }

    #[test]
    fn test_application_builds_successfully() {
        let rocket = create_minimal_test_rocket();
        let client = Client::tracked(rocket);
        
        assert!(client.is_ok(), "Application should build without errors");
    }

    #[test]
    fn test_index_route_works() {
        let client = Client::tracked(create_minimal_test_rocket())
            .expect("valid rocket instance");
        
        let response = client.get("/").dispatch();
        
        // Should redirect to blog page
        assert_eq!(response.status(), Status::SeeOther);
    }

    #[test]
    fn test_static_routes_not_found() {
        let client = Client::tracked(create_minimal_test_rocket())
            .expect("valid rocket instance");
        
        // Test that non-existent routes return 404
        let response = client.get("/nonexistent").dispatch();
        
        // Should trigger the catch_default catcher which redirects to "/"
        assert_eq!(response.status(), Status::SeeOther);
    }

    #[test]
    fn test_catch_default_behavior() {
        let client = Client::tracked(create_minimal_test_rocket())
            .expect("valid rocket instance");
        
        // Test various routes that should trigger catch_default
        let test_routes = vec![
            "/invalid",
            "/missing-page", 
            "/api/nonexistent",
            "/admin/missing",
        ];

        for route in test_routes {
            let response = client.get(route).dispatch();
            
            // All should be caught by catch_default and redirect to "/"
            assert_eq!(response.status(), Status::SeeOther);
            
            if let Some(location) = response.headers().get_one("location") {
                assert_eq!(location, "/");
            }
        }
    }

    #[test]
    fn test_application_configuration() {
        // Test that AppConfig can be created and managed by Rocket
        let figment = rocket::Config::figment();
        let app_config = AppConfig::from_figment(&figment);
        
        let rocket = rocket::build().manage(app_config);
        let client = Client::tracked(rocket);
        
        assert!(client.is_ok(), "Rocket should accept AppConfig as managed state");
    }

    #[test]
    fn test_template_fairing_attachment() {
        // Test that Template fairing can be attached without errors
        let rocket = rocket::build().attach(Template::fairing());
        let client = Client::tracked(rocket);
        
        assert!(client.is_ok(), "Template fairing should attach successfully");
    }

    // Note: More comprehensive integration tests would require:
    // 1. Test database setup (SQLite in-memory or PostgreSQL test instance)
    // 2. Database migration execution
    // 3. Service state management (AuthService, BlogService, etc.)
    // 4. Template rendering with real data
    // 5. File upload/download functionality
    // 6. Authentication flow testing
    // 7. Complete CRUD operations through HTTP endpoints
    //
    // These would typically be in separate test files or modules:
    // - database_integration_tests.rs
    // - auth_integration_tests.rs  
    // - blog_integration_tests.rs
    // - file_handling_tests.rs
}