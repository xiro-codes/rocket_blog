use rocket::figment::Figment;

pub struct AppConfig {
    pub data_path: String,
    pub openai_api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_path: "/home/tod/.local/share/blog".to_string(), // fallback to original development path
            openai_api_key: None,
        }
    }
}

impl AppConfig {
    pub fn from_figment(figment: &Figment) -> Self {
        // Try to extract data_path from figment, otherwise use default
        let data_path = figment
            .find_value("data_path")
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| Self::default().data_path);

        // Try to get OpenAI API key from environment or figment
        let openai_api_key = std::env::var("OPENAI_API_KEY")
            .ok()
            .or_else(|| {
                figment
                    .find_value("openai_api_key")
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            });

        Self { 
            data_path,
            openai_api_key,
        }
    }
}
