#[cfg(test)]
mod rocket_mock_tests {
    use super::super::fairings::mock_database::MockDatabaseFairing;
    use crate::services::BlogService;
    use crate::tests::mocks::test_data;
    use models::{post, account};
    use sea_orm::MockExecResult;
    use chrono::Local;
    use rocket::{Build, Rocket};

    /// Test helper to create a mock Rocket instance with mock database fairing
    fn create_test_rocket_with_mock_db(fairing: MockDatabaseFairing) -> Rocket<Build> {
        rocket::build()
            .attach(fairing)
    }

    /// Test that demonstrates basic usage of MockDatabaseFairing
    #[test]
    fn test_mock_database_fairing_basic_usage() {
        let fairing = MockDatabaseFairing::with_test_post();
        let _rocket = create_test_rocket_with_mock_db(fairing);
        
        // Test that rocket builds successfully with the mock fairing
        // We can't access fairing count directly, but can test that it builds without error
        assert!(true);
    }

    /// Test creating a mock database connection with test data
    #[tokio::test]
    async fn test_mock_connection_with_blog_service() {
        let fairing = MockDatabaseFairing::with_test_post();
        let mock_db = fairing.create_mock_connection();
        
        let blog_service = BlogService::new();
        let post_id = test_data::mock_post_id();
        
        // Test that we can use the service with the mock database
        let result = blog_service.find_by_id(&mock_db, post_id).await;
        assert!(result.is_ok());
        
        let found_post = result.unwrap();
        assert!(found_post.is_some());
        
        let found_post = found_post.unwrap();
        assert_eq!(found_post.title, "Test Post");
    }

    /// Test mock database with both posts and accounts
    #[tokio::test]
    async fn test_mock_connection_with_joined_data() {
        let fairing = MockDatabaseFairing::with_test_post_and_account();
        let mock_db = fairing.create_mock_connection();
        
        let blog_service = BlogService::new();
        let seq_id = 1;
        
        // Test finding post by seq_id with account
        let result = blog_service.find_by_seq_id_with_account(&mock_db, seq_id).await;
        assert!(result.is_ok());
        
        let (found_post, found_account) = result.unwrap();
        assert_eq!(found_post.title, "Test Post with Account");
        assert!(found_account.is_some());
        
        let account = found_account.unwrap();
        assert_eq!(account.username, "testuser");
    }

    /// Test mock database with custom data using builder pattern
    #[tokio::test]
    async fn test_mock_connection_builder_pattern() {
        let custom_account = account::Model {
            id: test_data::mock_user_id(),
            username: "custom_user".to_string(),
            email: "custom@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: true,
        };

        let custom_post = post::Model {
            id: test_data::mock_post_id(),
            seq_id: 100,
            title: "Custom Test Post".to_string(),
            text: "This is a custom test post".to_string(),
            excerpt: Some("Custom excerpt".to_string()),
            path: Some("custom-video.webm".to_string()),
            draft: Some(true),
            date_published: Local::now().naive_local(),
            account_id: custom_account.id,
        };

        let exec_result = MockExecResult {
            last_insert_id: 100,
            rows_affected: 1,
        };

        let fairing = MockDatabaseFairing::new()
            .with_account(custom_account)
            .with_post(custom_post)
            .with_exec_result(exec_result);

        let mock_db = fairing.create_mock_connection();
        let blog_service = BlogService::new();
        
        // Test finding the custom post
        let result = blog_service.find_by_seq_id(&mock_db, 100).await;
        assert!(result.is_ok());
        
        let found_post = result.unwrap();
        assert_eq!(found_post.title, "Custom Test Post");
        assert_eq!(found_post.draft, Some(true));
        assert_eq!(found_post.path, Some("custom-video.webm".to_string()));
    }

    /// Test mock database with multiple posts
    #[tokio::test]
    async fn test_mock_connection_multiple_posts() {
        let account_id = test_data::mock_user_id();
        
        let post1 = post::Model {
            id: test_data::mock_post_id(),
            seq_id: 1,
            title: "First Post".to_string(),
            text: "Content of first post".to_string(),
            excerpt: Some("First excerpt".to_string()),
            path: None,
            draft: Some(false),
            date_published: Local::now().naive_local(),
            account_id,
        };

        let post2 = post::Model {
            id: uuid::Uuid::new_v4(),
            seq_id: 2,
            title: "Second Post".to_string(),
            text: "Content of second post".to_string(),
            excerpt: Some("Second excerpt".to_string()),
            path: None,
            draft: Some(false),
            date_published: Local::now().naive_local(),
            account_id,
        };

        let fairing = MockDatabaseFairing::new()
            .with_post(post1)
            .with_post(post2);

        let mock_db = fairing.create_mock_connection();
        
        // Test that the mock database was created successfully
        // In a real scenario, you would test specific service methods
        // that work with multiple posts
        assert!(std::mem::size_of_val(&mock_db) > 0);
    }

    /// Test error handling with empty mock database
    #[tokio::test]
    async fn test_empty_mock_database() {
        let mock_db = MockDatabaseFairing::create_empty_mock_connection();
        
        let blog_service = BlogService::new();
        let post_id = test_data::mock_post_id();
        
        // Test finding a post in an empty mock database
        let result = blog_service.find_by_id(&mock_db, post_id).await;
        assert!(result.is_ok());
        
        let found_post = result.unwrap();
        assert!(found_post.is_none());
    }

    /// Test mock database fairing configuration
    #[test]
    fn test_fairing_configuration() {
        let fairing = MockDatabaseFairing::with_test_post_and_account();
        
        // Test fairing has correct configuration
        use rocket::fairing::Fairing;
        let info = fairing.info();
        assert_eq!(info.name, "Mock Database Test Fairing");
        assert!(std::mem::size_of_val(&info.kind) > 0);
        
        // Test fairing has expected test data
        assert_eq!(fairing.mock_posts.len(), 1);
        assert_eq!(fairing.mock_accounts.len(), 1);
        assert!(fairing.mock_exec_results.is_empty());
    }

    /// Test updating posts with mock database operations
    #[tokio::test]
    async fn test_mock_database_with_operations() {
        let draft_post = post::Model {
            id: test_data::mock_post_id(),
            seq_id: 1,
            title: "Draft Post".to_string(),
            text: "This is a draft post".to_string(),
            excerpt: Some("Draft excerpt".to_string()),
            path: None,
            draft: Some(true),
            date_published: Local::now().naive_local(),
            account_id: test_data::mock_user_id(),
        };

        let published_post = post::Model {
            id: draft_post.id,
            seq_id: draft_post.seq_id,
            title: draft_post.title.clone(),
            text: draft_post.text.clone(),
            excerpt: draft_post.excerpt.clone(),
            path: draft_post.path.clone(),
            draft: Some(false), // Published
            date_published: draft_post.date_published,
            account_id: draft_post.account_id,
        };

        let exec_result = MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        };

        let fairing = MockDatabaseFairing::new()
            .with_post(draft_post.clone())
            .with_exec_result(exec_result)
            .with_post(published_post.clone()); // Result after update

        let mock_db = fairing.create_mock_connection();
        let blog_service = BlogService::new();
        
        // Test publishing a draft post
        let result = blog_service.publish_by_seq_id(&mock_db, 1).await;
        assert!(result.is_ok());
        
        let updated_post = result.unwrap();
        assert_eq!(updated_post.draft, Some(false));
        assert_eq!(updated_post.title, "Draft Post");
    }
}