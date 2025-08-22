use async_openai::{Client, types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, 
    ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs, Role
}};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use crate::services::{ai_provider::AIProvider, base::BaseService, SettingsService};

pub struct OpenAIService {
    settings_service: SettingsService,
}

impl OpenAIService {
    pub fn new() -> Self {
        Self {
            settings_service: SettingsService::new(),
        }
    }

    /// Get a configured OpenAI client if API key is available
    async fn get_client(&self, db: &DatabaseConnection) -> Result<Client<async_openai::config::OpenAIConfig>, String> {
        let api_key = self.settings_service
            .get_openai_api_key(db)
            .await
            .map_err(|e| format!("Failed to get API key from database: {}", e))?
            .ok_or_else(|| "OpenAI API key not configured".to_string())?;

        Ok(Client::with_config(
            async_openai::config::OpenAIConfig::new().with_api_key(api_key)
        ))
    }

    /// Generate blog post content from a title/prompt
    pub async fn generate_post_content(&self, db: &DatabaseConnection, title: &str, additional_prompt: Option<&str>) -> Result<String, String> {
        let client = self.get_client(db).await?;

        let system_message = "You are a helpful assistant that writes engaging blog posts. Create well-structured, informative content with proper paragraphs. Write in markdown format with headers, lists, and emphasis where appropriate.";
        
        let user_prompt = if let Some(prompt) = additional_prompt {
            format!("Write a blog post titled '{}'. Additional context: {}", title, prompt)
        } else {
            format!("Write a detailed and engaging blog post titled '{}'", title)
        };

        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: system_message.to_string(),
                    role: Role::System,
                    name: None,
                }
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(user_prompt),
                    role: Role::User,
                    name: None,
                }
            ),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(messages)
            .max_tokens(1500u16)
            .temperature(0.7)
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI API error: {}", e))?;

        response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .map(|content| content.clone())
            .ok_or_else(|| "No content in OpenAI response".to_string())
    }

    /// Generate an excerpt from content
    pub async fn generate_excerpt(&self, db: &DatabaseConnection, content: &str) -> Result<String, String> {
        let client = self.get_client(db).await?;

        let system_message = "You are a helpful assistant that creates concise, engaging excerpts from blog posts. Create a 1-2 sentence summary that captures the main idea and entices readers.";
        
        let user_prompt = format!("Create a brief excerpt (1-2 sentences) for this blog post content:\n\n{}", content);

        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: system_message.to_string(),
                    role: Role::System,
                    name: None,
                }
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(user_prompt),
                    role: Role::User,
                    name: None,
                }
            ),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(messages)
            .max_tokens(150u16)
            .temperature(0.5)
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI API error: {}", e))?;

        response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .map(|content| content.trim().to_string())
            .ok_or_else(|| "No content in OpenAI response".to_string())
    }

    /// Generate suggested tags from content
    pub async fn generate_tags(&self, db: &DatabaseConnection, title: &str, content: &str) -> Result<Vec<String>, String> {
        let client = self.get_client(db).await?;

        let system_message = "You are a helpful assistant that suggests relevant tags for blog posts. Return 3-5 single-word or short phrase tags separated by commas.";
        
        let user_prompt = format!("Suggest relevant tags for this blog post:\n\nTitle: {}\n\nContent: {}", title, content);

        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: system_message.to_string(),
                    role: Role::System,
                    name: None,
                }
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(user_prompt),
                    role: Role::User,
                    name: None,
                }
            ),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(messages)
            .max_tokens(100u16)
            .temperature(0.3)
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI API error: {}", e))?;

        let tags_text = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| "No content in OpenAI response".to_string())?;

        // Parse comma-separated tags
        let tags: Vec<String> = tags_text
            .split(',')
            .map(|tag| tag.trim().to_lowercase())
            .filter(|tag| !tag.is_empty())
            .collect();

        Ok(tags)
    }
}

#[async_trait]
impl AIProvider for OpenAIService {
    async fn is_available(&self, db: &DatabaseConnection) -> bool {
        self.settings_service
            .get_openai_api_key(db)
            .await
            .unwrap_or(None)
            .is_some()
    }

    async fn generate_post_content(
        &self,
        db: &DatabaseConnection,
        title: &str,
        additional_prompt: Option<&str>,
    ) -> Result<String, String> {
        self.generate_post_content(db, title, additional_prompt).await
    }

    async fn generate_excerpt(&self, db: &DatabaseConnection, content: &str) -> Result<String, String> {
        self.generate_excerpt(db, content).await
    }

    async fn generate_tags(
        &self,
        db: &DatabaseConnection,
        title: &str,
        content: &str,
    ) -> Result<Vec<String>, String> {
        self.generate_tags(db, title, content).await
    }

    fn provider_name(&self) -> &'static str {
        "OpenAI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_service_new() {
        let service = OpenAIService::new();
        // Service should be created successfully
        assert!(std::ptr::addr_of!(service).is_null() == false);
    }
}