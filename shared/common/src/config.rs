use rocket::figment::{Figment, providers::{Format, Serialized, Toml, Env}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub database_url: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Rocket App".to_string(),
            version: "0.1.0".to_string(),
            environment: "development".to_string(),
            database_url: None,
        }
    }
}

impl AppConfig {
    pub fn from_figment(figment: &Figment) -> Self {
        figment.extract().unwrap_or_default()
    }
}