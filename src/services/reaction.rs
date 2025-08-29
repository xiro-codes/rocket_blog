use crate::services::base::BaseService;
use crate::{impl_service_with_base, services::base::ServiceHelpers};
use chrono::Local;
use models::{
    post_reaction::{self, ReactionType},
    prelude::PostReaction,
};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, Set, ActiveModelTrait, TryIntoModel, PaginatorTrait};
use std::collections::HashMap;
use uuid::Uuid;

pub struct Service {
    base: BaseService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ReactionCount {
    pub reaction_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PostReactionSummary {
    pub post_id: Uuid,
    pub total_reactions: i64,
    pub user_reaction: Option<String>,
    pub reactions: Vec<ReactionCount>,
}

impl Service {
    /// Add or update a reaction to a post
    pub async fn add_reaction(
        &self,
        db: &DbConn,
        post_id: Uuid,
        reaction_type: &str,
        ip_address: &str,
        session_id: Option<&str>,
    ) -> Result<post_reaction::Model, sea_orm::DbErr> {
        // Validate reaction type
        if ReactionType::from_str(reaction_type).is_none() {
            return Err(sea_orm::DbErr::Custom(format!(
                "Invalid reaction type: {}",
                reaction_type
            )));
        }

        // Check if user already has a reaction for this post
        if let Some(existing) = self.get_user_reaction(db, post_id, ip_address).await? {
            // If same reaction type, remove it (toggle off)
            if existing.reaction_type == reaction_type {
                return self.remove_reaction(db, post_id, ip_address).await;
            } else {
                // Update to new reaction type
                let mut existing: post_reaction::ActiveModel = existing.into();
                existing.reaction_type = Set(reaction_type.to_string());
                return existing.save(db).await.map(|active_model| {
                    match active_model.try_into_model() {
                        Ok(model) => model,
                        Err(_) => panic!("Failed to convert active model to model"),
                    }
                });
            }
        }

        // Create new reaction
        let reaction = post_reaction::ActiveModel {
            id: Set(Uuid::new_v4()),
            post_id: Set(post_id),
            reaction_type: Set(reaction_type.to_string()),
            ip_address: Set(ip_address.to_string()),
            session_id: Set(session_id.map(|s| s.to_string())),
            created_at: Set(Local::now().naive_local()),
        };

        reaction.insert(db).await
    }

    /// Remove a user's reaction from a post
    pub async fn remove_reaction(
        &self,
        db: &DbConn,
        post_id: Uuid,
        ip_address: &str,
    ) -> Result<post_reaction::Model, sea_orm::DbErr> {
        let reaction = PostReaction::find()
            .filter(post_reaction::Column::PostId.eq(post_id))
            .filter(post_reaction::Column::IpAddress.eq(ip_address))
            .one(db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(
                "No reaction found for this user and post".to_string(),
            ))?;

        let reaction_to_return = reaction.clone();
        PostReaction::delete_by_id(reaction.id).exec(db).await?;
        Ok(reaction_to_return)
    }

    /// Get a user's current reaction for a post
    pub async fn get_user_reaction(
        &self,
        db: &DbConn,
        post_id: Uuid,
        ip_address: &str,
    ) -> Result<Option<post_reaction::Model>, sea_orm::DbErr> {
        PostReaction::find()
            .filter(post_reaction::Column::PostId.eq(post_id))
            .filter(post_reaction::Column::IpAddress.eq(ip_address))
            .one(db)
            .await
    }

    /// Get reaction counts for a post
    pub async fn get_reaction_counts(
        &self,
        db: &DbConn,
        post_id: Uuid,
    ) -> Result<Vec<ReactionCount>, sea_orm::DbErr> {
        use sea_orm::{sea_query::Expr, QuerySelect};

        let counts: Vec<(String, i64)> = PostReaction::find()
            .filter(post_reaction::Column::PostId.eq(post_id))
            .select_only()
            .column(post_reaction::Column::ReactionType)
            .column_as(Expr::col(post_reaction::Column::Id).count(), "count")
            .group_by(post_reaction::Column::ReactionType)
            .into_tuple()
            .all(db)
            .await?;

        Ok(counts
            .into_iter()
            .map(|(reaction_type, count)| ReactionCount {
                reaction_type,
                count,
            })
            .collect())
    }

    /// Get comprehensive reaction summary for a post
    pub async fn get_post_reaction_summary(
        &self,
        db: &DbConn,
        post_id: Uuid,
        ip_address: &str,
    ) -> Result<PostReactionSummary, sea_orm::DbErr> {
        let reactions = self.get_reaction_counts(db, post_id).await?;
        let total_reactions = reactions.iter().map(|r| r.count).sum();
        let user_reaction = self
            .get_user_reaction(db, post_id, ip_address)
            .await?
            .map(|r| r.reaction_type);

        Ok(PostReactionSummary {
            post_id,
            total_reactions,
            user_reaction,
            reactions,
        })
    }

    /// Get reaction summaries for multiple posts (for list views)
    pub async fn get_posts_reaction_summaries(
        &self,
        db: &DbConn,
        post_ids: &[Uuid],
        ip_address: &str,
    ) -> Result<HashMap<Uuid, PostReactionSummary>, sea_orm::DbErr> {
        let mut summaries = HashMap::new();

        for &post_id in post_ids {
            let summary = self.get_post_reaction_summary(db, post_id, ip_address).await?;
            summaries.insert(post_id, summary);
        }

        Ok(summaries)
    }

    /// Get total reaction count for a post (simple count)
    pub async fn get_total_reaction_count(
        &self,
        db: &DbConn,
        post_id: Uuid,
    ) -> Result<i64, sea_orm::DbErr> {
        use sea_orm::QuerySelect;

        let count = PostReaction::find()
            .filter(post_reaction::Column::PostId.eq(post_id))
            .count(db)
            .await?;

        Ok(count as i64)
    }
}

impl_service_with_base!(Service);