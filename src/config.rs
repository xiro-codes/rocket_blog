use rocket::figment::Figment;

pub struct AppConfig {
    pub data_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_path: "/home/tod/.local/share/blog".to_string(), // fallback to original development path
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
            
        Self { data_path }
    }
}