use std::collections::HashMap;

use models::account;
use models::prelude::Account;
use pwhash::bcrypt;
use rocket::futures::lock::Mutex;
use sea_orm::*;
use uuid::Uuid;

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
