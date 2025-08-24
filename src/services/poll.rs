use crate::services::base::BaseService;
use chrono::Local;
use models::{
    poll::{self, Entity as Poll},
    poll_option::{self, Entity as PollOptionEntity},
    poll_vote::{self, Entity as PollVote},
    prelude::*,
};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, QueryFilter, 
    QuerySelect, Set, TryIntoModel, PaginatorTrait, QueryOrder, DatabaseConnection
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct PollService {
    base: BaseService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PollOptionData {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreatePollRequest {
    pub title: String,
    pub description: Option<String>,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PollResult {
    pub poll: poll::Model,
    pub options: Vec<PollOptionWithVotes>,
    pub total_votes: i64,
    pub user_voted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PollOptionWithVotes {
    pub option: poll_option::Model,
    pub vote_count: i64,
    pub percentage: f64,
}

impl PollService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    /// Create a new poll with options
    pub async fn create(
        &self,
        db: &DbConn,
        request: CreatePollRequest,
        account_id: Uuid,
    ) -> Result<poll::Model, sea_orm::DbErr> {
        let poll_id = Uuid::new_v4();
        
        // Generate sequence ID by counting existing polls
        let seq_id = Poll::find().count(db).await? as i32 + 1;
        
        // Create the poll
        let poll = poll::ActiveModel {
            id: Set(poll_id),
            seq_id: Set(seq_id),
            title: Set(request.title),
            description: Set(request.description),
            account_id: Set(account_id),
            date_published: Set(Local::now().naive_local()),
            active: Set(true),
            ..Default::default()
        };

        let poll_model = poll.insert(db).await?;

        // Create poll options
        for (index, option_text) in request.options.iter().enumerate() {
            let option = poll_option::ActiveModel {
                id: Set(Uuid::new_v4()),
                poll_id: Set(poll_id),
                text: Set(option_text.clone()),
                position: Set(index as i32),
                ..Default::default()
            };
            option.insert(db).await?;
        }

        Ok(poll_model)
    }

    /// Find poll by seq_id
    pub async fn find_by_seq_id(&self, db: &DbConn, seq_id: i32) -> Result<poll::Model, sea_orm::DbErr> {
        Poll::find()
            .filter(poll::Column::SeqId.eq(seq_id))
            .one(db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Poll not found".to_string()))
    }

    /// Get poll with options and vote counts
    pub async fn get_poll_result(
        &self,
        db: &DbConn,
        seq_id: i32,
        ip_address: &str,
    ) -> Result<PollResult, sea_orm::DbErr> {
        let poll = self.find_by_seq_id(db, seq_id).await?;
        
        // Get poll options
        let options = PollOptionEntity::find()
            .filter(poll_option::Column::PollId.eq(poll.id))
            .order_by_asc(poll_option::Column::Position)
            .all(db)
            .await?;

        // Check if user has voted
        let user_voted = PollVote::find()
            .filter(poll_vote::Column::PollId.eq(poll.id))
            .filter(poll_vote::Column::IpAddress.eq(ip_address))
            .one(db)
            .await?
            .is_some();

        // Get vote counts for each option
        let vote_counts = self.get_vote_counts(db, poll.id).await?;
        let total_votes: i64 = vote_counts.values().sum();

        let options_with_votes: Vec<PollOptionWithVotes> = options
            .into_iter()
            .map(|option| {
                let vote_count = vote_counts.get(&option.id).copied().unwrap_or(0);
                let percentage = if total_votes > 0 {
                    (vote_count as f64 / total_votes as f64) * 100.0
                } else {
                    0.0
                };

                PollOptionWithVotes {
                    option,
                    vote_count,
                    percentage,
                }
            })
            .collect();

        Ok(PollResult {
            poll,
            options: options_with_votes,
            total_votes,
            user_voted,
        })
    }

    /// Cast a vote
    pub async fn vote(
        &self,
        db: &DbConn,
        poll_seq_id: i32,
        option_id: Uuid,
        ip_address: &str,
        session_id: Option<&str>,
    ) -> Result<(), sea_orm::DbErr> {
        let poll = self.find_by_seq_id(db, poll_seq_id).await?;

        // Check if poll is active
        if !poll.active {
            return Err(sea_orm::DbErr::Custom("Poll is not active".to_string()));
        }

        // Check if user has already voted
        let existing_vote = PollVote::find()
            .filter(poll_vote::Column::PollId.eq(poll.id))
            .filter(poll_vote::Column::IpAddress.eq(ip_address))
            .one(db)
            .await?;

        if existing_vote.is_some() {
            return Err(sea_orm::DbErr::Custom("You have already voted on this poll".to_string()));
        }

        // Verify the option belongs to this poll
        let option = PollOptionEntity::find()
            .filter(poll_option::Column::Id.eq(option_id))
            .filter(poll_option::Column::PollId.eq(poll.id))
            .one(db)
            .await?
            .ok_or(sea_orm::DbErr::Custom("Invalid option".to_string()))?;

        // Create the vote
        let vote = poll_vote::ActiveModel {
            id: Set(Uuid::new_v4()),
            poll_id: Set(poll.id),
            option_id: Set(option.id),
            ip_address: Set(ip_address.to_string()),
            session_id: Set(session_id.map(|s| s.to_string())),
            created_at: Set(Local::now().naive_local()),
            ..Default::default()
        };

        vote.insert(db).await?;
        Ok(())
    }

    /// List all polls with pagination
    pub async fn list_polls(
        &self,
        db: &DbConn,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<poll::Model>, u64), sea_orm::DbErr> {
        let paginator = Poll::find()
            .filter(poll::Column::Active.eq(true))
            .order_by_desc(poll::Column::DatePublished)
            .paginate(db, page_size);

        let total_pages = paginator.num_pages().await?;
        let polls = paginator.fetch_page(page - 1).await?;

        Ok((polls, total_pages))
    }

    /// Get vote counts for all options in a poll
    async fn get_vote_counts(
        &self,
        db: &DbConn,
        poll_id: Uuid,
    ) -> Result<HashMap<Uuid, i64>, sea_orm::DbErr> {
        let votes = PollVote::find()
            .filter(poll_vote::Column::PollId.eq(poll_id))
            .all(db)
            .await?;

        let mut counts = HashMap::new();
        for vote in votes {
            *counts.entry(vote.option_id).or_insert(0) += 1;
        }

        Ok(counts)
    }

    /// Toggle poll active status (admin only)
    pub async fn toggle_active(
        &self,
        db: &DbConn,
        seq_id: i32,
    ) -> Result<poll::Model, sea_orm::DbErr> {
        let poll = self.find_by_seq_id(db, seq_id).await?;
        
        let mut poll_active: poll::ActiveModel = poll.into();
        poll_active.active = Set(!poll_active.active.unwrap());
        
        poll_active.update(db).await
    }

    /// Delete poll (admin only)
    pub async fn delete(
        &self,
        db: &DbConn,
        seq_id: i32,
    ) -> Result<(), sea_orm::DbErr> {
        let poll = self.find_by_seq_id(db, seq_id).await?;
        
        // This will cascade delete options and votes due to FK constraints
        Poll::delete_by_id(poll.id).exec(db).await?;
        
        Ok(())
    }
}