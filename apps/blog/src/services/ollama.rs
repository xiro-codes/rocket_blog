use crate::services::{ai_provider::AIProvider, BaseService, SettingsService};
use async_trait::async_trait;
use reqwest::Client;
use sea_orm::DatabaseConnection;
use rocket::serde::{Deserialize, Serialize};

pub struct OllamaService {
    base: BaseService,
    settings_service: SettingsService,
    http_client: Client,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct OllamaOptions {
    num_predict: i32,
    temperature: f32,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct OllamaResponse {
    response: String,
    done: bool,
}

impl OllamaService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
            settings_service: SettingsService::new(),
            http_client: Client::new(),
        }
    }

    /// Get the configured Ollama URL from settings
    async fn get_ollama_url(&self, db: &DatabaseConnection) -> Result<String, String> {
        self.settings_service
            .get_ollama_url(db)
            .await
            .map_err(|e| format!("Failed to get Ollama URL from database: {}", e))?
            .ok_or_else(|| "Ollama URL not configured".to_string())
    }

    /// Get the configured Ollama model from settings
    async fn get_ollama_model(&self, db: &DatabaseConnection) -> Result<String, String> {
        Ok(self.settings_service
            .get_ollama_model(db)
            .await
            .map_err(|e| format!("Failed to get Ollama model from database: {}", e))?
            .unwrap_or_else(|| "llama2".to_string())) // Default model
    }

    /// Test Ollama connection
    async fn test_connection(&self, db: &DatabaseConnection) -> Result<bool, String> {
        let base_url = self.get_ollama_url(db).await?;
        let url = format!("{}/api/tags", base_url);
        
        match self.http_client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Generate content using Ollama API
    async fn generate_content(&self, db: &DatabaseConnection, prompt: &str, max_tokens: i32, temperature: f32) -> Result<String, String> {
        debug!("Ollama API request: max_tokens={}, temperature={}, prompt_length={}", 
               max_tokens, temperature, prompt.len());
        
        let base_url = self.get_ollama_url(db).await?;
        let model = self.get_ollama_model(db).await?;
        let url = format!("{}/api/generate", base_url);

        let request = OllamaRequest {
            model,
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                num_predict: max_tokens,
                temperature,
            },
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Ollama API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama API error: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(ollama_response.response.trim().to_string())
    }
}

#[async_trait]
impl AIProvider for OllamaService {
    async fn is_available(&self, db: &DatabaseConnection) -> bool {
        // Check if Ollama is enabled and can connect
        let enabled = self.settings_service
            .get_ollama_enabled(db)
            .await
            .unwrap_or(false);
        
        if !enabled {
            return false;
        }

        self.test_connection(db).await.unwrap_or(false)
    }

    async fn generate_post_content(
        &self,
        db: &DatabaseConnection,
        title: &str,
        additional_prompt: Option<&str>,
    ) -> Result<String, String> {
        let prompt = if let Some(additional) = additional_prompt {
            format!(
                "Write a detailed and engaging blog post titled '{}'. Additional context: {}. Please write in markdown format with proper headers, paragraphs, and structure.",
                title, additional
            )
        } else {
            format!(
                "Write a detailed and engaging blog post titled '{}'. Please write in markdown format with proper headers, paragraphs, and structure.",
                title
            )
        };

        self.generate_content(db, &prompt, 1500, 0.7).await
    }

    async fn generate_excerpt(&self, db: &DatabaseConnection, content: &str) -> Result<String, String> {
        let prompt = format!(
            "Create a brief excerpt (1-2 sentences) that summarizes the main idea of this blog post and entices readers:\n\n{}",
            content
        );

        self.generate_content(db, &prompt, 150, 0.5).await
    }

    async fn generate_tags(&self, db: &DatabaseConnection, title: &str, content: &str) -> Result<Vec<String>, String> {
        let prompt = format!(
            "Suggest 3-5 relevant single-word or short-phrase tags for this blog post. Return only the tags separated by commas, no extra text:\n\nTitle: {}\n\nContent: {}",
            title, content
        );

        let response = self.generate_content(db, &prompt, 100, 0.3).await?;
        
        // Parse comma-separated tags
        let tags: Vec<String> = response
            .split(',')
            .map(|tag| tag.trim().to_lowercase())
            .filter(|tag| !tag.is_empty())
            .collect();

        Ok(tags)
    }

    fn provider_name(&self) -> &'static str {
        "Ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_service_new() {
        let service = OllamaService::new();
        assert_eq!(service.provider_name(), "Ollama");
    }
}