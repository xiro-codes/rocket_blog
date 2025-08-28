use std::process::Command;
use std::path::Path;
use uuid::Uuid;
use common::config::AppConfig;
use rocket::State;
use sea_orm::*;
use models::{background_job, prelude::BackgroundJob};
use crate::services::BackgroundJobService;
use serde_json;

pub struct YoutubeDownloadService;

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
}

impl ToString for DownloadStatus {
    fn to_string(&self) -> String {
        match self {
            DownloadStatus::Pending => background_job::STATUS_PENDING.to_string(),
            DownloadStatus::Downloading => background_job::STATUS_DOWNLOADING.to_string(),
            DownloadStatus::Completed => background_job::STATUS_COMPLETED.to_string(),
            DownloadStatus::Failed => background_job::STATUS_FAILED.to_string(),
        }
    }
}

#[derive(Debug, rocket::serde::Serialize, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct YoutubeJobData {
    pub youtube_url: String,
    pub data_path: String,
}

impl YoutubeDownloadService {
    pub fn new() -> Self {
        Self
    }

    /// Validates if the URL is a valid YouTube URL
    pub fn is_valid_youtube_url(url: &str) -> bool {
        url.contains("youtube.com/watch?v=") 
            || url.contains("youtu.be/")
            || url.contains("youtube.com/embed/")
            || url.contains("youtube.com/v/")
    }

    /// Extract video ID from YouTube URL
    pub fn extract_video_id(url: &str) -> Option<String> {
        if url.contains("youtube.com/watch?v=") {
            url.split("watch?v=").nth(1)?.split('&').next().map(|s| s.to_string())
        } else if url.contains("youtu.be/") {
            url.split("youtu.be/").nth(1)?.split('?').next().map(|s| s.to_string())
        } else if url.contains("youtube.com/embed/") {
            url.split("embed/").nth(1)?.split('?').next().map(|s| s.to_string())
        } else if url.contains("youtube.com/v/") {
            url.split("v/").nth(1)?.split('?').next().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Start downloading a YouTube video in the background
    pub async fn start_download(
        &self,
        db: &DbConn,
        app_config: &State<AppConfig>,
        post_id: Uuid,
        youtube_url: String,
    ) -> Result<(), DbErr> {
        log::info!("Starting YouTube download for post {}: {}", post_id, youtube_url);

        let job_service = BackgroundJobService::new();
        
        // Create job data
        let job_data = YoutubeJobData {
            youtube_url: youtube_url.clone(),
            data_path: app_config.get_data_path(),
        };

        // Create background job
        let job = job_service.create_job(
            db,
            background_job::JOB_TYPE_YOUTUBE_DOWNLOAD.to_string(),
            background_job::ENTITY_TYPE_POST.to_string(),
            post_id,
            DownloadStatus::Pending.to_string(),
            Some(serde_json::to_value(&job_data).map_err(|e| DbErr::Custom(e.to_string()))?),
        ).await?;

        log::info!("Created background job {} for YouTube download", job.id);

        // Update job status to downloading and start processing
        job_service.update_job_status(
            db,
            job.id,
            DownloadStatus::Downloading.to_string(),
            None,
        ).await?;

        // Start immediate download (synchronous for now, in a real implementation you'd use a job queue)
        match Self::download_video_sync(&job_data, post_id).await {
            Ok(file_path) => {
                log::info!("YouTube download completed for post {}: {}", post_id, file_path);
                // Update job with completed status
                job_service.update_job_status(
                    db,
                    job.id,
                    DownloadStatus::Completed.to_string(),
                    None,
                ).await?;
                
                // Update post with file path
                self.update_post_with_file_path(db, post_id, file_path).await?;
            },
            Err(e) => {
                log::error!("YouTube download failed for post {}: {}", post_id, e);
                job_service.update_job_status(
                    db,
                    job.id,
                    DownloadStatus::Failed.to_string(),
                    Some(e),
                ).await?;
            }
        }

        Ok(())
    }

    /// Update post with the downloaded file path
    async fn update_post_with_file_path(
        &self,
        db: &DbConn,
        post_id: Uuid,
        file_path: String,
    ) -> Result<(), DbErr> {
        use models::{post, prelude::Post};
        
        let mut post: post::ActiveModel = Post::find()
            .filter(post::Column::Id.eq(post_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", post_id)))?
            .into();

        post.path = Set(Some(file_path));
        post.update(db).await?;
        Ok(())
    }

    /// Download video using yt-dlp synchronously
    async fn download_video_sync(job_data: &YoutubeJobData, post_id: Uuid) -> Result<String, String> {
        // Generate output filename
        let video_id = Self::extract_video_id(&job_data.youtube_url)
            .ok_or("Failed to extract video ID from URL")?;
        let output_filename = format!("{}_{}.%(ext)s", post_id, video_id);
        let output_path = format!("{}/{}", job_data.data_path, output_filename);

        log::debug!("Downloading video to: {}", output_path);

        // Check if yt-dlp is available
        let yt_dlp_check = Command::new("yt-dlp")
            .arg("--version")
            .output();

        match yt_dlp_check {
            Ok(_) => {
                log::debug!("yt-dlp found, proceeding with download");
            },
            Err(_) => {
                return Err("yt-dlp not found. Please install yt-dlp to enable YouTube downloads".to_string());
            }
        }
        // Run yt-dlp to download the video
        let output = Command::new("yt-dlp")
            .arg("--format")
            .arg("best[ext=mp4]/best") // Prefer mp4 format, fallback to best available
            .arg("--output")
            .arg(&output_path)
            .arg("--no-playlist") // Download only the specific video, not the entire playlist
            // .arg("--extract-flat")
            // .arg("false")
            .arg(&job_data.youtube_url)
            .output()
            .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

        if output.status.success() {
            // Find the actual downloaded file (since extension might vary)
            let path_without_ext = output_path.replace(".%(ext)s", "");
            
            // Common video extensions yt-dlp might use
            let extensions = &["mp4", "webm", "mkv", "avi", "flv"];
            
            for ext in extensions {
                let potential_path = format!("{}.{}", path_without_ext, ext);
                if Path::new(&potential_path).exists() {
                    log::info!("Downloaded video found at: {}", potential_path);
                    return Ok(potential_path);
                }
            }
            
            Err("Download completed but could not find the downloaded file".to_string())
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr);
            Err(format!("yt-dlp failed: {}", error_message))
        }
    }

    /// Get download status for a post
    pub async fn get_download_status(
        &self,
        db: &DbConn,
        post_id: Uuid,
    ) -> Result<Option<(String, Option<String>)>, DbErr> {
        let job_service = BackgroundJobService::new();
        
        match job_service.get_job_by_entity(
            db,
            background_job::ENTITY_TYPE_POST.to_string(),
            post_id,
            background_job::JOB_TYPE_YOUTUBE_DOWNLOAD.to_string(),
        ).await? {
            Some(job) => Ok(Some((job.status, job.error_message))),
            None => Ok(None),
        }
    }
}
