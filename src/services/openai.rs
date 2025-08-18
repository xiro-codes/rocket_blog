use reqwest::Client;
use rocket::serde::{Deserialize, Serialize};
use sea_orm::DatabaseConnection;
use crate::services::SettingsService;

pub struct OpenAIService {
    client: Client,
    settings_service: SettingsService,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

impl OpenAIService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            settings_service: SettingsService::new(),
        }
    }

    pub async fn generate_post(&self, db: &DatabaseConnection, topic: &str) -> Result<GeneratedPost, String> {
        let api_key = self.settings_service.get_setting(db, "openai_api_key").await
            .map_err(|e| format!("Failed to get API key: {}", e))?
            .ok_or("OpenAI API key not configured")?;

        if api_key.trim().is_empty() {
            return Err("OpenAI API key is empty".to_string());
        }

        let model = self.settings_service.get_setting(db, "openai_model").await
            .unwrap_or_else(|_| Some("gpt-3.5-turbo".to_string()))
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        let max_tokens = self.settings_service.get_setting(db, "openai_max_tokens").await
            .unwrap_or_else(|_| Some("1000".to_string()))
            .unwrap_or_else(|| "1000".to_string())
            .parse::<u32>()
            .unwrap_or(1000);

        let temperature = self.settings_service.get_setting(db, "openai_temperature").await
            .unwrap_or_else(|_| Some("0.7".to_string()))
            .unwrap_or_else(|| "0.7".to_string())
            .parse::<f32>()
            .unwrap_or(0.7);

        let prompt = format!(
            "Write a comprehensive blog post about '{}'. The post should be informative, engaging, and well-structured. Include an introduction, main content with multiple sections, and a conclusion. Use markdown formatting. The post should be between 500-1000 words.",
            topic
        );

        let request = OpenAIRequest {
            model,
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a professional blog writer. Write high-quality, informative blog posts in markdown format.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            max_tokens,
            temperature,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to OpenAI: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        let content = openai_response.choices
            .first()
            .ok_or("No response from OpenAI")?
            .message
            .content
            .clone();

        // Extract title from the content (first line that looks like a header)
        let lines: Vec<&str> = content.lines().collect();
        let title = lines.iter()
            .find(|line| line.starts_with('#'))
            .map(|line| line.trim_start_matches('#').trim().to_string())
            .unwrap_or_else(|| format!("Generated Post: {}", topic));

        // Generate a simple excerpt from the first paragraph
        let excerpt = lines.iter()
            .find(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|line| {
                let words: Vec<&str> = line.split_whitespace().collect();
                if words.len() > 20 {
                    format!("{}...", words[..20].join(" "))
                } else {
                    line.to_string()
                }
            })
            .unwrap_or_else(|| "AI-generated blog post".to_string());

        Ok(GeneratedPost {
            title,
            content,
            excerpt,
        })
    }
}

#[derive(Debug)]
pub struct GeneratedPost {
    pub title: String,
    pub content: String,
    pub excerpt: String,
}