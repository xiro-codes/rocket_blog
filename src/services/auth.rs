use std::collections::HashMap;

use models::{account, dto::{AccountFormDTO, AdminCreateFormDTO}, prelude::Account};
use pwhash::bcrypt;
use rocket::futures::lock::Mutex;
use sea_orm::*;
use uuid::Uuid;

use crate::services::base::BaseService;

pub struct Service {
    base: BaseService,
    token_map: Mutex<HashMap<Token, AccountId>>,
}
type Token = Uuid;
type AccountId = Uuid;

impl Service {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
            token_map: Mutex::new(HashMap::new()),
        }
    }

    pub async fn login(&self, db: &DbConn, data: AccountFormDTO) -> Result<Token, DbErr> {
        let ac = Account::find()
            .filter(account::Column::Username.eq(data.username))
            .one(db)
            .await
            .unwrap();
        if let Some(ac) = ac {
            let auth = bcrypt::verify(data.password, &ac.password);
            if !auth {
                return Err(DbErr::Custom("".to_owned()));
            }
            let token = {
                let mut tm = self.token_map.lock().await;
                let token = BaseService::generate_id();
                tm.insert(token, ac.id);
                token
            };
            return Ok(token);
        }
        Err(DbErr::Custom("".to_owned()))
    }

    pub async fn check_token(&self, db: &DbConn, token: Token) -> Option<account::Model> {
        let id = {
            let tm = self.token_map.lock().await;
            tm.get(&token).cloned()
        };
        if let Some(id) = id {
            let account = Account::find_by_id(id).one(db).await;
            return account.unwrap();
        }
        None
    }

    pub async fn has_any_accounts(&self, db: &DbConn) -> bool {
        match Account::find().limit(1).one(db).await {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    pub async fn create_admin_account(&self, db: &DbConn, data: AdminCreateFormDTO) -> Result<account::Model, DbErr> {
        // Check if any accounts exist first
        if self.has_any_accounts(db).await {
            return Err(DbErr::Custom("Admin account already exists".to_owned()));
        }

        let pw = bcrypt::hash(&data.password).map_err(|_| DbErr::Custom("Password hashing failed".to_owned()))?;
        let account = account::ActiveModel {
            id: Set(BaseService::generate_id()),
            username: Set(data.username),
            password: Set(pw),
            email: Set(data.email),
            admin: Set(true),
        }
        .insert(db)
        .await?;

        Ok(account)
    }
}
