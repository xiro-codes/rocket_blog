use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, Set, ActiveModelTrait};
use models::{settings, prelude::Settings};
use crate::services::BaseService;
use chrono::Utc;

pub struct SettingsService {
    base: BaseService,
}

impl SettingsService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    pub async fn get_setting(&self, db: &DatabaseConnection, key: &str) -> Result<Option<String>, sea_orm::DbErr> {
        let setting = Settings::find()
            .filter(settings::Column::Key.eq(key))
            .one(db)
            .await?;
        
        Ok(setting.map(|s| s.value).flatten())
    }

    pub async fn set_setting(&self, db: &DatabaseConnection, key: &str, value: &str) -> Result<(), sea_orm::DbErr> {
        let existing = Settings::find()
            .filter(settings::Column::Key.eq(key))
            .one(db)
            .await?;

        if let Some(setting) = existing {
            // Update existing setting
            let mut active_model: settings::ActiveModel = setting.into();
            active_model.value = Set(Some(value.to_string()));
            active_model.updated_at = Set(Utc::now().naive_utc());
            active_model.update(db).await?;
        } else {
            // Create new setting
            let new_setting = settings::ActiveModel {
                id: Set(BaseService::generate_id()),
                key: Set(key.to_string()),
                value: Set(Some(value.to_string())),
                description: Set(None),
                created_at: Set(Utc::now().naive_utc()),
                updated_at: Set(Utc::now().naive_utc()),
            };
            new_setting.insert(db).await?;
        }

        Ok(())
    }

    pub async fn get_all_settings(&self, db: &DatabaseConnection) -> Result<Vec<settings::Model>, sea_orm::DbErr> {
        Settings::find().all(db).await
    }

    pub async fn update_settings(&self, db: &DatabaseConnection, updates: Vec<(String, String)>) -> Result<(), sea_orm::DbErr> {
        for (key, value) in updates {
            self.set_setting(db, &key, &value).await?;
        }
        Ok(())
    }
}