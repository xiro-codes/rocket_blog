#[cfg(test)]
mod tests {
    use common::config::AppConfig;
    use crate::tests::utils::create_test_figment;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.get_data_path(), "/home/tod/.local/share/blog");
    }

    #[test]
    fn test_app_config_from_figment_with_data_path() {
        let test_path = "/tmp/test_blog";
        let figment = create_test_figment(Some(test_path.to_string()));
        
        let config = AppConfig::from_figment(&figment);
        assert_eq!(config.get_data_path(), test_path);
    }

    #[test]
    fn test_app_config_from_figment_without_data_path() {
        let figment = create_test_figment(None);
        
        let config = AppConfig::from_figment(&figment);
        assert_eq!(config.data_path, "/home/tod/.local/share/blog");
    }

    #[test]
    fn test_app_config_from_figment_empty_figment() {
        let figment = rocket::Config::figment();
        
        let config = AppConfig::from_figment(&figment);
        assert_eq!(config.data_path, "/home/tod/.local/share/blog");
    }
}