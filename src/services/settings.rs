use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, AeadCore, KeyInit, OsRng}};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::Utc;
use models::prelude::*;
use models::settings;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::services::base::BaseService;

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
        let now = Utc::now();
        
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

    /// Test if an OpenAI API key is valid by making a simple API call
    pub async fn test_openai_api_key(api_key: &str) -> Result<bool, String> {
        use async_openai::{Client, types::{
            ChatCompletionRequestMessage, ChatCompletionRequestUserMessage, 
            CreateChatCompletionRequestArgs, Role
        }};

        let client = Client::with_config(
            async_openai::config::OpenAIConfig::new().with_api_key(api_key)
        );

        let messages = vec![
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text("Hello".to_string()),
                    role: Role::User,
                    name: None,
                }
            ),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(messages)
            .max_tokens(1u16)
            .build()
            .map_err(|e| format!("Failed to build test request: {}", e))?;

        match client.chat().create(request).await {
            Ok(_) => Ok(true),
            Err(e) => {
                let error_msg = format!("{}", e);
                if error_msg.contains("unauthorized") || error_msg.contains("invalid_api_key") {
                    Ok(false)
                } else {
                    Err(format!("API test error: {}", e))
                }
            }
        }
    }
}

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