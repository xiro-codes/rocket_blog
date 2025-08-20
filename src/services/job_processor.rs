use std::time::Duration;
use rocket::tokio::time::sleep;
use uuid::Uuid;
use serde_json::json;

use crate::services::{AIProviderService, BackgroundJobService, BlogService, AIJobPayload};
use models::background_job::{JobType, JobStatus};
use models::prelude::*;
use models::post;
use sea_orm::*;
use chrono::Utc;

pub struct JobProcessor;

impl JobProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Start the background job processor as a Rocket fairing
    pub fn start_processor_task(db: DatabaseConnection) {
        log::info!("Starting background job processor task");
        
        rocket::tokio::spawn(async move {
            let background_job_service = BackgroundJobService::new();
            
            loop {
                // Process pending jobs
                match background_job_service.get_pending_jobs(&db, 10).await {
                    Ok(jobs) => {
                        for job in jobs {
                            if let Err(e) = Self::process_job(&background_job_service, &db, job).await {
                                log::error!("Error processing job: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error fetching pending jobs: {}", e);
                    }
                }

                // Sleep between job processing cycles
                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    /// Process a single job
    async fn process_job(
        background_job_service: &BackgroundJobService,
        db: &DatabaseConnection,
        job: models::background_job::Model,
    ) -> Result<(), String> {
        log::info!("Processing job {} of type {}", job.id, job.job_type);

        // Mark job as running
        if let Err(e) = background_job_service.mark_job_running(db, job.id).await {
            return Err(format!("Failed to mark job as running: {}", e));
        }

        // Parse job type
        let job_type = match job.job_type.parse::<JobType>() {
            Ok(jt) => jt,
            Err(e) => {
                let _ = background_job_service.fail_job(db, job.id, e.clone()).await;
                return Err(format!("Invalid job type: {}", e));
            }
        };

        // Parse payload
        let payload: AIJobPayload = match job.payload {
            Some(p) => match serde_json::from_value(p) {
                Ok(payload) => payload,
                Err(e) => {
                    let _ = background_job_service.fail_job(db, job.id, format!("Invalid payload: {}", e)).await;
                    return Err(format!("Invalid payload: {}", e));
                }
            },
            None => {
                let _ = background_job_service.fail_job(db, job.id, "Missing payload".to_string()).await;
                return Err("Missing payload".to_string());
            }
        };

        // Create AI provider service to get providers
        let mut ai_service = AIProviderService::new();
        ai_service.add_provider(Box::new(crate::services::OpenAIService::new()));
        ai_service.add_provider(Box::new(crate::services::OllamaService::new()));

        // Get AI provider
        let provider = match ai_service.get_available_provider(db).await {
            Some(p) => p,
            None => {
                let _ = background_job_service.fail_job(db, job.id, "No AI provider available".to_string()).await;
                return Err("No AI provider available".to_string());
            }
        };

        // Process based on job type
        let result = match job_type {
            JobType::GenerateContent => {
                match provider.generate_post_content(db, &payload.title, payload.prompt.as_deref()).await {
                    Ok(content) => {
                        // Create draft post automatically
                        match Self::create_draft_post(db, &payload.title, &content, job.account_id).await {
                            Ok((post_id, seq_id)) => {
                                json!({
                                    "content": content,
                                    "provider": provider.provider_name(),
                                    "draft_post_id": post_id,
                                    "draft_post_seq_id": seq_id,
                                    "draft_post_title": format!("[AI DRAFT] {}", payload.title)
                                })
                            }
                            Err(e) => {
                                log::warn!("Generated content but failed to create draft post: {}", e);
                                json!({
                                    "content": content,
                                    "provider": provider.provider_name(),
                                    "warning": "Content generated but draft post creation failed"
                                })
                            }
                        }
                    }
                    Err(e) => {
                        let _ = background_job_service.fail_job(db, job.id, e.clone()).await;
                        return Err(e);
                    }
                }
            }
            JobType::GenerateExcerpt => {
                let content = payload.content.unwrap_or_default();
                match provider.generate_excerpt(db, &content).await {
                    Ok(excerpt) => json!({
                        "excerpt": excerpt,
                        "provider": provider.provider_name()
                    }),
                    Err(e) => {
                        let _ = background_job_service.fail_job(db, job.id, e.clone()).await;
                        return Err(e);
                    }
                }
            }
            JobType::GenerateTags => {
                let content = payload.content.unwrap_or_default();
                match provider.generate_tags(db, &payload.title, &content).await {
                    Ok(tags) => json!({
                        "tags": tags,
                        "provider": provider.provider_name()
                    }),
                    Err(e) => {
                        let _ = background_job_service.fail_job(db, job.id, e.clone()).await;
                        return Err(e);
                    }
                }
            }
        };

        // Mark job as completed
        if let Err(e) = background_job_service.complete_job(db, job.id, result).await {
            return Err(format!("Failed to mark job as completed: {}", e));
        }

        log::info!("Successfully processed job {}", job.id);
        Ok(())
    }

    /// Create a draft post with the generated content
    async fn create_draft_post(
        db: &DatabaseConnection,
        title: &str,
        content: &str,
        account_id: Uuid,
    ) -> Result<(Uuid, i32), String> {
        let now = Utc::now().naive_utc();
        let post_id = Uuid::new_v4();

        // Get the next sequence ID (this is simplified - in production you'd want proper sequence handling)
        let max_seq_id = Post::find()
            .select_only()
            .column_as(post::Column::SeqId.max(), "max_seq_id")
            .into_tuple::<Option<i32>>()
            .one(db)
            .await
            .map_err(|e| format!("Failed to get max seq_id: {}", e))?
            .unwrap_or(Some(0))
            .unwrap_or(0);

        let seq_id = max_seq_id + 1;

        let new_post = post::ActiveModel {
            id: Set(post_id),
            seq_id: Set(seq_id),
            title: Set(format!("[AI DRAFT] {}", title)),
            text: Set(content.to_string()),
            excerpt: NotSet,
            path: NotSet,
            draft: Set(Some(true)),
            date_published: Set(now),
            account_id: Set(account_id),
        };

        new_post.insert(db)
            .await
            .map_err(|e| format!("Failed to create draft post: {}", e))?;

        Ok((post_id, seq_id))
    }
}