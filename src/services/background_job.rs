use chrono::Utc;
use models::{background_job, prelude::*};
use rocket::serde::{Deserialize, Serialize};
use sea_orm::*;
use serde_json;
use uuid::Uuid;

use crate::services::BaseService;
use models::background_job::{JobStatus, JobType};

type DbConn = DatabaseConnection;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AIJobPayload {
    pub title: String,
    pub prompt: Option<String>,
    pub content: Option<String>,
    pub provider: Option<String>,
}

pub struct BackgroundJobService {
    base: BaseService,
}

impl BackgroundJobService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    /// Create a new background job
    pub async fn create_job(
        &self,
        db: &DbConn,
        job_type: JobType,
        payload: AIJobPayload,
        account_id: Uuid,
    ) -> Result<background_job::Model, DbErr> {
        let now = Utc::now().naive_utc();
        let job_id = Uuid::new_v4();

        let job = background_job::ActiveModel {
            id: Set(job_id),
            job_type: Set(job_type.to_string()),
            status: Set(JobStatus::Pending.to_string()),
            payload: Set(Some(serde_json::to_value(payload).unwrap())),
            result: NotSet,
            error: NotSet,
            created_at: Set(now),
            updated_at: Set(now),
            completed_at: NotSet,
            account_id: Set(account_id),
        };

        job.insert(db).await
    }

    /// Get a job by ID
    pub async fn get_job_by_id(
        &self,
        db: &DbConn,
        job_id: Uuid,
    ) -> Result<Option<background_job::Model>, DbErr> {
        BackgroundJob::find_by_id(job_id).one(db).await
    }

    /// Update job status
    pub async fn update_job_status(
        &self,
        db: &DbConn,
        job_id: Uuid,
        status: JobStatus,
        result: Option<serde_json::Value>,
        error: Option<String>,
    ) -> Result<background_job::Model, DbErr> {
        let now = Utc::now().naive_utc();
        
        let mut job: background_job::ActiveModel = BackgroundJob::find_by_id(job_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Job not found".to_string()))?
            .into();

        job.status = Set(status.to_string());
        job.updated_at = Set(now);
        
        if let Some(result_data) = result {
            job.result = Set(Some(result_data));
        }
        
        if let Some(error_msg) = error {
            job.error = Set(Some(error_msg));
        }
        
        if status == JobStatus::Completed || status == JobStatus::Failed {
            job.completed_at = Set(Some(now));
        }

        job.update(db).await
    }

    /// Get pending jobs for processing
    pub async fn get_pending_jobs(
        &self,
        db: &DbConn,
        limit: u64,
    ) -> Result<Vec<background_job::Model>, DbErr> {
        BackgroundJob::find()
            .filter(background_job::Column::Status.eq(JobStatus::Pending.to_string()))
            .order_by_asc(background_job::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await
    }

    /// Mark job as running
    pub async fn mark_job_running(
        &self,
        db: &DbConn,
        job_id: Uuid,
    ) -> Result<background_job::Model, DbErr> {
        self.update_job_status(db, job_id, JobStatus::Running, None, None)
            .await
    }

    /// Complete job with success
    pub async fn complete_job(
        &self,
        db: &DbConn,
        job_id: Uuid,
        result: serde_json::Value,
    ) -> Result<background_job::Model, DbErr> {
        self.update_job_status(db, job_id, JobStatus::Completed, Some(result), None)
            .await
    }

    /// Fail job with error
    pub async fn fail_job(
        &self,
        db: &DbConn,
        job_id: Uuid,
        error: String,
    ) -> Result<background_job::Model, DbErr> {
        self.update_job_status(db, job_id, JobStatus::Failed, None, Some(error))
            .await
    }

    /// Get jobs for a specific account
    pub async fn get_jobs_for_account(
        &self,
        db: &DbConn,
        account_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<background_job::Model>, DbErr> {
        BackgroundJob::find()
            .filter(background_job::Column::AccountId.eq(account_id))
            .order_by_desc(background_job::Column::CreatedAt)
            .paginate(db, per_page)
            .fetch_page(page)
            .await
    }

    /// Clean up old completed jobs (optional maintenance)
    pub async fn cleanup_old_jobs(
        &self,
        db: &DbConn,
        days_old: i64,
    ) -> Result<DeleteResult, DbErr> {
        let cutoff_date = Utc::now().naive_utc() - chrono::Duration::days(days_old);
        
        BackgroundJob::delete_many()
            .filter(
                background_job::Column::Status
                    .is_in([JobStatus::Completed.to_string(), JobStatus::Failed.to_string()])
                    .and(background_job::Column::CompletedAt.lt(cutoff_date))
            )
            .exec(db)
            .await
    }
}