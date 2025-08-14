use sea_orm::entity::prelude::*;
// manual addition
use rocket::serde::{Deserialize, Serialize};
use sea_orm::{DerivePartialModel, FromQueryResult};
use rocket::FromForm;
// end manual addition

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "account")]
#[serde(crate = "rocket::serde")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub admin: bool,
}
/// manual addition
#[derive(
    Clone,
    Debug,
    PartialEq,
    DerivePartialModel,
    FromQueryResult,
    Eq,
    Serialize,
    Deserialize,
    FromForm,
)]
#[serde(crate = "rocket::serde")]
#[sea_orm(entity = "Entity")]
pub struct FormDTO {
    pub username: String,
    pub password: String,
}
/// end manual addition
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post::Entity")]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
