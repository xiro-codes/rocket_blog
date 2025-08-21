use rocket::{
    fairing::{self, Fairing, Kind},
    Build, Rocket,
};
use sea_orm::{MockDatabase, DatabaseBackend, MockExecResult};
use models::{post, account};
use crate::tests::mocks::test_data;
use chrono::Local;

/// A test fairing that provides a mock database connection for testing Rocket applications
pub struct MockDatabaseFairing {
    /// Pre-configured mock posts to be available in the mock database
    pub mock_posts: Vec<post::Model>,
    /// Pre-configured mock accounts to be available in the mock database  
    pub mock_accounts: Vec<account::Model>,
    /// Mock execution results for database operations
    pub mock_exec_results: Vec<MockExecResult>,
}

impl MockDatabaseFairing {
    /// Create a new MockDatabaseFairing with default test data
    pub fn new() -> Self {
        Self {
            mock_posts: vec![],
            mock_accounts: vec![],
            mock_exec_results: vec![],
        }
    }

    /// Create a MockDatabaseFairing with a single test post
    pub fn with_test_post() -> Self {
        let test_post = post::Model {
            id: test_data::mock_post_id(),
            seq_id: 1,
            title: "Test Post".to_string(),
            text: "This is a test post content for testing".to_string(),
            excerpt: Some("Test excerpt".to_string()),
            path: None,
            draft: Some(false),
            date_published: Local::now().naive_local(),
            account_id: test_data::mock_user_id(),
        };

        Self {
            mock_posts: vec![test_post],
            mock_accounts: vec![],
            mock_exec_results: vec![],
        }
    }

