use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, AeadCore, KeyInit, OsRng}};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::Local;
use models::prelude::*;
use models::settings;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::services::base::BaseService;
use crate::{impl_service_custom, services::base::ServiceHelpers};

pub struct SettingsService {
    base: BaseService,
    encryption_key: [u8; 32],
}

impl SettingsService {
    pub fn new() -> Self {
        // In production, this should be loaded from a secure location
        // For now, we'll use a default key, but this should be configurable
        let encryption_key = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
        ];
        
        Self {
            base: BaseService::new(),
            encryption_key,
        }
    }

    /// Encrypt a value using AES-256-GCM
    fn encrypt(&self, value: &str) -> Result<String, String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, value.as_bytes())
            .map_err(|e| format!("Encryption error: {}", e))?;
        
        // Combine nonce and ciphertext and encode in base64
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);
        
        Ok(BASE64.encode(combined))
    }

    /// Decrypt a value using AES-256-GCM
    fn decrypt(&self, encrypted_value: &str) -> Result<String, String> {
        let combined = BASE64
            .decode(encrypted_value)
            .map_err(|e| format!("Base64 decode error: {}", e))?;
        
        if combined.len() < 12 {
            return Err("Invalid encrypted value".to_string());
        }
        
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let key = Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption error: {}", e))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| format!("UTF-8 conversion error: {}", e))
    }

    /// Get a setting value by key
    pub async fn get_setting(&self, db: &DatabaseConnection, key: &str) -> Result<Option<String>, String> {
        let setting = Settings::find()
            .filter(settings::Column::Key.eq(key))
            .one(db)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        match setting {
            Some(setting) => {
                if let Some(value) = setting.value {
                    if setting.encrypted {
                        self.decrypt(&value).map(Some)
                    } else {
                        Ok(Some(value))
                    }
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Set a setting value by key
    pub async fn set_setting(
        &self,
        db: &DatabaseConnection,
        key: &str,
        value: &str,
        encrypted: bool,
    ) -> Result<(), String> {
        let now = Local::now().naive_local();
        
        let stored_value = if encrypted {
            self.encrypt(value)?
        } else {
            value.to_string()
        };

        // Check if setting already exists
        if let Some(existing) = Settings::find()
            .filter(settings::Column::Key.eq(key))
            .one(db)
            .await
            .map_err(|e| format!("Database error: {}", e))?
        {
            // Update existing setting
            let mut active_model: settings::ActiveModel = existing.into();
            active_model.value = Set(Some(stored_value));
            active_model.encrypted = Set(encrypted);
            active_model.updated_at = Set(now);
            
            active_model
                .update(db)
                .await
                .map_err(|e| format!("Failed to update setting: {}", e))?;
        } else {
            // Create new setting
            let new_setting = settings::ActiveModel {
                id: Set(Uuid::new_v4()),
                key: Set(key.to_string()),
                value: Set(Some(stored_value)),
                encrypted: Set(encrypted),
                created_at: Set(now),
                updated_at: Set(now),
            };
            
            new_setting
                .insert(db)
                .await
                .map_err(|e| format!("Failed to create setting: {}", e))?;
        }

        Ok(())
    }

    /// Delete a setting by key
    pub async fn delete_setting(&self, db: &DatabaseConnection, key: &str) -> Result<(), String> {
        Settings::delete_many()
            .filter(settings::Column::Key.eq(key))
            .exec(db)
            .await
            .map_err(|e| format!("Failed to delete setting: {}", e))?;
        
        Ok(())
    }

    /// Get OpenAI API key specifically
    pub async fn get_openai_api_key(&self, db: &DatabaseConnection) -> Result<Option<String>, String> {
        self.get_setting(db, "openai_api_key").await
    }

    /// Set OpenAI API key specifically
    pub async fn set_openai_api_key(&self, db: &DatabaseConnection, api_key: &str) -> Result<(), String> {
        self.set_setting(db, "openai_api_key", api_key, true).await
    }

    /// Get user timezone preference by account ID
    pub async fn get_user_timezone(&self, db: &DatabaseConnection, account_id: Uuid) -> Result<Option<String>, String> {
        let key = format!("user_timezone_{}", account_id);
        self.get_setting(db, &key).await
    }

    /// Set user timezone preference by account ID
    pub async fn set_user_timezone(&self, db: &DatabaseConnection, account_id: Uuid, timezone: &str) -> Result<(), String> {
        let key = format!("user_timezone_{}", account_id);
        self.set_setting(db, &key, timezone, false).await
    }

    /// Get user pay period settings by account ID
    pub async fn get_user_pay_period_settings(&self, db: &DatabaseConnection, account_id: Uuid) -> Result<Option<models::dto::PayPeriodSettingsDTO>, String> {
        let start_day_key = format!("user_pay_period_start_day_{}", account_id);
        let period_length_key = format!("user_pay_period_length_{}", account_id);
        
        let start_day = self.get_setting(db, &start_day_key).await?.unwrap_or_else(|| "monday".to_string());
        let period_length = self.get_setting(db, &period_length_key).await?
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(2); // Default to 2 weeks
            
        Ok(Some(models::dto::PayPeriodSettingsDTO {
            start_day,
            period_length,
        }))
    }

    /// Set user pay period settings by account ID
    pub async fn set_user_pay_period_settings(&self, db: &DatabaseConnection, account_id: Uuid, settings: &models::dto::PayPeriodSettingsFormDTO) -> Result<(), String> {
        let start_day_key = format!("user_pay_period_start_day_{}", account_id);
        let period_length_key = format!("user_pay_period_length_{}", account_id);
        
        self.set_setting(db, &start_day_key, &settings.start_day, false).await?;
        self.set_setting(db, &period_length_key, &settings.period_length.to_string(), false).await?;
        
        Ok(())
    }
}

impl_service_custom!(SettingsService);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let service = SettingsService::new();
        let original = "test_api_key_12345";
        
        let encrypted = service.encrypt(original).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        
        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let service = SettingsService::new();
        let original = "";
        
        let encrypted = service.encrypt(original).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        
        assert_eq!(original, decrypted);
    }
}