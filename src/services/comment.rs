use chrono::Local;
use models::{comment, dto::CommentFormDTO, prelude::Comment};
use sea_orm::*;
use uuid::Uuid;
use rocket::serde::{Deserialize, Serialize};

use crate::services::base::BaseService;

/// Hierarchical comment structure for threaded display
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CommentThread {
    pub comment: comment::Model,
    pub replies: Vec<CommentThread>,
}

pub struct Service {
    base: BaseService,
}

impl Service {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    pub async fn create(
        &self,
        db: &DbConn,
        post_id: Uuid,
        data: CommentFormDTO,
        user_id: Option<Uuid>, // None for anonymous users
    ) -> Result<(), DbErr> {
        let username = if user_id.is_some() {
            // For authenticated users, don't store username (will use account relation)
            None
        } else {
            // For anonymous users, use provided username or default to "Anonymous"
            Some(data.username.unwrap_or_else(|| "Anonymous".to_string()))
        };

        let _comment = comment::ActiveModel {
            id: Set(BaseService::generate_id()),
            text: Set(data.text),
            date_published: Set(Local::now().naive_local()),
            post_id: Set(post_id),
            user_id: Set(user_id),
            username: Set(username),
            parent_id: Set(data.parent_id),
        }
        .insert(db)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<comment::Model, DbErr> {
        let result = Comment::find_by_id(id).one(db).await?;
        BaseService::handle_not_found(result, "Comment")
    }

    pub async fn find_many_by_post_id(
        &self,
        db: &DbConn,
        post_id: Uuid,
    ) -> Result<Vec<comment::Model>, DbErr> {
        Comment::find()
            .filter(comment::Column::PostId.eq(post_id))
            .order_by_desc(comment::Column::DatePublished)
            .all(db)
            .await
    }

    /// Get comments organized in a hierarchical thread structure
    pub async fn find_threaded_by_post_id(
        &self,
        db: &DbConn,
        post_id: Uuid,
    ) -> Result<Vec<CommentThread>, DbErr> {
        // Get all comments for the post
        let all_comments = Comment::find()
            .filter(comment::Column::PostId.eq(post_id))
            .order_by_asc(comment::Column::DatePublished) // Order by date for consistent threading
            .all(db)
            .await?;

        // Build the hierarchical structure
        let threads = self.build_comment_threads(all_comments);
        Ok(threads)
    }

    /// Helper method to build hierarchical comment structure
    fn build_comment_threads(&self, comments: Vec<comment::Model>) -> Vec<CommentThread> {
        use std::collections::HashMap;
        
        let mut comment_map: HashMap<Uuid, comment::Model> = HashMap::new();
        let mut children_map: HashMap<Option<Uuid>, Vec<Uuid>> = HashMap::new();
        
        // Index comments and group by parent_id
        for comment in comments {
            let id = comment.id;
            let parent_id = comment.parent_id;
            
            comment_map.insert(id, comment);
            children_map.entry(parent_id).or_insert_with(Vec::new).push(id);
        }
        
        // Build threads starting from top-level comments (parent_id = None)
        let mut threads = Vec::new();
        if let Some(top_level_ids) = children_map.get(&None) {
            for &comment_id in top_level_ids {
                if let Some(comment) = comment_map.get(&comment_id) {
                    let thread = self.build_thread_recursive(comment.clone(), &comment_map, &children_map);
                    threads.push(thread);
                }
            }
        }
        
        threads
    }

    /// Recursively build a comment thread
    fn build_thread_recursive(
        &self,
        comment: comment::Model,
        comment_map: &std::collections::HashMap<Uuid, comment::Model>,
        children_map: &std::collections::HashMap<Option<Uuid>, Vec<Uuid>>,
    ) -> CommentThread {
        let mut replies = Vec::new();
        
        if let Some(child_ids) = children_map.get(&Some(comment.id)) {
            for &child_id in child_ids {
                if let Some(child_comment) = comment_map.get(&child_id) {
                    let child_thread = self.build_thread_recursive(
                        child_comment.clone(),
                        comment_map,
                        children_map,
                    );
                    replies.push(child_thread);
                }
            }
        }
        
        CommentThread { comment, replies }
    }
}
