//! `SeaORM` Entity, Poll Model

use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "poll")]
#[serde(crate = "rocket::serde")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub seq_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub account_id: Uuid,
    pub date_published: DateTime,
    pub active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Account,
    #[sea_orm(has_many = "super::poll_option::Entity")]
    PollOption,
    #[sea_orm(has_many = "super::poll_vote::Entity")]
    PollVote,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::poll_option::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PollOption.def()
    }
}

impl Related<super::poll_vote::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PollVote.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}