use sea_orm::*;
use uuid::Uuid;
use chrono::{Utc, FixedOffset};
use models::{background_job, prelude::BackgroundJob};
use serde_json;

pub struct BackgroundJobService;

impl BackgroundJobService {
    pub fn new() -> Self {
        Self
    }

    /// Create a new background job
    pub async fn create_job(
        &self,
        db: &DbConn,
        job_type: String,
        entity_type: String,
        entity_id: Uuid,
        status: String,
        job_data: Option<serde_json::Value>,
    ) -> Result<background_job::Model, DbErr> {
        let job_id = Uuid::new_v4();
        let now = Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap());

        let job = background_job::ActiveModel {
            id: Set(job_id),
            job_type: Set(job_type),
            entity_type: Set(entity_type),
            entity_id: Set(entity_id),
            status: Set(status),
            error_message: Set(None),
            job_data: Set(job_data),
            created_at: Set(now),
            updated_at: Set(now),
        };

        job.insert(db).await
    }

    /// Update job status
    pub async fn update_job_status(
        &self,
        db: &DbConn,
        job_id: Uuid,
        status: String,
        error_message: Option<String>,
    ) -> Result<background_job::Model, DbErr> {
        let job = BackgroundJob::find_by_id(job_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Job with id: {}", job_id)))?;

        let mut job: background_job::ActiveModel = job.into();
        job.status = Set(status);
        job.error_message = Set(error_message);
        job.updated_at = Set(Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()));

        job.update(db).await
    }

    /// Get job by entity
    pub async fn get_job_by_entity(
        &self,
        db: &DbConn,
        entity_type: String,
        entity_id: Uuid,
        job_type: String,
    ) -> Result<Option<background_job::Model>, DbErr> {
        BackgroundJob::find()
            .filter(background_job::Column::EntityType.eq(entity_type))
            .filter(background_job::Column::EntityId.eq(entity_id))
            .filter(background_job::Column::JobType.eq(job_type))
            .order_by_desc(background_job::Column::CreatedAt)
            .one(db)
            .await
    }

    /// Get all pending jobs of a specific type
    pub async fn get_pending_jobs(
        &self,
        db: &DbConn,
        job_type: String,
    ) -> Result<Vec<background_job::Model>, DbErr> {
        BackgroundJob::find()
            .filter(background_job::Column::JobType.eq(job_type))
            .filter(background_job::Column::Status.eq(background_job::STATUS_PENDING))
            .order_by_asc(background_job::Column::CreatedAt)
            .all(db)
            .await
    }

    /// Delete completed jobs older than a certain age
    pub async fn cleanup_old_jobs(
        &self,
        db: &DbConn,
        days_old: i64,
    ) -> Result<DeleteResult, DbErr> {
        let cutoff_date = Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()) - chrono::Duration::days(days_old);
        
        BackgroundJob::delete_many()
            .filter(background_job::Column::Status.eq(background_job::STATUS_COMPLETED))
            .filter(background_job::Column::UpdatedAt.lt(cutoff_date))
            .exec(db)
            .await
    }
}