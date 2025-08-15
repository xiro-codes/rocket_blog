use rocket::figment::{Figment, providers::Serialized};
use tempfile::TempDir;

// Utility functions for testing
pub fn create_test_figment(data_path: Option<String>) -> Figment {
    let mut figment = rocket::Config::default().figment();
    
    if let Some(path) = data_path {
        figment = figment.merge(Serialized::default("data_path", path));
    }
    
    figment
}

pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

pub fn get_temp_path() -> String {
    create_temp_dir().path().to_string_lossy().to_string()
}

// Mock rocket instance for testing
pub fn create_test_rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
}

// Helper macros for testing async functions
#[macro_export]
macro_rules! test_async {
    ($test:expr) => {
        tokio_test::block_on($test)
    };
}