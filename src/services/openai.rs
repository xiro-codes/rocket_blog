use async_openai::{Client, types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, 
    ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs, Role
}};
use crate::services::base::BaseService;

pub struct OpenAIService {
    base: BaseService,
    client: Option<Client<async_openai::config::OpenAIConfig>>,
}

impl OpenAIService {
    pub fn new(api_key: Option<String>) -> Self {
        let client = if let Some(key) = api_key {
            Some(Client::with_config(
                async_openai::config::OpenAIConfig::new().with_api_key(key)
            ))
        } else {
            None
        };
        
        Self {
            base: BaseService::new(),
            client,
        }
    }

    /// Check if OpenAI service is available (has API key)
    pub fn is_available(&self) -> bool {
        self.client.is_some()
    }

    /// Generate blog post content from a title/prompt
    pub async fn generate_post_content(&self, title: &str, additional_prompt: Option<&str>) -> Result<String, String> {
        let client = self.client.as_ref()
            .ok_or_else(|| "OpenAI API key not configured".to_string())?;

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
    pub async fn generate_excerpt(&self, content: &str) -> Result<String, String> {
        let client = self.client.as_ref()
            .ok_or_else(|| "OpenAI API key not configured".to_string())?;

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
    pub async fn generate_tags(&self, title: &str, content: &str) -> Result<Vec<String>, String> {
        let client = self.client.as_ref()
            .ok_or_else(|| "OpenAI API key not configured".to_string())?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_service_new_without_api_key() {
        let service = OpenAIService::new(None);
        assert!(!service.is_available());
    }

    #[test]
    fn test_openai_service_new_with_api_key() {
        let service = OpenAIService::new(Some("test-key".to_string()));
        assert!(service.is_available());
    }

    #[test]
    fn test_openai_service_availability() {
        let service_without_key = OpenAIService::new(None);
        let service_with_key = OpenAIService::new(Some("test-key".to_string()));
        
        assert!(!service_without_key.is_available());
        assert!(service_with_key.is_available());
    }
}