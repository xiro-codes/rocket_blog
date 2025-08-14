use std::collections::HashMap;

use models::account;
use models::prelude::Account;
use pwhash::bcrypt;
use rocket::futures::lock::Mutex;
use sea_orm::*;
use uuid::Uuid;
use crate::generic::{CrudService, ErrorUtils};

pub struct Service {
    token_map: Mutex<HashMap<Token, AccountId>>
}
type Token = Uuid;
type AccountId = Uuid;

impl Service {
    pub fn new() -> Self {
        Self { token_map: Mutex::new(HashMap::new())}
    }
    
    pub async fn login(&self, db: &DbConn, data: account::FormDTO) -> Result<Token, DbErr> {
        let ac = Account::find()
            .filter(account::Column::Username.eq(data.username))
            .one(db)
            .await
            .unwrap();
        if let Some(ac)= ac {
            let auth = bcrypt::verify( data.password, &ac.password);
            if !auth {
                return Err(DbErr::Custom("Invalid credentials".to_owned()))
            }
            let token = {
                let mut tm = self.token_map.lock().await;
                let token = Uuid::new_v4();
                tm.insert(token, ac.id);
                token
            };
            return Ok(token)
        }
        Err(DbErr::Custom("Invalid credentials".to_owned()))
    }
    
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

// Implement generic CRUD operations for accounts (basic implementation)
impl CrudService<account::Model, account::FormDTO, account::FormDTO, Uuid> for Service {
    async fn create(&self, _db: &DbConn, _data: account::FormDTO) -> Result<account::Model, DbErr> {
        // Account creation typically requires password hashing and other logic
        Err(DbErr::Custom("Use specific account creation method instead".to_owned()))
    }
    
    async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<Option<account::Model>, DbErr> {
        Account::find_by_id(id).one(db).await
    }
    
    async fn update_by_id(&self, db: &DbConn, id: Uuid, data: account::FormDTO) -> Result<account::Model, DbErr> {
        let mut account: account::ActiveModel = Account::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| ErrorUtils::not_found("Account", id))
            .map(Into::into)?;
        account.username = Set(data.username);
        // Note: password would need hashing in a real implementation
        account.password = Set(data.password);
        account.update(db).await
    }
    
    async fn delete_by_id(&self, db: &DbConn, id: Uuid) -> Result<(), DbErr> {
        let account = Account::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| ErrorUtils::not_found("Account", id))?;
        account.delete(db).await.map(|_| ())
    }
}
