pub struct YoutubeEmbedService;

impl YoutubeEmbedService {
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

    /// Generate YouTube embed HTML from video ID
    pub fn generate_embed_html(video_id: &str) -> String {
        format!(
            r#"<div class="youtube-embed-container" style="position: relative; width: 100%; height: 0; padding-bottom: 56.25%;">
                <iframe src="https://www.youtube.com/embed/{}" 
                        style="position: absolute; top: 0; left: 0; width: 100%; height: 100%;" 
                        frameborder="0" 
                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" 
                        allowfullscreen>
                </iframe>
            </div>"#,
            video_id
        )
    }

    /// Generate YouTube embed HTML from URL
    pub fn generate_embed_html_from_url(url: &str) -> Option<String> {
        Self::extract_video_id(url).map(|video_id| Self::generate_embed_html(&video_id))
    }
}
