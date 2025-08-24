use chrono::Local;
use models::{comment, dto::CommentFormDTO, prelude::{Comment, Account}};
use sea_orm::*;
use uuid::Uuid;
use rocket::serde::{Deserialize, Serialize};

use crate::services::base::BaseService;

/// Enhanced comment structure with resolved username
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CommentWithUser {
    pub comment: comment::Model,
    pub display_username: String,
}

/// Hierarchical comment structure for threaded display
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CommentThread {
    pub comment: comment::Model,
    pub display_username: String,
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
        // Get all comments for the post with their related account data
        let comments_with_accounts = Comment::find()
            .filter(comment::Column::PostId.eq(post_id))
            .find_also_related(Account)
            .order_by_asc(comment::Column::DatePublished) // Order by date for consistent threading
            .all(db)
            .await?;

        // Convert to CommentWithUser structures
        let comments_with_users: Vec<CommentWithUser> = comments_with_accounts
            .into_iter()
            .map(|(comment, account)| {
                let display_username = if let Some(account) = account {
                    // Use the account's username for authenticated users
                    account.username
                } else if let Some(username) = &comment.username {
                    // Use the stored username for anonymous users
                    username.clone()
                } else {
                    // Fallback for edge cases
                    "Anonymous".to_string()
                };

                CommentWithUser {
                    comment,
                    display_username,
                }
            })
            .collect();

        // Build the hierarchical structure
        let threads = self.build_comment_threads(comments_with_users);
        Ok(threads)
    }

    /// Helper method to build hierarchical comment structure
    fn build_comment_threads(&self, comments: Vec<CommentWithUser>) -> Vec<CommentThread> {
        use std::collections::HashMap;
        
        let mut comment_map: HashMap<Uuid, CommentWithUser> = HashMap::new();
        let mut children_map: HashMap<Option<Uuid>, Vec<Uuid>> = HashMap::new();
        
        // Index comments and group by parent_id
        for comment_with_user in comments {
            let id = comment_with_user.comment.id;
            let parent_id = comment_with_user.comment.parent_id;
            
            comment_map.insert(id, comment_with_user);
            children_map.entry(parent_id).or_insert_with(Vec::new).push(id);
        }
        
        // Build threads starting from top-level comments (parent_id = None)
        let mut threads = Vec::new();
        if let Some(top_level_ids) = children_map.get(&None) {
            for &comment_id in top_level_ids {
                if let Some(comment_with_user) = comment_map.get(&comment_id) {
                    let thread = self.build_thread_recursive(comment_with_user.clone(), &comment_map, &children_map);
                    threads.push(thread);
                }
            }
        }
        
        threads
    }

    /// Recursively build a comment thread
    fn build_thread_recursive(
        &self,
        comment_with_user: CommentWithUser,
        comment_map: &std::collections::HashMap<Uuid, CommentWithUser>,
        children_map: &std::collections::HashMap<Option<Uuid>, Vec<Uuid>>,
    ) -> CommentThread {
        let mut replies = Vec::new();
        
        if let Some(child_ids) = children_map.get(&Some(comment_with_user.comment.id)) {
            for &child_id in child_ids {
                if let Some(child_comment_with_user) = comment_map.get(&child_id) {
                    let child_thread = self.build_thread_recursive(
                        child_comment_with_user.clone(),
                        comment_map,
                        children_map,
                    );
                    replies.push(child_thread);
                }
            }
        }
        
        CommentThread {
            comment: comment_with_user.comment,
            display_username: comment_with_user.display_username,
            replies,
        }
    }
}
