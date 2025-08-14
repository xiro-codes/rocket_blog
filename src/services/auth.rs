//! Authentication service for user login and session management.
//!
//! This service provides authentication functionality including password verification,
//! token generation, and session management using in-memory token storage.

use std::collections::HashMap;

use models::account;
use models::prelude::Account;
use pwhash::bcrypt;
use rocket::futures::lock::Mutex;
use sea_orm::*;
use uuid::Uuid;

/// Authentication service for handling user login and session management.
///
/// This service manages user authentication workflows including:
/// - Password verification using bcrypt hashing
/// - Session token generation and storage
/// - User session validation
/// - In-memory token-to-user mapping
pub struct Service {
    /// Thread-safe mapping of authentication tokens to user account IDs
    token_map: Mutex<HashMap<Token, AccountId>>
}

/// Type alias for authentication tokens (UUIDs)
type Token = Uuid;
/// Type alias for user account IDs (UUIDs) 
type AccountId = Uuid;

impl Service {
    /// Creates a new AuthService instance with empty token storage.
    ///
    /// # Returns
    ///
    /// A new Service instance ready for authentication operations.
    pub fn new() -> Self {
        Self { token_map: Mutex::new(HashMap::new())}
    }

    /// Authenticates a user with their credentials and creates a session token.
    ///
    /// This method:
    /// 1. Looks up the user by username in the database
    /// 2. Verifies the provided password against the stored bcrypt hash
    /// 3. Generates a new session token if authentication succeeds
    /// 4. Stores the token-to-user mapping in memory
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection for user lookup
    /// * `data` - Form data containing username and password
    ///
    /// # Returns
    ///
    /// * `Ok(Token)` - Authentication token on successful login
    /// * `Err(DbErr)` - Database or authentication error
    pub async fn login(&self, db: &DbConn, data: account::FormDTO) -> Result<Token, DbErr> {
        let ac = Account::find()
            .filter(account::Column::Username.eq(data.username))
            .one(db)
            .await
            .unwrap();
        if let Some(ac)= ac {
            let auth = bcrypt::verify( data.password, &ac.password);
            if !auth {
                return Err(DbErr::Custom("".to_owned()))
            }
            let token = {
                let mut tm = self.token_map.lock().await;
                let token = Uuid::new_v4();
                tm.insert(token, ac.id);
                token
            };
            return Ok(token)
        }
        Err(DbErr::Custom("".to_owned()))
    }

    /// Validates a session token and returns the associated user account.
    ///
    /// This method checks if the provided token exists in the active session
    /// storage and retrieves the corresponding user account from the database.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection for user lookup
    /// * `token` - Session token to validate
    ///
    /// # Returns
    ///
    /// * `Some(account::Model)` - User account if token is valid
    /// * `None` - If token is invalid or user not found
    pub async fn check_token(&self, db: &DbConn, token: Token) -> Option<account::Model> {
        let id = {
            let tm = self.token_map.lock().await;
            tm.get(&token).cloned()
        };
        if let Some(id) = id {
            let account = Account::find_by_id(id).one(db).await;
            return account.unwrap()
        }
        None
    }
}
