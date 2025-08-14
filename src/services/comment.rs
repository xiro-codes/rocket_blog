use std::fmt::format;

use chrono::Local;
use models::comment;
use models::dto::CommentFormDTO;
use models::prelude::Comment;
use sea_orm::*;
use uuid::Uuid;

use crate::services::base::BaseService;

pub struct Service {
    base: BaseService,
}

impl Service {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }
    
    pub async fn create(&self, db: &DbConn, post_id: Uuid, data: CommentFormDTO) -> Result<(), DbErr> {
        let _comment = comment::ActiveModel {
            id: Set(BaseService::generate_id()),
            text: Set(data.text),
            date_published: Set(Local::now().naive_local()),
            post_id: Set(post_id),
        }.insert(db).await?;
        Ok(())
    }
    
    pub async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<comment::Model, DbErr> {
        let result = Comment::find_by_id(id).one(db).await?;
        BaseService::handle_not_found(result, "Comment")
    }
    
    pub async fn find_many_by_post_id(&self, db: &DbConn, post_id: Uuid) -> Result<Vec<comment::Model>, DbErr> {
        Comment::find()
            .filter(comment::Column::PostId.eq(post_id))
            .order_by_desc(comment::Column::DatePublished)
            .all(db)
            .await
    }
}
