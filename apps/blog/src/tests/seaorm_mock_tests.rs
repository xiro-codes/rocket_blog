#[cfg(test)]
mod seaorm_mock_tests {
    use sea_orm::{MockDatabase, DatabaseBackend, MockExecResult, TransactionTrait};
    use models::{post, account};
    use crate::services::BlogService;
    use crate::tests::mocks::test_data;
    use uuid::Uuid;
    use chrono::Local;

    /// Test BlogService::find_by_id using SeaORM mock database
    #[tokio::test]
    async fn test_blog_service_find_by_id_with_mock() {
        let post_id = test_data::mock_post_id();
        let mock_post = post::Model {
            id: post_id,
            seq_id: 1,
            title: "Test Post".to_string(),
            text: "This is a test post content".to_string(),
            excerpt: Some("Test excerpt".to_string()),
            path: None,
            draft: Some(false),
            date_published: Local::now().naive_local(),
            account_id: test_data::mock_user_id(),
        };

        // Create mock database with expected query results
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                // Mock result for find_by_id query
                vec![mock_post.clone()],
            ])
            .into_connection();

        let blog_service = BlogService::new();
        let result = blog_service.find_by_id(&db, post_id).await;

        assert!(result.is_ok());
        let found_post = result.unwrap();
        assert!(found_post.is_some());
        
        let found_post = found_post.unwrap();
        assert_eq!(found_post.id, post_id);
        assert_eq!(found_post.title, "Test Post");
        assert_eq!(found_post.text, "This is a test post content");
        assert_eq!(found_post.draft, Some(false));
    }

    /// Test BlogService::find_by_id when post is not found
    #[tokio::test]
    async fn test_blog_service_find_by_id_not_found() {
        let post_id = test_data::mock_post_id();

        // Create mock database with empty result
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                // Empty result for find_by_id query (post not found)
                Vec::<post::Model>::new(),
            ])
            .into_connection();

        let blog_service = BlogService::new();
        let result = blog_service.find_by_id(&db, post_id).await;

        assert!(result.is_ok());
        let found_post = result.unwrap();
        assert!(found_post.is_none());
    }

    /// Test BlogService::find_by_seq_id using SeaORM mock database
    #[tokio::test]
    async fn test_blog_service_find_by_seq_id_with_mock() {
        let seq_id = 42;
        let mock_post = post::Model {
            id: test_data::mock_post_id(),
            seq_id,
            title: "Test Post by Seq ID".to_string(),
            text: "This is a test post found by seq_id".to_string(),
            excerpt: Some("Test excerpt for seq_id".to_string()),
            path: None,
            draft: Some(false),
            date_published: Local::now().naive_local(),
            account_id: test_data::mock_user_id(),
        };

        // Create mock database with expected query results
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                // Mock result for find by seq_id query
                vec![mock_post.clone()],
            ])
            .into_connection();

        let blog_service = BlogService::new();
        let result = blog_service.find_by_seq_id(&db, seq_id).await;

        assert!(result.is_ok());
        let found_post = result.unwrap();
        assert_eq!(found_post.seq_id, seq_id);
        assert_eq!(found_post.title, "Test Post by Seq ID");
        assert_eq!(found_post.text, "This is a test post found by seq_id");
    }

    /// Test BlogService::find_by_seq_id when post is not found (should return error)
    #[tokio::test]
    async fn test_blog_service_find_by_seq_id_not_found() {
        let seq_id = 999;

        // Create mock database with empty result
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                // Empty result for find by seq_id query
                Vec::<post::Model>::new(),
            ])
            .into_connection();

        let blog_service = BlogService::new();
        let result = blog_service.find_by_seq_id(&db, seq_id).await;

        // This should return an error because find_by_seq_id calls handle_not_found
        assert!(result.is_err());
        match result.unwrap_err() {
            sea_orm::DbErr::RecordNotFound(msg) => {
                assert!(msg.contains("Post not found"));
            }
            _ => panic!("Expected RecordNotFound error"),
        }
    }

    /// Test BlogService::find_by_seq_id_with_account using SeaORM mock database
    #[tokio::test]
    async fn test_blog_service_find_by_seq_id_with_account() {
        let seq_id = 123;
        let mock_account = account::Model {
            id: test_data::mock_user_id(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: false,
        };

        let mock_post = post::Model {
            id: test_data::mock_post_id(),
            seq_id,
            title: "Post with Account".to_string(),
            text: "This post includes account info".to_string(),
            excerpt: Some("Post with account excerpt".to_string()),
            path: None,
            draft: Some(false),
            date_published: Local::now().naive_local(),
            account_id: test_data::mock_user_id(),
        };

        // Create mock database with expected query results
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                // Mock result for find by seq_id with account join query
                vec![(mock_post.clone(), Some(mock_account.clone()))],
            ])
            .into_connection();

        let blog_service = BlogService::new();
        let result = blog_service.find_by_seq_id_with_account(&db, seq_id).await;

        assert!(result.is_ok());
        let (found_post, found_account) = result.unwrap();
        
        // Verify post data
        assert_eq!(found_post.seq_id, seq_id);
        assert_eq!(found_post.title, "Post with Account");
        
        // Verify account data
        assert!(found_account.is_some());
        let account = found_account.unwrap();
        assert_eq!(account.username, "testuser");
        assert_eq!(account.email, "test@example.com");
        assert!(!account.admin);
    }

    /// Test BlogService::publish_by_seq_id using SeaORM mock database
    #[tokio::test]
    async fn test_blog_service_publish_by_seq_id() {
        let seq_id = 456;
        let mock_post = post::Model {
            id: test_data::mock_post_id(),
            seq_id,
            title: "Draft Post".to_string(),
            text: "This post was a draft".to_string(),
            excerpt: Some("Draft excerpt".to_string()),
            path: None,
            draft: Some(true), // Initially a draft
            date_published: Local::now().naive_local(),
            account_id: test_data::mock_user_id(),
        };

        // Create the published version
        let published_post = post::Model {
            id: mock_post.id,
            seq_id: mock_post.seq_id,
            title: mock_post.title.clone(),
            text: mock_post.text.clone(),
            excerpt: mock_post.excerpt.clone(),
            path: mock_post.path.clone(),
            draft: Some(false), // Published, not draft
            date_published: mock_post.date_published,
            account_id: mock_post.account_id,
        };

        // Create mock database with expected query and exec results
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                // First query: find the draft post
                vec![mock_post.clone()],
            ])
            .append_exec_results([
                // Update operation result
                MockExecResult {
                    last_insert_id: 0,
                    rows_affected: 1,
                },
            ])
            .append_query_results([
                // Return the updated/published post
                vec![published_post.clone()],
            ])
            .into_connection();

        let blog_service = BlogService::new();
        let result = blog_service.publish_by_seq_id(&db, seq_id).await;

        assert!(result.is_ok());
        let published = result.unwrap();
        assert_eq!(published.seq_id, seq_id);
        assert_eq!(published.title, "Draft Post");
        assert_eq!(published.draft, Some(false)); // Should be published now
    }

    /// Test SeaORM mock transaction behavior
    #[tokio::test]
    async fn test_seaorm_mock_transaction_usage() {
        // This test demonstrates how to use SeaORM mock with transactions
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                // Mock transaction and query results
                vec![post::Model {
                    id: test_data::mock_post_id(),
                    seq_id: 1,
                    title: "Transaction Test".to_string(),
                    text: "Testing transaction behavior".to_string(),
                    excerpt: None,
                    path: None,
                    draft: Some(false),
                    date_published: Local::now().naive_local(),
                    account_id: test_data::mock_user_id(),
                }],
            ])
            .into_connection();

        // Test that we can begin a transaction with the mock database
        let transaction = db.begin().await;
        assert!(transaction.is_ok());
        
        // Mock databases support transaction operations
        let _txn = transaction.unwrap();
        // In a real test, you would perform operations within the transaction
        // and then commit or rollback
    }
}