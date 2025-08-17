use sea_orm::*;
use models::{settings, dto::SettingsFormDTO};
use crate::services::BaseService;
use sea_orm::DatabaseConnection as DbConn;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct Service {
    base: BaseService,
}

impl Service {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    pub async fn get_settings(&self, db: &DbConn) -> Result<Option<settings::Model>, DbErr> {
        settings::Entity::find().one(db).await
    }

    pub async fn create_or_update_settings(&self, db: &DbConn, data: SettingsFormDTO) -> Result<settings::Model, DbErr> {
        // Check if settings already exist
        if let Some(existing) = self.get_settings(db).await? {
            // Update existing settings
            let mut settings_to_update: settings::ActiveModel = existing.into();
            settings_to_update.openai_api_key = Set(Some(data.openai_api_key));
            settings_to_update.openai_base_prompt = Set(Some(data.openai_base_prompt));
            settings_to_update.updated_at = Set(Utc::now().naive_utc());
            
            settings_to_update.update(db).await
        } else {
            // Create new settings
            let new_settings = settings::ActiveModel {
                id: Set(BaseService::generate_id()),
                openai_api_key: Set(Some(data.openai_api_key)),
                openai_base_prompt: Set(Some(data.openai_base_prompt)),
                created_at: Set(Utc::now().naive_utc()),
                updated_at: Set(Utc::now().naive_utc()),
            };
            
            new_settings.insert(db).await
        }
    }

    pub async fn get_openai_config(&self, db: &DbConn) -> Result<Option<(String, String)>, DbErr> {
        if let Some(settings) = self.get_settings(db).await? {
            if let (Some(api_key), Some(base_prompt)) = (settings.openai_api_key, settings.openai_base_prompt) {
                return Ok(Some((api_key, base_prompt)));
            }
        }
        Ok(None)
    }
}