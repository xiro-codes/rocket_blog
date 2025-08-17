#[cfg(test)]
mod admin_tests {
    use crate::services::SettingsService;
    use models::dto::SettingsFormDTO;

    #[test]
    fn test_settings_service_creation() {
        let service = SettingsService::new();
        // Just verify the service can be created
        assert_eq!(std::any::type_name_of_val(&service), "app::services::settings::Service");
    }

    #[test]
    fn test_settings_dto_creation() {
        let dto = SettingsFormDTO {
            openai_api_key: "test-key".to_string(),
            openai_base_prompt: "test prompt".to_string(),
        };
        assert_eq!(dto.openai_api_key, "test-key");
        assert_eq!(dto.openai_base_prompt, "test prompt");
    }
}