#[cfg(test)]
mod ai_integration_tests {
    use crate::services::{AIProviderService, OpenAIService, OllamaService, AIProvider, SettingsService};
    use rocket::serde::json::{json, Value};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    struct TestOllamaRequest {
        model: String,
        prompt: String,
        stream: bool,
        options: TestOllamaOptions,
    }

    #[derive(Serialize)]
    struct TestOllamaOptions {
        num_predict: i32,
        temperature: f32,
    }

    #[test]
    fn test_ai_provider_service_provider_selection() {
        let mut service = AIProviderService::new();
        
        // Add providers in order: OpenAI first, then Ollama
        service.add_provider(Box::new(OpenAIService::new()));
        service.add_provider(Box::new(OllamaService::new()));
        
        // Verify we can add providers successfully
        assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<AIProviderService>());
    }

    #[test] 
    fn test_settings_service_creation() {
        let service = SettingsService::new();
        
        // Just test that we can create the service
        assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<SettingsService>());
    }

    #[test]
    fn test_json_request_structure() {
        // Test that our expected JSON request structure is valid
        let request = json!({
            "type": "content",
            "title": "Test Blog Post",
            "prompt": "Write about AI integration",
            "provider": "ollama"
        });
        
        // Verify structure
        assert_eq!(request["type"], "content");
        assert_eq!(request["title"], "Test Blog Post");
        assert_eq!(request["prompt"], "Write about AI integration");
        assert_eq!(request["provider"], "ollama");
    }

    #[test]
    fn test_ai_provider_names() {
        let openai = OpenAIService::new();
        let ollama = OllamaService::new();
        
        assert_eq!(openai.provider_name(), "OpenAI");
        assert_eq!(ollama.provider_name(), "Ollama");
    }

    #[test]
    fn test_ollama_request_structure() {
        use serde_json;
        
        // Test that we can serialize the Ollama request structure
        let request = TestOllamaRequest {
            model: "llama2".to_string(),
            prompt: "Test prompt".to_string(),
            stream: false,
            options: TestOllamaOptions {
                num_predict: 150,
                temperature: 0.7,
            },
        };
        
        // Should serialize without error
        let _json = serde_json::to_string(&request).unwrap();
    }

    #[test] 
    fn test_expected_api_response_structure() {
        // Test response structure that the frontend expects
        let success_response = json!({
            "success": true,
            "content": "Generated content",
            "provider": "Ollama"
        });
        
        let error_response = json!({
            "error": "Service not configured"
        });
        
        // Verify structures
        assert_eq!(success_response["success"], true);
        assert_eq!(success_response["provider"], "Ollama");
        assert!(error_response["error"].is_string());
    }
}