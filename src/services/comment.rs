use std::fmt::format;

use chrono::Local;
use models::comment;
use models::prelude::Comment;
use sea_orm::*;
use uuid::Uuid;

pub struct Service;

impl Service {
    pub fn new() -> Self {
        Self
    }
    pub async fn create(&self, db: &DbConn, post_id: Uuid, data: comment::FormDTO) -> Result<(), DbErr> {
        let comment = comment::ActiveModel {
            id: Set(Uuid::new_v4()),
            text: Set(data.text),
            date_published: Set(Local::now().naive_local()),
            post_id: Set(post_id),
        }.insert(db).await?;
        Ok(())
    }
    pub async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<comment::Model, DbErr> {
        Comment::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Comment")))
    }
    pub async fn find_many_by_post_id(&self, db: &DbConn, post_id: Uuid) -> Result<Vec<comment::Model>, DbErr> {
        Comment::find()
            .filter(comment::Column::PostId.eq(post_id))
            .order_by_desc(comment::Column::DatePublished)
            .all(db)
            .await
    }
}
