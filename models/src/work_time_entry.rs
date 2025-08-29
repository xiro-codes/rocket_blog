//! `SeaORM` Entity for work time entries

use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "work_time_entry")]
#[serde(crate = "rocket::serde")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub user_role_id: Uuid,
    pub pay_period_id: Option<Uuid>,
    pub start_time: DateTime,
    pub end_time: Option<DateTime>,
    pub duration: Option<i32>, // Duration in minutes
    pub description: Option<String>,
    pub project: Option<String>,
    pub is_active: bool, // For active time tracking
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
    #[sea_orm(
        belongs_to = "super::user_role::Entity",
        from = "Column::UserRoleId",
        to = "super::user_role::Column::Id"
    )]
    UserRole,
    #[sea_orm(
        belongs_to = "super::pay_period::Entity",
        from = "Column::PayPeriodId",
        to = "super::pay_period::Column::Id"
    )]
    PayPeriod,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::user_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRole.def()
    }
}

impl Related<super::pay_period::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PayPeriod.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}