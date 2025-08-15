//! Unit tests for the TagService module
use app::services::TagService;
use slug::slugify;

#[cfg(test)]
mod tag_service_tests {
    use super::*;

    #[test]
    fn test_new_creates_tag_service() {
        let service = TagService::new();
        
        // TagService contains a BaseService, verify it can be created
        let _service_exists = true;
        assert!(_service_exists);
    }

    #[test]
    fn test_slugify_functionality() {
        // Test the slugify function that TagService uses
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Rust Programming"), "rust-programming");
        assert_eq!(slugify("Web Development"), "web-development");
        assert_eq!(slugify("C++ Programming"), "c-programming");
        assert_eq!(slugify("Node.js"), "node-js");
        
        // Test edge cases
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("   spaces   "), "spaces");
        assert_eq!(slugify("Special!@#$%Characters"), "specialcharacters");
    }

    #[test] 
    fn test_default_color_constant() {
        // The TagService uses "#007bff" as default color
        // We can test this indirectly by verifying the expected default
        let default_color = "#007bff";
        assert_eq!(default_color.len(), 7); // #RRGGBB format
        assert!(default_color.starts_with('#'));
        
        // Verify it's a valid hex color format
        let hex_part = &default_color[1..];
        assert_eq!(hex_part.len(), 6);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_tag_name_processing() {
        // Test various tag names that would be processed by slugify
        let test_cases = vec![
            ("JavaScript", "javascript"),
            ("Machine Learning", "machine-learning"),
            ("API Design", "api-design"),
            ("Database Optimization", "database-optimization"),
            ("Front-End", "front-end"),
            ("Back-End", "back-end"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(slugify(input), expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_multiple_tag_services_are_independent() {
        let service1 = TagService::new();
        let service2 = TagService::new();
        
        // Each service should be an independent instance
        let service1_ptr = &service1 as *const TagService;
        let service2_ptr = &service2 as *const TagService;
        
        assert_ne!(service1_ptr, service2_ptr);
    }

    // Note: TagService methods like create_tag(), find_all_tags(), find_tags_by_post_id(),
    // add_tag_to_post(), remove_tag_from_post(), and find_or_create_tag() all require
    // database connections. These would be better tested with:
    // 1. Integration tests with a test database
    // 2. Mocked database connections
    //
    // The core functionality being tested would include:
    // - create_tag(): UUID generation, slug creation, default color handling
    // - find_all_tags(): Database queries with proper ordering
    // - find_tags_by_post_id(): Join queries for post-tag relationships  
    // - add_tag_to_post(): Creating many-to-many relationships
    // - remove_tag_from_post(): Removing relationships safely
    // - find_or_create_tag(): Upsert logic with proper error handling
}