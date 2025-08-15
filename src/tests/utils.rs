// Test utilities and simple test data
use rocket::figment::{Figment, providers::Serialized};

pub fn create_test_figment(data_path: Option<String>) -> Figment {
    let mut figment = rocket::Config::figment();
    
    if let Some(path) = data_path {
        figment = figment.merge(Serialized::default("data_path", path));
    }
    
    figment
}

pub fn create_temp_path() -> String {
    "/tmp/test_blog_data".to_string()
}

// Helper function for creating test rocket instance
pub fn create_basic_rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
}