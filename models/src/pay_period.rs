//! PayPeriod entity for managing pay periods in work time tracking

use sea_orm::{DeriveEntityModel, DeriveRelation, EntityTrait, EnumIter, Related, RelationTrait, RelationDef, prelude::*};
use uuid::Uuid;
use rocket::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[sea_orm(table_name = "pay_period")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub period_name: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
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