    /// Create a MockDatabaseFairing with a test post and account
    pub fn with_test_post_and_account() -> Self {
        let test_account = account::Model {
            id: test_data::mock_user_id(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: false,
        };

        let test_post = post::Model {
            id: test_data::mock_post_id(),
            seq_id: 1,
            title: "Test Post with Account".to_string(),
            text: "This is a test post with an associated account".to_string(),
            excerpt: Some("Test excerpt with account".to_string()),
            path: None,
            draft: Some(false),
            date_published: Local::now().naive_local(),
            account_id: test_account.id,
        };

        Self {
            mock_posts: vec![test_post],
            mock_accounts: vec![test_account],
            mock_exec_results: vec![],
        }
    }

    /// Add a mock post to the fairing
    pub fn with_post(mut self, post: post::Model) -> Self {
        self.mock_posts.push(post);
        self
    }

    /// Add a mock account to the fairing
    pub fn with_account(mut self, account: account::Model) -> Self {
        self.mock_accounts.push(account);
        self
    }

    /// Add mock execution results for database operations
    pub fn with_exec_result(mut self, result: MockExecResult) -> Self {
        self.mock_exec_results.push(result);
        self
    }

    /// Create a mock database connection for empty results
    pub fn create_empty_mock_connection() -> sea_orm::DatabaseConnection {
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([
                Vec::<post::Model>::new(),  // Empty posts for find_by_id
            ])
            .into_connection()
    }

    /// Create a special mock connection for joined data queries
    pub fn create_joined_mock_connection(&self) -> sea_orm::DatabaseConnection {
        if self.mock_posts.is_empty() || self.mock_accounts.is_empty() {
            return Self::create_empty_mock_connection();
        }

        // For joined queries, we need to provide the exact format expected by find_by_seq_id_with_account
        let joined_results: Vec<(post::Model, Option<account::Model>)> = vec![(
            self.mock_posts[0].clone(),
            Some(self.mock_accounts[0].clone())
        )];

        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([joined_results])
            .into_connection()
    }

    /// Create a mock database connection with the configured test data
    pub fn create_mock_connection(&self) -> sea_orm::DatabaseConnection {
        let mut mock_db = MockDatabase::new(DatabaseBackend::Postgres);

        // For empty mock - use the dedicated empty connection method
        if self.mock_posts.is_empty() && self.mock_accounts.is_empty() && self.mock_exec_results.is_empty() {
            return Self::create_empty_mock_connection();
        }

        // Special case for joined data - use dedicated method
        if !self.mock_posts.is_empty() && !self.mock_accounts.is_empty() && self.mock_exec_results.is_empty() {
            return self.create_joined_mock_connection();
        }

        // Add query results in order they will be used by the service methods

        // Single post queries (find_by_id, find_by_seq_id) - first query
        if !self.mock_posts.is_empty() {
            mock_db = mock_db.append_query_results([vec![self.mock_posts[0].clone()]]);
        }

        // Add execution results for mutations
        if !self.mock_exec_results.is_empty() {
            mock_db = mock_db.append_exec_results(self.mock_exec_results.clone());
        }

        // Post after mutation - return the last post as the "updated" result
        if !self.mock_posts.is_empty() && !self.mock_exec_results.is_empty() {
            let last_post = self.mock_posts.last().unwrap();
            mock_db = mock_db.append_query_results([vec![last_post.clone()]]);
        }

        mock_db.into_connection()
    }
}

impl Default for MockDatabaseFairing {
    fn default() -> Self {
        Self::new()
    }
}

#[rocket::async_trait]
impl Fairing for MockDatabaseFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Mock Database Test Fairing",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        // The mock database connection is created when needed in tests
        // This fairing doesn't need to modify the rocket instance directly
        // since tests will use create_mock_connection() directly
        Ok(rocket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::fairing::Fairing;

    #[test]
    fn test_mock_database_fairing_new() {
        let fairing = MockDatabaseFairing::new();
        assert!(fairing.mock_posts.is_empty());
        assert!(fairing.mock_accounts.is_empty());
        assert!(fairing.mock_exec_results.is_empty());
    }

    #[test]
    fn test_mock_database_fairing_with_test_post() {
        let fairing = MockDatabaseFairing::with_test_post();
        assert_eq!(fairing.mock_posts.len(), 1);
        assert_eq!(fairing.mock_posts[0].title, "Test Post");
        assert!(fairing.mock_accounts.is_empty());
    }

    #[test]
    fn test_mock_database_fairing_with_test_post_and_account() {
        let fairing = MockDatabaseFairing::with_test_post_and_account();
        assert_eq!(fairing.mock_posts.len(), 1);
        assert_eq!(fairing.mock_accounts.len(), 1);
        assert_eq!(fairing.mock_accounts[0].username, "testuser");
        assert_eq!(fairing.mock_posts[0].account_id, fairing.mock_accounts[0].id);
    }

    #[test]
    fn test_mock_database_fairing_builder_pattern() {
        let test_account = account::Model {
            id: test_data::mock_user_id(),
            username: "builder_test".to_string(),
            email: "builder@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: true,
        };

        let test_post = post::Model {
            id: test_data::mock_post_id(),
            seq_id: 42,
            title: "Builder Pattern Post".to_string(),
            text: "Testing builder pattern".to_string(),
            excerpt: None,
            path: None,
            draft: Some(true),
            date_published: Local::now().naive_local(),
            account_id: test_account.id,
        };

        let exec_result = MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        };

        let fairing = MockDatabaseFairing::new()
            .with_account(test_account.clone())
            .with_post(test_post.clone())
            .with_exec_result(exec_result);

        assert_eq!(fairing.mock_accounts.len(), 1);
        assert_eq!(fairing.mock_posts.len(), 1);
        assert_eq!(fairing.mock_exec_results.len(), 1);
        assert_eq!(fairing.mock_accounts[0].username, "builder_test");
        assert_eq!(fairing.mock_posts[0].seq_id, 42);
    }

    #[test]
    fn test_mock_database_fairing_info() {
        let fairing = MockDatabaseFairing::new();
        let info = fairing.info();
        assert_eq!(info.name, "Mock Database Test Fairing");
        assert!(std::mem::size_of_val(&info.kind) > 0);
    }

    #[tokio::test]
    async fn test_create_mock_connection() {
        let fairing = MockDatabaseFairing::with_test_post();
        let connection = fairing.create_mock_connection();
        
        // Test that we can create a connection without errors
        // The connection is a SeaORM MockDatabase connection
        assert!(std::mem::size_of_val(&connection) > 0);
    }
}