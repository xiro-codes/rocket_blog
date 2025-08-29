//! `SeaORM` Entity, Post Reaction Model

use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;
use query_builder_macro::QueryBuilder;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, QueryBuilder)]
#[sea_orm(table_name = "post_reaction")]
#[serde(crate = "rocket::serde")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub post_id: Uuid,
    pub reaction_type: String,
    pub ip_address: String,
    pub session_id: Option<String>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::post::Entity",
        from = "Column::PostId",
        to = "super::post::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Define common reaction types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum ReactionType {
    Like,
    Love,
    Laugh,
    Wow,
    Sad,
    Angry,
}

impl ReactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReactionType::Like => "like",
            ReactionType::Love => "love",
            ReactionType::Laugh => "laugh",
            ReactionType::Wow => "wow",
            ReactionType::Sad => "sad",
            ReactionType::Angry => "angry",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "like" => Some(ReactionType::Like),
            "love" => Some(ReactionType::Love),
            "laugh" => Some(ReactionType::Laugh),
            "wow" => Some(ReactionType::Wow),
            "sad" => Some(ReactionType::Sad),
            "angry" => Some(ReactionType::Angry),
            _ => None,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            ReactionType::Like => "👍",
            ReactionType::Love => "❤️",
            ReactionType::Laugh => "😂",
            ReactionType::Wow => "😮",
            ReactionType::Sad => "😢",
            ReactionType::Angry => "😡",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            ReactionType::Like,
            ReactionType::Love,
            ReactionType::Laugh,
            ReactionType::Wow,
            ReactionType::Sad,
            ReactionType::Angry,
        ]
    }
}

impl std::fmt::Display for ReactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}