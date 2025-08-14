use chrono::Local;
use models::comment;
use models::prelude::Comment;
use sea_orm::*;
use uuid::Uuid;
use crate::generic::CrudService;

pub struct Service;

impl Service {
    pub fn new() -> Self {
        Self
    }
    
    /// Create comment for specific post - comment-specific method
    pub async fn create(&self, db: &DbConn, post_id: Uuid, data: comment::FormDTO) -> Result<(), DbErr> {
        let _comment = comment::ActiveModel {
            id: Set(Uuid::new_v4()),
            text: Set(data.text),
            date_published: Set(Local::now().naive_local()),
            post_id: Set(post_id),
        }.insert(db).await?;
        Ok(())
    }
    
    pub async fn find_many_by_post_id(&self, db: &DbConn, post_id: Uuid) -> Result<Vec<comment::Model>, DbErr> {
        Comment::find()
            .filter(comment::Column::PostId.eq(post_id))
            .order_by_desc(comment::Column::DatePublished)
            .all(db)
            .await
    }
}

// Implement generic CRUD operations for comments
impl CrudService<comment::Model, comment::FormDTO, comment::FormDTO, Uuid> for Service {
    async fn create(&self, _db: &DbConn, _data: comment::FormDTO) -> Result<comment::Model, DbErr> {
        // Comments need post ID, so this generic method isn't directly used
        Err(DbErr::Custom("Use create with post ID instead".to_owned()))
    }
    
    async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<Option<comment::Model>, DbErr> {
        Comment::find_by_id(id).one(db).await
    }
    
    async fn update_by_id(&self, db: &DbConn, id: Uuid, data: comment::FormDTO) -> Result<comment::Model, DbErr> {
        let mut comment: comment::ActiveModel = Comment::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Comment with id: {}", id)))
            .map(Into::into)?;
        comment.text = Set(data.text);
        comment.update(db).await
    }
    
    async fn delete_by_id(&self, db: &DbConn, id: Uuid) -> Result<(), DbErr> {
        let comment = Comment::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Comment with id: {}", id)))?;
        comment.delete(db).await.map(|_| ())
    }
}
