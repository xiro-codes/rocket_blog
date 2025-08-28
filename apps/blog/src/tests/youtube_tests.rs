use std::process::Command;

#[cfg(test)]
mod youtube_tests {
    use super::*;
    use crate::services::YoutubeDownloadService;

    #[test]
    fn test_youtube_url_validation() {
        // Test valid YouTube URLs
        assert!(YoutubeDownloadService::is_valid_youtube_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(YoutubeDownloadService::is_valid_youtube_url("https://youtu.be/dQw4w9WgXcQ"));
        assert!(YoutubeDownloadService::is_valid_youtube_url("https://www.youtube.com/embed/dQw4w9WgXcQ"));
        assert!(YoutubeDownloadService::is_valid_youtube_url("https://www.youtube.com/v/dQw4w9WgXcQ"));
        
        // Test invalid URLs
        assert!(!YoutubeDownloadService::is_valid_youtube_url("https://www.example.com"));
        assert!(!YoutubeDownloadService::is_valid_youtube_url(""));
        assert!(!YoutubeDownloadService::is_valid_youtube_url("not_a_url"));
    }

    #[test]
    fn test_video_id_extraction() {
        // Test video ID extraction from different URL formats
        assert_eq!(
            YoutubeDownloadService::extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            YoutubeDownloadService::extract_video_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            YoutubeDownloadService::extract_video_id("https://www.youtube.com/embed/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            YoutubeDownloadService::extract_video_id("https://www.youtube.com/v/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        
        // Test invalid URLs
        assert_eq!(
            YoutubeDownloadService::extract_video_id("https://www.example.com"),
            None
        );
    }

    #[test]
    fn test_yt_dlp_availability() {
        // Check if yt-dlp is installed
        let result = Command::new("yt-dlp")
            .arg("--version")
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("yt-dlp is available, version: {}", version.trim());
                } else {
                    println!("yt-dlp command failed");
                }
            }
            Err(_) => {
                println!("yt-dlp is not installed or not in PATH");
            }
        }
    }
}