use rocket::figment::Figment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub database_url: Option<String>,
    pub data_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Rocket App".to_string(),
            version: "0.1.0".to_string(),
            environment: "development".to_string(),
            database_url: None,
            data_path: None,
        }
    }
}

impl AppConfig {
    pub fn from_figment(figment: &Figment) -> Self {
        figment.extract().unwrap_or_default()
    }
    
    pub fn get_data_path(&self) -> String {
        self.data_path.clone().unwrap_or_else(|| {
            "/home/tod/.local/share/blog".to_string() // fallback to original development path
        })
    }
}