use std::process::Command;
use std::path::Path;
use uuid::Uuid;
use crate::config::AppConfig;
use rocket::State;
use sea_orm::*;
use models::{post, prelude::Post};

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
            DownloadStatus::Pending => "pending".to_string(),
            DownloadStatus::Downloading => "downloading".to_string(),
            DownloadStatus::Completed => "completed".to_string(),
            DownloadStatus::Failed => "failed".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct DownloadJob {
    pub post_id: Uuid,
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

        // Update post status to downloading
        self.update_download_status(db, post_id, DownloadStatus::Downloading, None).await?;

        // Create download job
        let job = DownloadJob {
            post_id,
            youtube_url: youtube_url.clone(),
            data_path: app_config.data_path.clone(),
        };

        // Start immediate download (synchronous for now, in a real implementation you'd use a job queue)
        match Self::download_video_sync(&job).await {
            Ok(file_path) => {
                log::info!("YouTube download completed for post {}: {}", post_id, file_path);
                // Update post with completed status and file path
                self.update_download_completed(db, post_id, file_path).await?;
            },
            Err(e) => {
                log::error!("YouTube download failed for post {}: {}", post_id, e);
                self.update_download_status(db, post_id, DownloadStatus::Failed, Some(e)).await?;
            }
        }

        Ok(())
    }

    /// Update the download status in the database
    pub async fn update_download_status(
        &self,
        db: &DbConn,
        post_id: Uuid,
        status: DownloadStatus,
        error_message: Option<String>,
    ) -> Result<(), DbErr> {
        let mut post: post::ActiveModel = Post::find()
            .filter(post::Column::Id.eq(post_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", post_id)))?
            .into();

        post.download_status = Set(Some(status.to_string()));
        if let Some(error) = error_message {
            post.download_error = Set(Some(error));
        }

        post.update(db).await?;
        Ok(())
    }

    /// Update post with completed download and file path
    pub async fn update_download_completed(
        &self,
        db: &DbConn,
        post_id: Uuid,
        file_path: String,
    ) -> Result<(), DbErr> {
        let mut post: post::ActiveModel = Post::find()
            .filter(post::Column::Id.eq(post_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", post_id)))?
            .into();

        post.download_status = Set(Some(DownloadStatus::Completed.to_string()));
        post.download_error = Set(None);
        post.path = Set(Some(file_path));

        post.update(db).await?;
        Ok(())
    }

    /// Download video using yt-dlp synchronously
    async fn download_video_sync(job: &DownloadJob) -> Result<String, String> {
        // Generate output filename
        let video_id = Self::extract_video_id(&job.youtube_url)
            .ok_or("Failed to extract video ID from URL")?;
        let output_filename = format!("{}_{}.%(ext)s", job.post_id, video_id);
        let output_path = format!("{}/{}", job.data_path, output_filename);

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
            .arg("--extract-flat")
            .arg("false")
            .arg(&job.youtube_url)
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
        let post = Post::find()
            .filter(post::Column::Id.eq(post_id))
            .one(db)
            .await?;

        match post {
            Some(p) => Ok(p.download_status.map(|status| (status, p.download_error))),
            None => Ok(None),
        }
    }
}