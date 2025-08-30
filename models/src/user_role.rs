//! `SeaORM` Entity for user roles

use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_role")]
#[serde(crate = "rocket::serde")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub role_name: String,
    pub hourly_wage: f64,
    pub currency: String,
    pub is_tipped: bool,
    pub is_active: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
    #[sea_orm(has_many = "super::work_time_entry::Entity")]
    WorkTimeEntry,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::work_time_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WorkTimeEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}