//! `SeaORM` Entity, Poll Vote Model

use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "poll_vote")]
#[serde(crate = "rocket::serde")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub poll_id: Uuid,
    pub option_id: Uuid,
    pub ip_address: String,
    pub session_id: Option<String>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::poll::Entity",
        from = "Column::PollId",
        to = "super::poll::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Poll,
    #[sea_orm(
        belongs_to = "super::poll_option::Entity",
        from = "Column::OptionId",
        to = "super::poll_option::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    PollOption,
}

impl Related<super::poll::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Poll.def()
    }
}

impl Related<super::poll_option::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PollOption.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}