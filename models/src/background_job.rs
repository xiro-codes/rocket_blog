//! `SeaORM` Entity for background jobs

use rocket::serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "background_job")]
#[serde(crate = "rocket::serde")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub job_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub status: String,
    pub error_message: Option<String>,
    pub job_data: Option<Json>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Job type constants
pub const JOB_TYPE_YOUTUBE_DOWNLOAD: &str = "youtube_download";

// Entity type constants  
pub const ENTITY_TYPE_POST: &str = "post";

// Status constants
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_DOWNLOADING: &str = "downloading";
pub const STATUS_COMPLETED: &str = "completed";
pub const STATUS_FAILED: &str = "failed";