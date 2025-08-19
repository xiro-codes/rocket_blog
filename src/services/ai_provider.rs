use async_trait::async_trait;
use sea_orm::DatabaseConnection;

/// Common interface for AI content generation providers (OpenAI, Ollama, etc.)
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Check if the AI provider is available and configured
    async fn is_available(&self, db: &DatabaseConnection) -> bool;
    
    /// Generate blog post content from a title and optional additional prompt
    async fn generate_post_content(
        &self,
        db: &DatabaseConnection,
        title: &str,
        additional_prompt: Option<&str>,
    ) -> Result<String, String>;
    
    /// Generate an excerpt from content
    async fn generate_excerpt(
        &self,
        db: &DatabaseConnection,
        content: &str,
    ) -> Result<String, String>;
    
    /// Generate suggested tags from title and content
    async fn generate_tags(
        &self,
        db: &DatabaseConnection,
        title: &str,
        content: &str,
    ) -> Result<Vec<String>, String>;
    
    /// Get the name of this AI provider
    fn provider_name(&self) -> &'static str;
}

/// Aggregator service that manages multiple AI providers
pub struct AIProviderService {
    providers: Vec<Box<dyn AIProvider>>,
}

impl AIProviderService {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
    
    /// Add an AI provider to the service
    pub fn add_provider(&mut self, provider: Box<dyn AIProvider>) {
        self.providers.push(provider);
    }
    
    /// Get the first available AI provider
    pub async fn get_available_provider(&self, db: &DatabaseConnection) -> Option<&dyn AIProvider> {
        for provider in &self.providers {
            if provider.is_available(db).await {
                return Some(provider.as_ref());
            }
        }
        None
    }
    
    /// Get all available providers
    pub async fn get_available_providers(&self, db: &DatabaseConnection) -> Vec<&dyn AIProvider> {
        let mut available = Vec::new();
        for provider in &self.providers {
            if provider.is_available(db).await {
                available.push(provider.as_ref());
            }
        }
        available
    }
    
    /// Check if any AI provider is available
    pub async fn is_any_available(&self, db: &DatabaseConnection) -> bool {
        for provider in &self.providers {
            if provider.is_available(db).await {
                return true;
            }
        }
        false
    }
}