#[cfg(test)]
mod youtube_tests {
    use crate::services::YoutubeEmbedService;

    #[test]
    fn test_youtube_url_validation() {
        // Test valid YouTube URLs
        assert!(YoutubeEmbedService::is_valid_youtube_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(YoutubeEmbedService::is_valid_youtube_url("https://youtu.be/dQw4w9WgXcQ"));
        assert!(YoutubeEmbedService::is_valid_youtube_url("https://www.youtube.com/embed/dQw4w9WgXcQ"));
        assert!(YoutubeEmbedService::is_valid_youtube_url("https://www.youtube.com/v/dQw4w9WgXcQ"));
        
        // Test invalid URLs
        assert!(!YoutubeEmbedService::is_valid_youtube_url("https://www.example.com"));
        assert!(!YoutubeEmbedService::is_valid_youtube_url(""));
        assert!(!YoutubeEmbedService::is_valid_youtube_url("not_a_url"));
    }

    #[test]
    fn test_video_id_extraction() {
        // Test video ID extraction from different URL formats
        assert_eq!(
            YoutubeEmbedService::extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            YoutubeEmbedService::extract_video_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            YoutubeEmbedService::extract_video_id("https://www.youtube.com/embed/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            YoutubeEmbedService::extract_video_id("https://www.youtube.com/v/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        
        // Test URLs with additional parameters
        assert_eq!(
            YoutubeEmbedService::extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s"),
            Some("dQw4w9WgXcQ".to_string())
        );
        
        // Test invalid URLs
        assert_eq!(
            YoutubeEmbedService::extract_video_id("https://www.example.com"),
            None
        );
    }

    #[test]
    fn test_embed_html_generation() {
        let video_id = "dQw4w9WgXcQ";
        let embed_html = YoutubeEmbedService::generate_embed_html(video_id);
        
        // Check that the HTML contains expected elements
        assert!(embed_html.contains("iframe"));
        assert!(embed_html.contains("youtube.com/embed/dQw4w9WgXcQ"));
        assert!(embed_html.contains("allowfullscreen"));
        assert!(embed_html.contains("youtube-embed-container"));
    }

    #[test]
    fn test_embed_html_from_url() {
        let test_url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let embed_html = YoutubeEmbedService::generate_embed_html_from_url(test_url);
        
        assert!(embed_html.is_some());
        let html = embed_html.unwrap();
        assert!(html.contains("youtube.com/embed/dQw4w9WgXcQ"));
        
        // Test with invalid URL
        let invalid_url = "https://www.example.com";
        assert!(YoutubeEmbedService::generate_embed_html_from_url(invalid_url).is_none());
    }
}